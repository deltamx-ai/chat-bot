//! Search and retrieval capabilities used during execution.

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use serde_json::{Value, json};

use super::{ExecutionContext, Tool, ToolError, ToolOutput};

#[derive(Debug, Deserialize)]
struct SearchFilesInput {
    query: String,
    path: Option<String>,
    case_sensitive: Option<bool>,
}

pub struct SearchTool;

impl Tool for SearchTool {
    fn name(&self) -> &str {
        "search"
    }

    fn description(&self) -> &str {
        "Search file contents under the workspace and return matching lines."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": { "type": "string" },
                "path": { "type": ["string", "null"] },
                "case_sensitive": { "type": ["boolean", "null"] }
            },
            "required": ["query"]
        })
    }

    fn call(&self, ctx: ExecutionContext, input: Value) -> Result<ToolOutput, ToolError> {
        let input: SearchFilesInput = serde_json::from_value(input).map_err(|err| ToolError {
            code: "invalid_search_input".into(),
            message: err.to_string(),
        })?;

        let root = resolve_workspace_path(&ctx, input.path.as_deref().unwrap_or("."));
        let query = if input.case_sensitive.unwrap_or(false) {
            input.query.clone()
        } else {
            input.query.to_lowercase()
        };

        let mut matches = Vec::new();
        visit_files(&root, &mut |path| {
            if let Ok(content) = fs::read_to_string(path) {
                for (index, line) in content.lines().enumerate() {
                    let haystack = if input.case_sensitive.unwrap_or(false) {
                        line.to_string()
                    } else {
                        line.to_lowercase()
                    };
                    if haystack.contains(&query) {
                        matches.push(json!({
                            "path": path.display().to_string(),
                            "line": index + 1,
                            "content": line,
                        }));
                    }
                }
            }
        })
        .map_err(|err| ToolError {
            code: "search_failed".into(),
            message: err,
        })?;

        Ok(ToolOutput {
            content: json!({
                "query": input.query,
                "matches": matches,
                "count": matches.len()
            }),
        })
    }
}

fn visit_files(root: &Path, visit: &mut dyn FnMut(&Path)) -> Result<(), String> {
    if root.is_file() {
        visit(root);
        return Ok(());
    }

    let entries = fs::read_dir(root).map_err(|err| format!("{}: {err}", root.display()))?;
    for entry in entries {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().and_then(|name| name.to_str()) == Some(".git") {
                continue;
            }
            visit_files(&path, visit)?;
        } else if path.is_file() {
            visit(&path);
        }
    }
    Ok(())
}

fn resolve_workspace_path(ctx: &ExecutionContext, path: &str) -> PathBuf {
    if let Some(workspace) = &ctx.workspace {
        PathBuf::from(&workspace.0).join(path)
    } else {
        PathBuf::from(path)
    }
}
