use std::collections::HashMap;

use serde_json::Value;

use super::{ExecutionContext, Tool, ToolError, ToolOutput};

#[derive(Default)]
pub struct InMemoryToolRouter {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl InMemoryToolRouter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub fn call(
        &self,
        ctx: ExecutionContext,
        name: &str,
        input: Value,
    ) -> Result<ToolOutput, ToolError> {
        let tool = self.tools.get(name).ok_or_else(|| ToolError {
            code: "tool_not_found".into(),
            message: format!("tool `{name}` not registered"),
        })?;

        tool.call(ctx, input)
    }
}
