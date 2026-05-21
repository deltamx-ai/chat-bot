use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::error::ProviderError;
use super::runtime::{
    AlmaProvider, AnthropicProvider, CopilotProvider, CopilotTokenSource, OpenAiProvider, Provider,
};
use super::{ProviderDescriptor, ProviderKind};

#[derive(Debug, Clone, Default)]
pub struct ProviderRegistry {
    providers: Vec<ProviderDescriptor>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelSelection {
    pub model_id: String,
    pub kind: ProviderKind,
}

impl ModelSelection {
    pub fn new(model_id: impl Into<String>) -> Result<Self, ProviderError> {
        let model_id = model_id.into();
        if model_id.trim().is_empty() {
            return Err(ProviderError::MissingModel);
        }
        let kind = ProviderKind::from_model_id(&model_id);
        Ok(Self { model_id, kind })
    }
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, provider: ProviderDescriptor) {
        self.providers.push(provider);
    }

    pub fn all(&self) -> &[ProviderDescriptor] {
        &self.providers
    }

    pub fn provider_for(
        kind: ProviderKind,
        copilot_token_source: Option<Arc<dyn CopilotTokenSource>>,
    ) -> Box<dyn Provider> {
        if matches!(kind, ProviderKind::Copilot)
            && let Some(source) = copilot_token_source
            && let Ok(provider) = CopilotProvider::new(source)
        {
            return Box::new(provider);
        }
        if matches!(kind, ProviderKind::Anthropic)
            && let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY")
            && let Ok(provider) = AnthropicProvider::new(api_key)
        {
            return Box::new(provider);
        }
        if matches!(kind, ProviderKind::OpenAi)
            && let Ok(api_key) = std::env::var("OPENAI_API_KEY")
        {
            let base_url = std::env::var("OPENAI_BASE_URL").ok();
            if let Ok(provider) = OpenAiProvider::new(api_key, base_url) {
                return Box::new(provider);
            }
        }
        match kind {
            ProviderKind::Anthropic
            | ProviderKind::OpenAi
            | ProviderKind::Copilot
            | ProviderKind::Custom => Box::new(AlmaProvider::default()),
        }
    }
}
