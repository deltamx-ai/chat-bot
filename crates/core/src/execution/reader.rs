//! Read operations for files, content, and external resources.

use std::{fs, path::PathBuf};

use serde::Deserialize;
use serde_json::{Value, json};

use super::{ExecutionContext, Tool, ToolError, ToolOutput};

#[derive(Debug, Deserialize)]
struct ReadFileInput {
    path: String,
    offset: Option<usize>,
    limit: Option<usize>,
}

pub struct ReadTool;

impl Tool for ReadTool {
    fn name(&self) -> &str {
        "read"
    }

    fn description(&self) -> &str {
        "Read file content from the workspace."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "offset": { "type": ["integer", "null"] },
                "limit": { "type": ["integer", "null"] }
            },
            "required": ["path"]
        })
    }

    fn call(&self, ctx: ExecutionContext, input: Value) -> Result<ToolOutput, ToolError> {
        let input: ReadFileInput = serde_json::from_value(input).map_err(|err| ToolError {
            code: "invalid_read_input".into(),
            message: err.to_string(),
        })?;

        let path = resolve_workspace_path(&ctx, &input.path);
        let content = fs::read_to_string(&path).map_err(|err| ToolError {
            code: "read_failed".into(),
            message: format!("{}: {err}", path.display()),
        })?;

        let lines: Vec<&str> = content.lines().collect();
        let offset = input.offset.unwrap_or(0);
        let limit = input.limit.unwrap_or(lines.len().saturating_sub(offset));
        let sliced = lines
            .iter()
            .skip(offset)
            .take(limit)
            .copied()
            .collect::<Vec<_>>()
            .join("\n");

        Ok(ToolOutput {
            content: json!({
                "path": path.display().to_string(),
                "offset": offset,
                "limit": limit,
                "content": sliced,
                "total_lines": lines.len()
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
