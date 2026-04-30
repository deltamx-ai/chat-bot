use super::{InMemoryToolRouter, ReadTool, SearchTool, Tool, ValidateTool, WriteTool};

pub struct ToolRegistry;

impl ToolRegistry {
    pub fn with_tools(tools: Vec<Box<dyn Tool>>) -> InMemoryToolRouter {
        let mut router = InMemoryToolRouter::new();
        for tool in tools {
            router.register(tool);
        }
        router
    }

    pub fn default_router() -> InMemoryToolRouter {
        Self::with_tools(vec![
            Box::new(ReadTool),
            Box::new(SearchTool),
            Box::new(WriteTool),
            Box::new(ValidateTool),
        ])
    }
}
