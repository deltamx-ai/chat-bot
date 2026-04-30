use super::{InMemoryToolRouter, Tool};

pub struct ToolRegistry;

impl ToolRegistry {
    pub fn with_tools(tools: Vec<Box<dyn Tool>>) -> InMemoryToolRouter {
        let mut router = InMemoryToolRouter::new();
        for tool in tools {
            router.register(tool);
        }
        router
    }
}
