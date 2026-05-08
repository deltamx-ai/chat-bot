use serde::{Deserialize, Serialize};

use super::ConversationId;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageAttachment {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub mime_type: Option<String>,
    pub path: Option<String>,
    pub size_bytes: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub conversation_id: ConversationId,
    pub role: MessageRole,
    pub content: String,
    pub model_id: Option<String>,
    pub attachments: Vec<MessageAttachment>,
    pub created_at: String,
}
