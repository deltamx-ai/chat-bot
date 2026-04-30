use serde_json::Value;

use super::{ExecutionContext, ToolError, ToolOutput};

pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> Value;
    fn call(&self, ctx: ExecutionContext, input: Value) -> Result<ToolOutput, ToolError>;
}
