use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::conversation::MessageAttachment;

use super::error::ProviderError;

mod alma;
mod anthropic;
mod copilot;
mod openai;
mod openai_chat;

pub use alma::AlmaProvider;
pub use anthropic::AnthropicProvider;
pub use copilot::{CopilotProvider, CopilotTokenSource};
pub use openai::OpenAiProvider;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatRole {
    System,
    User,
    Assistant,
    Tool,
}

impl ChatRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::Tool => "tool",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatTurn {
    pub role: ChatRole,
    pub content: String,
}

impl ChatTurn {
    pub fn new(role: ChatRole, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
        }
    }
}

pub struct GenerateRequest<'a> {
    pub model_id: &'a str,
    pub prompt: &'a str,
    pub attachments: &'a [MessageAttachment],
    pub history: &'a [ChatTurn],
}

pub struct GenerateResponse {
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct Delta {
    pub content: String,
}

#[async_trait]
pub trait Provider: Send + Sync {
    async fn generate(
        &self,
        request: &GenerateRequest<'_>,
    ) -> Result<GenerateResponse, ProviderError>;

    async fn generate_stream(
        &self,
        request: &GenerateRequest<'_>,
        sender: mpsc::Sender<Delta>,
    ) -> Result<GenerateResponse, ProviderError> {
        let response = self.generate(request).await?;
        let _ = sender
            .send(Delta {
                content: response.content.clone(),
            })
            .await;
        Ok(response)
    }
}

pub(crate) fn compose_prompt(prompt: &str, attachments: &[MessageAttachment]) -> String {
    if attachments.is_empty() {
        return prompt.to_string();
    }
    let list = attachments
        .iter()
        .map(|item| format!("- {} ({})", item.name, item.kind))
        .collect::<Vec<_>>()
        .join("\n");
    format!("{prompt}\n\nAttachments:\n{list}")
}
