use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TeammateId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TeammateStatus {
    Idle,
    Working,
    Shutdown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Teammate {
    pub id: TeammateId,
    pub name: String,
    pub role: String,
    pub status: TeammateStatus,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TeammateMessageId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TeammateMessageStatus {
    Unread,
    Read,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeammateMessage {
    pub id: TeammateMessageId,
    pub teammate_id: TeammateId,
    pub from_name: String,
    pub content: String,
    pub status: TeammateMessageStatus,
    pub created_at: String,
    pub read_at: Option<String>,
}
