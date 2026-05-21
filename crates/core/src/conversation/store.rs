use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use super::{Conversation, ConversationId, Message, MessageId};

#[async_trait]
pub trait ConversationStore: Send + Sync {
    async fn save_conversation(&self, conversation: Conversation) -> Result<(), String>;
    async fn load_conversation(&self, id: &ConversationId) -> Result<Option<Conversation>, String>;
    async fn list_conversations(&self) -> Result<Vec<Conversation>, String>;
    async fn update_conversation(
        &self,
        id: &ConversationId,
        title: Option<String>,
        summary: Option<Option<String>>,
    ) -> Result<Option<Conversation>, String>;
    async fn delete_conversation(&self, id: &ConversationId) -> Result<bool, String>;
    async fn append_message(&self, message: Message) -> Result<(), String>;
    async fn list_messages(&self, conversation_id: &ConversationId)
    -> Result<Vec<Message>, String>;
    async fn get_message(&self, id: &MessageId) -> Result<Option<Message>, String>;
}

#[derive(Default)]
struct InMemoryInner {
    conversations: HashMap<String, Conversation>,
    messages: HashMap<String, Vec<Message>>,
}

#[derive(Default, Clone)]
pub struct InMemoryConversationStore {
    inner: Arc<Mutex<InMemoryInner>>,
}

impl std::fmt::Debug for InMemoryConversationStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InMemoryConversationStore")
            .finish_non_exhaustive()
    }
}

impl InMemoryConversationStore {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl ConversationStore for InMemoryConversationStore {
    async fn save_conversation(&self, conversation: Conversation) -> Result<(), String> {
        let mut guard = self.inner.lock().await;
        guard
            .conversations
            .insert(conversation.id.0.clone(), conversation);
        Ok(())
    }

    async fn load_conversation(&self, id: &ConversationId) -> Result<Option<Conversation>, String> {
        let guard = self.inner.lock().await;
        Ok(guard.conversations.get(&id.0).cloned())
    }

    async fn list_conversations(&self) -> Result<Vec<Conversation>, String> {
        let guard = self.inner.lock().await;
        Ok(guard.conversations.values().cloned().collect())
    }

    async fn update_conversation(
        &self,
        id: &ConversationId,
        title: Option<String>,
        summary: Option<Option<String>>,
    ) -> Result<Option<Conversation>, String> {
        let mut guard = self.inner.lock().await;
        let Some(conversation) = guard.conversations.get_mut(&id.0) else {
            return Ok(None);
        };
        if let Some(title) = title {
            conversation.title = title;
        }
        if let Some(summary) = summary {
            conversation.summary = summary;
        }
        Ok(Some(conversation.clone()))
    }

    async fn delete_conversation(&self, id: &ConversationId) -> Result<bool, String> {
        let mut guard = self.inner.lock().await;
        let existed = guard.conversations.remove(&id.0).is_some();
        guard.messages.remove(&id.0);
        Ok(existed)
    }

    async fn append_message(&self, message: Message) -> Result<(), String> {
        let mut guard = self.inner.lock().await;
        guard
            .messages
            .entry(message.conversation_id.0.clone())
            .or_default()
            .push(message);
        Ok(())
    }

    async fn list_messages(
        &self,
        conversation_id: &ConversationId,
    ) -> Result<Vec<Message>, String> {
        let guard = self.inner.lock().await;
        Ok(guard
            .messages
            .get(&conversation_id.0)
            .cloned()
            .unwrap_or_default())
    }

    async fn get_message(&self, id: &MessageId) -> Result<Option<Message>, String> {
        let guard = self.inner.lock().await;
        for messages in guard.messages.values() {
            if let Some(message) = messages.iter().find(|message| message.id == *id) {
                return Ok(Some(message.clone()));
            }
        }
        Ok(None)
    }
}
