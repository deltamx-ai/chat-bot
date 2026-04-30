//! Validation checks performed before or during execution.

use serde_json::{Value, json};

use super::{ExecutionContext, Tool, ToolError, ToolOutput};

pub struct ValidateTool;

impl Tool for ValidateTool {
    fn name(&self) -> &str {
        "validate"
    }

    fn description(&self) -> &str {
        "Validate workspace state and return a structured report."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "target": { "type": "string" }
            },
            "required": ["target"]
        })
    }

    fn call(&self, _ctx: ExecutionContext, input: Value) -> Result<ToolOutput, ToolError> {
        Ok(ToolOutput { content: input })
    }
}
