//! Write operations for files, patches, and generated output.

use std::{fs, path::PathBuf};

use serde::Deserialize;
use serde_json::{Value, json};

use super::{ExecutionContext, Tool, ToolError, ToolOutput};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum WriteMode {
    Overwrite,
    Append,
}

#[derive(Debug, Deserialize)]
struct WriteFileInput {
    path: String,
    content: String,
    mode: Option<WriteMode>,
}

pub struct WriteTool;

impl Tool for WriteTool {
    fn name(&self) -> &str {
        "write"
    }

    fn description(&self) -> &str {
        "Write or append file content in the workspace."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "content": { "type": "string" },
                "mode": { "type": "string", "enum": ["overwrite", "append"] }
            },
            "required": ["path", "content"]
        })
    }

    fn call(&self, ctx: ExecutionContext, input: Value) -> Result<ToolOutput, ToolError> {
        let input: WriteFileInput = serde_json::from_value(input).map_err(|err| ToolError {
            code: "invalid_write_input".into(),
            message: err.to_string(),
        })?;

        let path = resolve_workspace_path(&ctx, &input.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| ToolError {
                code: "create_parent_failed".into(),
                message: format!("{}: {err}", parent.display()),
            })?;
        }

        match input.mode.unwrap_or(WriteMode::Overwrite) {
            WriteMode::Overwrite => fs::write(&path, input.content.as_bytes()),
            WriteMode::Append => {
                use std::io::Write;
                let mut file = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&path)
                    .map_err(|err| ToolError {
                        code: "open_append_failed".into(),
                        message: format!("{}: {err}", path.display()),
                    })?;
                file.write_all(input.content.as_bytes())
            }
        }
        .map_err(|err| ToolError {
            code: "write_failed".into(),
            message: format!("{}: {err}", path.display()),
        })?;

        Ok(ToolOutput {
            content: json!({
                "path": path.display().to_string(),
                "bytes_written": input.content.len()
            }),
        })
    }
}

fn resolve_workspace_path(ctx: &ExecutionContext, path: &str) -> PathBuf {
    if let Some(workspace) = &ctx.workspace {
        PathBuf::from(&workspace.0).join(path)
    } else {
        PathBuf::from(path)
    }
}
