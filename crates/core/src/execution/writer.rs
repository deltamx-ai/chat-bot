//! Write operations for files, patches, and generated output.

use serde_json::{Value, json};

use super::{ExecutionContext, Tool, ToolError, ToolOutput};

pub struct WriteTool;

impl Tool for WriteTool {
    fn name(&self) -> &str {
        "write"
    }

    fn description(&self) -> &str {
        "Write changes or generated output to the workspace."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "content": { "type": "string" }
            },
            "required": ["path", "content"]
        })
    }

    fn call(&self, _ctx: ExecutionContext, input: Value) -> Result<ToolOutput, ToolError> {
        Ok(ToolOutput { content: input })
    }
}
