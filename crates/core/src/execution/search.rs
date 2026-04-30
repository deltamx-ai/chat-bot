//! Search and retrieval capabilities used during execution.

use serde_json::{Value, json};

use super::{ExecutionContext, Tool, ToolError, ToolOutput};

pub struct SearchTool;

impl Tool for SearchTool {
    fn name(&self) -> &str {
        "search"
    }

    fn description(&self) -> &str {
        "Search project content and return matching references."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": { "type": "string" }
            },
            "required": ["query"]
        })
    }

    fn call(&self, _ctx: ExecutionContext, input: Value) -> Result<ToolOutput, ToolError> {
        Ok(ToolOutput { content: input })
    }
}
