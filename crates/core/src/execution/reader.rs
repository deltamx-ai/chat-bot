//! Read operations for files, content, and external resources.

use serde_json::{Value, json};

use super::{ExecutionContext, Tool, ToolError, ToolOutput};

pub struct ReadTool;

impl Tool for ReadTool {
    fn name(&self) -> &str {
        "read"
    }

    fn description(&self) -> &str {
        "Read structured content from the current workspace."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" }
            },
            "required": ["path"]
        })
    }

    fn call(&self, _ctx: ExecutionContext, input: Value) -> Result<ToolOutput, ToolError> {
        Ok(ToolOutput { content: input })
    }
}
