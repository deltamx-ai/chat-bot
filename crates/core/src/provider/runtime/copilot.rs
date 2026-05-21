use std::sync::Arc;

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde_json::json;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

use super::super::error::ProviderError;
use super::openai_chat::{
    ChatCompletionResponse, FrameOutcome, build_messages, extract_completion_content, process_frame,
};
use super::{Delta, GenerateRequest, GenerateResponse, Provider, compose_prompt};

const CHAT_COMPLETIONS_URL: &str = "https://api.githubcopilot.com/chat/completions";
const EDITOR_VERSION: &str = "chat-bot/0.1";
const EDITOR_PLUGIN_VERSION: &str = "chat-bot/0.1";
const COPILOT_INTEGRATION_ID: &str = "vscode-chat";
const USER_AGENT: &str = "chat-bot/0.1";
const DEFAULT_COPILOT_MODEL: &str = "gpt-4o";

#[async_trait]
pub trait CopilotTokenSource: Send + Sync {
    async fn copilot_token(&self) -> Result<String, ProviderError>;
    async fn refresh_copilot_token(&self) -> Result<String, ProviderError>;
}

pub struct CopilotProvider {
    token_source: Arc<dyn CopilotTokenSource>,
    http: Client,
}

impl CopilotProvider {
    pub fn new(token_source: Arc<dyn CopilotTokenSource>) -> Result<Self, ProviderError> {
        let http = Client::builder()
            .user_agent(USER_AGENT)
            .https_only(true)
            .build()
            .map_err(|err| ProviderError::Network(format!("build http client: {err}")))?;
        Ok(Self { token_source, http })
    }

    async fn call(
        &self,
        token: &str,
        body: &serde_json::Value,
    ) -> Result<reqwest::Response, ProviderError> {
        self.http
            .post(CHAT_COMPLETIONS_URL)
            .bearer_auth(token)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .header("Editor-Version", EDITOR_VERSION)
            .header("Editor-Plugin-Version", EDITOR_PLUGIN_VERSION)
            .header("Copilot-Integration-Id", COPILOT_INTEGRATION_ID)
            .json(body)
            .send()
            .await
            .map_err(|err| ProviderError::Network(err.to_string()))
    }
}

#[async_trait]
impl Provider for CopilotProvider {
    async fn generate(
        &self,
        request: &GenerateRequest<'_>,
    ) -> Result<GenerateResponse, ProviderError> {
        let prompt = compose_prompt(request.prompt, request.attachments);
        let upstream_model = resolve_copilot_model(request.model_id);
        let messages = build_messages(request.history, &prompt);

        let body = json!({
            "model": upstream_model,
            "messages": messages,
            "stream": false,
        });

        let mut token = self.token_source.copilot_token().await?;
        let mut response = self.call(&token, &body).await?;

        if response.status() == StatusCode::UNAUTHORIZED {
            token = self.token_source.refresh_copilot_token().await?;
            response = self.call(&token, &body).await?;
        }

        let status = response.status();
        if status == StatusCode::UNAUTHORIZED {
            let body = response.text().await.unwrap_or_default();
            return Err(ProviderError::Unauthorized(body));
        }
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ProviderError::NonZeroExit {
                code: Some(status.as_u16() as i32),
                stderr: body,
            });
        }

        let payload: ChatCompletionResponse = response
            .json()
            .await
            .map_err(|err| ProviderError::Io(format!("decode chat response: {err}")))?;
        let content = extract_completion_content(payload)?;
        Ok(GenerateResponse { content })
    }

    async fn generate_stream(
        &self,
        request: &GenerateRequest<'_>,
        sender: mpsc::Sender<Delta>,
    ) -> Result<GenerateResponse, ProviderError> {
        let prompt = compose_prompt(request.prompt, request.attachments);
        let upstream_model = resolve_copilot_model(request.model_id);
        let messages = build_messages(request.history, &prompt);

        let body = json!({
            "model": upstream_model,
            "messages": messages,
            "stream": true,
        });

        let mut token = self.token_source.copilot_token().await?;
        let mut response = self.call(&token, &body).await?;

        if response.status() == StatusCode::UNAUTHORIZED {
            token = self.token_source.refresh_copilot_token().await?;
            response = self.call(&token, &body).await?;
        }

        let status = response.status();
        if status == StatusCode::UNAUTHORIZED {
            let body = response.text().await.unwrap_or_default();
            return Err(ProviderError::Unauthorized(body));
        }
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ProviderError::NonZeroExit {
                code: Some(status.as_u16() as i32),
                stderr: body,
            });
        }

        let mut stream = response.bytes_stream();
        let mut buffer = String::new();
        let mut content_acc = String::new();
        let mut done = false;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|err| ProviderError::Network(err.to_string()))?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(idx) = buffer.find("\n\n") {
                let frame: String = buffer.drain(..idx + 2).collect();
                match process_frame(&frame, &sender, &mut content_acc).await? {
                    FrameOutcome::Continue => {}
                    FrameOutcome::Done => {
                        done = true;
                        break;
                    }
                }
            }

            if done {
                break;
            }
        }

        if content_acc.trim().is_empty() {
            return Err(ProviderError::EmptyOutput);
        }
        Ok(GenerateResponse {
            content: content_acc,
        })
    }
}

fn resolve_copilot_model(requested: &str) -> String {
    let trimmed = requested.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("copilot-chat") {
        return DEFAULT_COPILOT_MODEL.into();
    }
    if let Some(rest) = trimmed.strip_prefix("copilot:") {
        return rest.to_string();
    }
    if let Some(rest) = trimmed.strip_prefix("copilot-") {
        return rest.to_string();
    }
    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::resolve_copilot_model;

    #[test]
    fn copilot_chat_maps_to_default() {
        assert_eq!(resolve_copilot_model("copilot-chat"), "gpt-4o");
        assert_eq!(resolve_copilot_model("Copilot-Chat"), "gpt-4o");
    }

    #[test]
    fn empty_or_blank_falls_back() {
        assert_eq!(resolve_copilot_model(""), "gpt-4o");
        assert_eq!(resolve_copilot_model("   "), "gpt-4o");
    }

    #[test]
    fn colon_prefix_strips() {
        assert_eq!(resolve_copilot_model("copilot:gpt-4o-mini"), "gpt-4o-mini");
        assert_eq!(
            resolve_copilot_model("copilot:claude-3.5-sonnet"),
            "claude-3.5-sonnet"
        );
    }

    #[test]
    fn dash_prefix_strips() {
        assert_eq!(resolve_copilot_model("copilot-o1"), "o1");
    }

    #[test]
    fn plain_model_passes_through() {
        assert_eq!(resolve_copilot_model("gpt-4o"), "gpt-4o");
    }
}
