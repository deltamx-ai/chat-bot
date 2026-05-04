use std::collections::HashMap;

use super::{Conversation, ConversationId, Message, MessageId};

pub trait ConversationStore {
    fn save_conversation(&mut self, conversation: Conversation) -> Result<(), String>;
    fn load_conversation(&self, id: &ConversationId) -> Result<Option<Conversation>, String>;
    fn list_conversations(&self) -> Result<Vec<Conversation>, String>;
}

pub trait MessageStore {
    fn append_message(&mut self, message: Message) -> Result<(), String>;
    fn list_messages(&self, conversation_id: &ConversationId) -> Result<Vec<Message>, String>;
    fn get_message(&self, id: &MessageId) -> Result<Option<Message>, String>;
}

#[derive(Debug, Default, Clone)]
pub struct InMemoryConversationStore {
    conversations: HashMap<String, Conversation>,
    messages: HashMap<String, Vec<Message>>,
}

impl InMemoryConversationStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ConversationStore for InMemoryConversationStore {
    fn save_conversation(&mut self, conversation: Conversation) -> Result<(), String> {
        self.conversations
            .insert(conversation.id.0.clone(), conversation);
        Ok(())
    }

    fn load_conversation(&self, id: &ConversationId) -> Result<Option<Conversation>, String> {
        Ok(self.conversations.get(&id.0).cloned())
    }

    fn list_conversations(&self) -> Result<Vec<Conversation>, String> {
        Ok(self.conversations.values().cloned().collect())
    }
}

impl MessageStore for InMemoryConversationStore {
    fn append_message(&mut self, message: Message) -> Result<(), String> {
        self.messages
            .entry(message.conversation_id.0.clone())
            .or_default()
            .push(message);
        Ok(())
    }

    fn list_messages(&self, conversation_id: &ConversationId) -> Result<Vec<Message>, String> {
        Ok(self
            .messages
            .get(&conversation_id.0)
            .cloned()
            .unwrap_or_default())
    }

    fn get_message(&self, id: &MessageId) -> Result<Option<Message>, String> {
        for messages in self.messages.values() {
            if let Some(message) = messages.iter().find(|message| message.id == *id) {
                return Ok(Some(message.clone()));
            }
        }
        Ok(None)
    }
}
