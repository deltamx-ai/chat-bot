//! Shared helpers for OpenAI-compatible chat completions providers.
//!
//! Both `CopilotProvider` (which speaks the OpenAI wire format) and
//! `OpenAiProvider` consume this module. New OpenAI-compatible backends
//! (Azure, OpenRouter, Groq, llama.cpp's server, etc.) should also use it.

use serde::Deserialize;
use serde_json::{Value, json};
use tokio::sync::mpsc;

use super::super::error::ProviderError;
use super::{ChatRole, ChatTurn, Delta};

#[derive(Debug)]
pub(super) enum FrameOutcome {
    Continue,
    Done,
}

#[derive(Deserialize)]
pub(super) struct ChatCompletionResponse {
    pub choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
pub(super) struct ChatChoice {
    pub message: ChatMessage,
}

#[derive(Deserialize)]
pub(super) struct ChatMessage {
    pub content: Option<String>,
}

#[derive(Deserialize)]
struct ChatStreamChunk {
    choices: Vec<ChatStreamChoice>,
}

#[derive(Deserialize)]
struct ChatStreamChoice {
    delta: ChatStreamDelta,
}

#[derive(Deserialize)]
struct ChatStreamDelta {
    #[serde(default)]
    content: Option<String>,
}

pub(super) fn build_messages(history: &[ChatTurn], new_user_prompt: &str) -> Vec<Value> {
    let mut messages = Vec::with_capacity(history.len() + 1);
    for turn in history {
        if matches!(turn.role, ChatRole::Tool) {
            continue;
        }
        if turn.content.trim().is_empty() {
            continue;
        }
        messages.push(json!({
            "role": turn.role.as_str(),
            "content": turn.content,
        }));
    }
    messages.push(json!({
        "role": "user",
        "content": new_user_prompt,
    }));
    messages
}

pub(super) fn extract_completion_content(
    payload: ChatCompletionResponse,
) -> Result<String, ProviderError> {
    payload
        .choices
        .into_iter()
        .next()
        .and_then(|choice| choice.message.content)
        .filter(|content| !content.trim().is_empty())
        .ok_or(ProviderError::EmptyOutput)
}

pub(super) async fn process_frame(
    frame: &str,
    sender: &mpsc::Sender<Delta>,
    content_acc: &mut String,
) -> Result<FrameOutcome, ProviderError> {
    let mut data_parts: Vec<&str> = Vec::new();
    for line in frame.lines() {
        if let Some(rest) = line.strip_prefix("data:") {
            data_parts.push(rest.trim_start_matches(' '));
        }
    }
    if data_parts.is_empty() {
        return Ok(FrameOutcome::Continue);
    }
    let data = data_parts.join("\n");
    let trimmed = data.trim();
    if trimmed == "[DONE]" {
        return Ok(FrameOutcome::Done);
    }
    let Ok(parsed) = serde_json::from_str::<ChatStreamChunk>(trimmed) else {
        return Ok(FrameOutcome::Continue);
    };
    let Some(content) = parsed
        .choices
        .into_iter()
        .next()
        .and_then(|choice| choice.delta.content)
    else {
        return Ok(FrameOutcome::Continue);
    };
    if content.is_empty() {
        return Ok(FrameOutcome::Continue);
    }
    content_acc.push_str(&content);
    if sender.send(Delta { content }).await.is_err() {
        return Err(ProviderError::Cancelled);
    }
    Ok(FrameOutcome::Continue)
}

#[cfg(test)]
mod tests {
    use super::{
        ChatRole, ChatTurn, Delta, FrameOutcome, ProviderError, build_messages, process_frame,
    };
    use serde_json::json;
    use tokio::sync::mpsc;

    #[test]
    fn build_messages_appends_user_turn() {
        let history = vec![
            ChatTurn::new(ChatRole::User, "hi"),
            ChatTurn::new(ChatRole::Assistant, "hello"),
        ];
        let messages = build_messages(&history, "second question");
        assert_eq!(
            messages,
            vec![
                json!({ "role": "user", "content": "hi" }),
                json!({ "role": "assistant", "content": "hello" }),
                json!({ "role": "user", "content": "second question" }),
            ]
        );
    }

