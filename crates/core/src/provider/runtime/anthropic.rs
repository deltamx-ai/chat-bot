use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use serde_json::json;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

use super::super::error::ProviderError;
use super::{
    ChatRole, ChatTurn, Delta, GenerateRequest, GenerateResponse, Provider, compose_prompt,
};

const MESSAGES_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";
const USER_AGENT: &str = "chat-bot/0.1";
const DEFAULT_MAX_TOKENS: u32 = 4096;
const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514";

pub struct AnthropicProvider {
    api_key: String,
    http: Client,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Result<Self, ProviderError> {
        if api_key.trim().is_empty() {
            return Err(ProviderError::MissingToken);
        }
        let http = Client::builder()
            .user_agent(USER_AGENT)
            .https_only(true)
            .build()
            .map_err(|err| ProviderError::Network(format!("build http client: {err}")))?;
        Ok(Self { api_key, http })
    }

    async fn call(&self, body: &serde_json::Value) -> Result<reqwest::Response, ProviderError> {
        self.http
            .post(MESSAGES_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await
            .map_err(|err| ProviderError::Network(err.to_string()))
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    async fn generate(
        &self,
        request: &GenerateRequest<'_>,
    ) -> Result<GenerateResponse, ProviderError> {
        let prompt = compose_prompt(request.prompt, request.attachments);
        let model = resolve_anthropic_model(request.model_id);
        let (system, messages) = build_messages(request.history, &prompt);

        let mut body = json!({
            "model": model,
            "max_tokens": DEFAULT_MAX_TOKENS,
            "messages": messages,
            "stream": false,
        });
        if let Some(system) = system {
            body["system"] = serde_json::Value::String(system);
        }

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

        let payload: MessageResponse = response
            .json()
            .await
            .map_err(|err| ProviderError::Io(format!("decode message response: {err}")))?;
        let content = payload
            .content
            .into_iter()
            .filter_map(|block| {
                if block.kind == "text" {
                    block.text
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("");
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return Err(ProviderError::EmptyOutput);
        }
        Ok(GenerateResponse { content })
    }

    async fn generate_stream(
        &self,
        request: &GenerateRequest<'_>,
        sender: mpsc::Sender<Delta>,
    ) -> Result<GenerateResponse, ProviderError> {
        let prompt = compose_prompt(request.prompt, request.attachments);
        let model = resolve_anthropic_model(request.model_id);
        let (system, messages) = build_messages(request.history, &prompt);

        let mut body = json!({
            "model": model,
            "max_tokens": DEFAULT_MAX_TOKENS,
            "messages": messages,
            "stream": true,
        });
        if let Some(system) = system {
            body["system"] = serde_json::Value::String(system);
        }

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

#[derive(Debug)]
enum FrameOutcome {
    Continue,
    Done,
}

async fn process_frame(
    frame: &str,
    sender: &mpsc::Sender<Delta>,
    content_acc: &mut String,
) -> Result<FrameOutcome, ProviderError> {
    let mut event_name: Option<&str> = None;
    let mut data_parts: Vec<&str> = Vec::new();
    for line in frame.lines() {
        if let Some(rest) = line.strip_prefix("event:") {
            event_name = Some(rest.trim());
        } else if let Some(rest) = line.strip_prefix("data:") {
            data_parts.push(rest.trim_start_matches(' '));
        }
    }
    if data_parts.is_empty() {
        return Ok(FrameOutcome::Continue);
    }
    let data = data_parts.join("\n");
    let trimmed = data.trim();
    match event_name {
        Some("message_stop") => return Ok(FrameOutcome::Done),
        Some("content_block_delta") => {}
        _ => return Ok(FrameOutcome::Continue),
    }
    let Ok(parsed) = serde_json::from_str::<ContentBlockDelta>(trimmed) else {
        return Ok(FrameOutcome::Continue);
    };
    let Some(text) = parsed.delta.text_for_text_delta() else {
        return Ok(FrameOutcome::Continue);
    };
    if text.is_empty() {
        return Ok(FrameOutcome::Continue);
    }
    content_acc.push_str(&text);
    if sender.send(Delta { content: text }).await.is_err() {
        return Err(ProviderError::Cancelled);
    }
    Ok(FrameOutcome::Continue)
}

fn resolve_anthropic_model(requested: &str) -> String {
    let trimmed = requested.trim();
    if trimmed.is_empty() {
        return DEFAULT_MODEL.into();
    }
    match trimmed {
        "claude-sonnet-4" => "claude-sonnet-4-20250514".into(),
        "claude-opus-4" => "claude-opus-4-20250514".into(),
        "claude-haiku-4-5" => "claude-haiku-4-5-20251001".into(),
        other => other.into(),
    }
}

fn build_messages(
    history: &[ChatTurn],
    new_user_prompt: &str,
) -> (Option<String>, Vec<serde_json::Value>) {
    let mut system_parts: Vec<String> = Vec::new();
    let mut messages: Vec<serde_json::Value> = Vec::with_capacity(history.len() + 1);
    for turn in history {
        if matches!(turn.role, ChatRole::Tool) {
            continue;
        }
        if turn.content.trim().is_empty() {
            continue;
        }
        match turn.role {
            ChatRole::System => system_parts.push(turn.content.clone()),
            ChatRole::User | ChatRole::Assistant => {
                messages.push(json!({
                    "role": turn.role.as_str(),
                    "content": turn.content,
                }));
            }
            ChatRole::Tool => {}
        }
    }
    messages.push(json!({
        "role": "user",
        "content": new_user_prompt,
    }));
    let system = if system_parts.is_empty() {
        None
    } else {
        Some(system_parts.join("\n\n"))
    };
    (system, messages)
}

#[derive(Deserialize)]
struct MessageResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    text: Option<String>,
}

#[derive(Deserialize)]
struct ContentBlockDelta {
    delta: StreamDelta,
}

#[derive(Deserialize)]
struct StreamDelta {
    #[serde(default, rename = "type")]
    kind: Option<String>,
    #[serde(default)]
    text: Option<String>,
}

impl StreamDelta {
    fn text_for_text_delta(self) -> Option<String> {
        if self.kind.as_deref() == Some("text_delta") {
            self.text
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ChatRole, ChatTurn, Delta, FrameOutcome, ProviderError, build_messages, process_frame,
        resolve_anthropic_model,
    };
    use serde_json::json;
    use tokio::sync::mpsc;

    #[test]
    fn resolver_maps_friendly_names() {
        assert_eq!(
            resolve_anthropic_model("claude-sonnet-4"),
            "claude-sonnet-4-20250514"
        );
        assert_eq!(
            resolve_anthropic_model("claude-opus-4"),
            "claude-opus-4-20250514"
        );
        assert_eq!(
            resolve_anthropic_model("claude-haiku-4-5"),
            "claude-haiku-4-5-20251001"
        );
    }

    #[test]
    fn resolver_passes_through_dated_models() {
        assert_eq!(
            resolve_anthropic_model("claude-sonnet-4-20250514"),
            "claude-sonnet-4-20250514"
        );
        assert_eq!(
            resolve_anthropic_model("claude-3-5-sonnet-20241022"),
            "claude-3-5-sonnet-20241022"
        );
    }

    #[test]
    fn resolver_falls_back_for_empty() {
        assert_eq!(resolve_anthropic_model(""), "claude-sonnet-4-20250514");
        assert_eq!(resolve_anthropic_model("   "), "claude-sonnet-4-20250514");
    }

    #[test]
    fn build_messages_separates_system_role() {
        let history = vec![
            ChatTurn::new(ChatRole::System, "be terse"),
            ChatTurn::new(ChatRole::User, "hi"),
            ChatTurn::new(ChatRole::Assistant, "hello"),
        ];
        let (system, messages) = build_messages(&history, "next");
        assert_eq!(system.as_deref(), Some("be terse"));
        assert_eq!(
            messages,
            vec![
                json!({ "role": "user", "content": "hi" }),
                json!({ "role": "assistant", "content": "hello" }),
                json!({ "role": "user", "content": "next" }),
            ]
        );
    }

    #[test]
    fn build_messages_joins_multiple_system_turns() {
        let history = vec![
            ChatTurn::new(ChatRole::System, "rule 1"),
            ChatTurn::new(ChatRole::System, "rule 2"),
        ];
        let (system, _messages) = build_messages(&history, "go");
        assert_eq!(system.as_deref(), Some("rule 1\n\nrule 2"));
    }

    #[tokio::test]
    async fn process_frame_emits_text_delta() {
        let (tx, mut rx) = mpsc::channel::<Delta>(4);
        let mut acc = String::new();
        let frame =
            "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"hi\"}}\n\n".to_string();
        let outcome = process_frame(&frame, &tx, &mut acc).await.unwrap();
        assert!(matches!(outcome, FrameOutcome::Continue));
        assert_eq!(acc, "hi");
        assert_eq!(rx.recv().await.unwrap().content, "hi");
    }

    #[tokio::test]
    async fn process_frame_message_stop_returns_done() {
        let (tx, _rx) = mpsc::channel::<Delta>(4);
        let mut acc = String::new();
        let frame = "event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n".to_string();
        let outcome = process_frame(&frame, &tx, &mut acc).await.unwrap();
        assert!(matches!(outcome, FrameOutcome::Done));
    }

    #[tokio::test]
    async fn process_frame_ignores_other_event_types() {
        let (tx, mut rx) = mpsc::channel::<Delta>(4);
        let mut acc = String::new();
        let frame = "event: message_start\ndata: {\"type\":\"message_start\"}\n\n".to_string();
        let outcome = process_frame(&frame, &tx, &mut acc).await.unwrap();
        assert!(matches!(outcome, FrameOutcome::Continue));
        assert!(acc.is_empty());
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn process_frame_ignores_non_text_delta() {
        let (tx, mut rx) = mpsc::channel::<Delta>(4);
        let mut acc = String::new();
        // input_json_delta or other delta types should be ignored
        let frame =
            "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"...\"}}\n\n".to_string();
        let outcome = process_frame(&frame, &tx, &mut acc).await.unwrap();
        assert!(matches!(outcome, FrameOutcome::Continue));
        assert!(acc.is_empty());
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn process_frame_returns_cancelled_when_receiver_dropped() {
        let (tx, rx) = mpsc::channel::<Delta>(1);
        drop(rx);
        let mut acc = String::new();
        let frame =
            "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"x\"}}\n\n".to_string();
        let err = process_frame(&frame, &tx, &mut acc).await.unwrap_err();
        assert!(matches!(err, ProviderError::Cancelled));
    }
}
