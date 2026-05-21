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

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
const USER_AGENT: &str = "chat-bot/0.1";
const DEFAULT_MODEL: &str = "gpt-4o";

pub struct OpenAiProvider {
    api_key: String,
    base_url: String,
    http: Client,
}

impl OpenAiProvider {
    pub fn new(api_key: String, base_url: Option<String>) -> Result<Self, ProviderError> {
        if api_key.trim().is_empty() {
            return Err(ProviderError::MissingToken);
        }
        let base_url = base_url
            .map(|url| url.trim_end_matches('/').to_string())
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
        let http = Client::builder()
            .user_agent(USER_AGENT)
            .https_only(true)
            .build()
            .map_err(|err| ProviderError::Network(format!("build http client: {err}")))?;
        Ok(Self {
            api_key,
            base_url,
            http,
        })
    }

    fn chat_url(&self) -> String {
        format!("{}/chat/completions", self.base_url)
    }

    async fn call(&self, body: &serde_json::Value) -> Result<reqwest::Response, ProviderError> {
        self.http
            .post(self.chat_url())
            .bearer_auth(&self.api_key)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .map_err(|err| ProviderError::Network(err.to_string()))
    }
}

#[async_trait]
impl Provider for OpenAiProvider {
    async fn generate(
        &self,
        request: &GenerateRequest<'_>,
    ) -> Result<GenerateResponse, ProviderError> {
        let prompt = compose_prompt(request.prompt, request.attachments);
        let model = resolve_openai_model(request.model_id);
        let messages = build_messages(request.history, &prompt);

        let body = json!({
            "model": model,
            "messages": messages,
            "stream": false,
        });

        let response = self.call(&body).await?;
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
        let model = resolve_openai_model(request.model_id);
        let messages = build_messages(request.history, &prompt);

        let body = json!({
            "model": model,
            "messages": messages,
            "stream": true,
        });

        let response = self.call(&body).await?;
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

fn resolve_openai_model(requested: &str) -> String {
    let trimmed = requested.trim();
    if trimmed.is_empty() {
        return DEFAULT_MODEL.into();
    }
    if let Some(rest) = trimmed.strip_prefix("openai:") {
        return rest.to_string();
    }
    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::resolve_openai_model;

    #[test]
    fn resolver_passes_through_plain_models() {
        assert_eq!(resolve_openai_model("gpt-4o"), "gpt-4o");
        assert_eq!(resolve_openai_model("gpt-4o-mini"), "gpt-4o-mini");
    }

    #[test]
    fn resolver_strips_openai_prefix() {
        assert_eq!(resolve_openai_model("openai:gpt-4o"), "gpt-4o");
        assert_eq!(resolve_openai_model("openai:o3-mini"), "o3-mini");
    }

    #[test]
    fn resolver_falls_back_for_empty() {
        assert_eq!(resolve_openai_model(""), "gpt-4o");
        assert_eq!(resolve_openai_model("   "), "gpt-4o");
    }
}
