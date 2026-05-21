//! Conversation models, lifecycle, message handling, and session-scoped options.

mod message;
mod service;
mod sqlite;
mod store;
mod types;

pub use message::{Message, MessageAttachment, MessageId, MessageRole};
pub use service::ConversationService;
pub use sqlite::SqliteConversationStore;
pub use store::{ConversationStore, InMemoryConversationStore};
pub use types::{Conversation, ConversationId, ConversationStatus};
