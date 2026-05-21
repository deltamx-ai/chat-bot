use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderKind {
    Copilot,
    OpenAi,
    Anthropic,
    Custom,
}

impl ProviderKind {
    pub fn from_model_id(model_id: &str) -> Self {
        if model_id.starts_with("claude") || model_id.starts_with("anthropic:") {
            Self::Anthropic
        } else if model_id.starts_with("copilot") {
            Self::Copilot
        } else if model_id.starts_with("gpt")
            || model_id.starts_with("o1")
            || model_id.starts_with("o3")
            || model_id.starts_with("openai:")
        {
            Self::OpenAi
        } else {
            Self::Custom
        }
    }

    pub fn as_slug(self) -> &'static str {
        match self {
            Self::Copilot => "copilot",
            Self::OpenAi => "openai",
            Self::Anthropic => "anthropic",
            Self::Custom => "custom",
        }
    }
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
