use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderKind {
    Copilot,
    OpenAi,
    Anthropic,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderCapability {
    Chat,
    Embeddings,
    Search,
    ToolUse,
    Authentication,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderDescriptor {
    pub id: String,
    pub kind: ProviderKind,
    pub display_name: String,
    pub capabilities: Vec<ProviderCapability>,
}