    #[test]
    fn build_messages_skips_tool_and_empty() {
        let history = vec![
            ChatTurn::new(ChatRole::Tool, "tool result"),
            ChatTurn::new(ChatRole::User, "   "),
            ChatTurn::new(ChatRole::User, "real"),
        ];
        let messages = build_messages(&history, "now");
        assert_eq!(
            messages,
            vec![
                json!({ "role": "user", "content": "real" }),
                json!({ "role": "user", "content": "now" }),
            ]
        );
    }

    #[tokio::test]
    async fn process_frame_emits_delta_with_content() {
        let (tx, mut rx) = mpsc::channel::<Delta>(4);
        let mut acc = String::new();
        let frame = "data: {\"choices\":[{\"delta\":{\"content\":\"hello\"}}]}\n\n".to_string();
        let outcome = process_frame(&frame, &tx, &mut acc).await.unwrap();
        assert!(matches!(outcome, FrameOutcome::Continue));
        assert_eq!(acc, "hello");
        assert_eq!(rx.recv().await.unwrap().content, "hello");
    }

    #[tokio::test]
    async fn process_frame_done_sentinel_returns_done() {
        let (tx, _rx) = mpsc::channel::<Delta>(4);
        let mut acc = String::new();
        let frame = "data: [DONE]\n\n".to_string();
        let outcome = process_frame(&frame, &tx, &mut acc).await.unwrap();
        assert!(matches!(outcome, FrameOutcome::Done));
        assert!(acc.is_empty());
    }

    #[tokio::test]
    async fn process_frame_accumulates_multi_line_data() {
        let (tx, mut rx) = mpsc::channel::<Delta>(4);
        let mut acc = String::new();
        let line1 = "data: {\"choices\":[{\"delta\":";
        let line2 = "data: {\"content\":\"hi\"}}]}";
        let frame = format!("{line1}\n{line2}\n\n");
        let outcome = process_frame(&frame, &tx, &mut acc).await.unwrap();
        assert!(matches!(outcome, FrameOutcome::Continue));
        assert_eq!(acc, "hi");
        assert_eq!(rx.recv().await.unwrap().content, "hi");
    }

    #[tokio::test]
    async fn process_frame_skips_empty_content() {
        let (tx, mut rx) = mpsc::channel::<Delta>(4);
        let mut acc = String::new();
        let frame = "data: {\"choices\":[{\"delta\":{\"content\":\"\"}}]}\n\n".to_string();
        let outcome = process_frame(&frame, &tx, &mut acc).await.unwrap();
        assert!(matches!(outcome, FrameOutcome::Continue));
        assert!(acc.is_empty());
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn process_frame_skips_missing_content() {
        let (tx, mut rx) = mpsc::channel::<Delta>(4);
        let mut acc = String::new();
        let frame = "data: {\"choices\":[{\"delta\":{}}]}\n\n".to_string();
        let outcome = process_frame(&frame, &tx, &mut acc).await.unwrap();
        assert!(matches!(outcome, FrameOutcome::Continue));
        assert!(acc.is_empty());
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn process_frame_drops_malformed_json() {
        let (tx, mut rx) = mpsc::channel::<Delta>(4);
        let mut acc = String::new();
        let frame = "data: not-json\n\n".to_string();
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
        let frame = "data: {\"choices\":[{\"delta\":{\"content\":\"x\"}}]}\n\n".to_string();
        let err = process_frame(&frame, &tx, &mut acc).await.unwrap_err();
        assert!(matches!(err, ProviderError::Cancelled));
    }

    #[tokio::test]
    async fn process_frame_no_data_lines_continues() {
        let (tx, _rx) = mpsc::channel::<Delta>(4);
        let mut acc = String::new();
        let frame = ": keep-alive comment\n\n".to_string();
        let outcome = process_frame(&frame, &tx, &mut acc).await.unwrap();
        assert!(matches!(outcome, FrameOutcome::Continue));
        assert!(acc.is_empty());
    }
}
