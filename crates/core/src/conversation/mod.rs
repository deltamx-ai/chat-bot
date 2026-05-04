//! Conversation models, lifecycle, message handling, and session-scoped options.

mod message;
mod service;
mod store;
mod types;

pub use message::{Message, MessageId, MessageRole};
pub use service::ConversationService;
pub use store::{ConversationStore, InMemoryConversationStore, MessageStore};
pub use types::{Conversation, ConversationId, ConversationStatus};
