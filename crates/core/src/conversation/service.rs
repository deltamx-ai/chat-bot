use super::{
    Conversation, ConversationId, ConversationStatus, ConversationStore, Message,
    MessageAttachment, MessageId, MessageRole, MessageStore,
};

pub struct ConversationService;

impl ConversationService {
    pub fn create_conversation(
        store: &mut impl ConversationStore,
        id: impl Into<String>,
        title: impl Into<String>,
    ) -> Result<Conversation, String> {
        let conversation = Conversation {
            id: ConversationId(id.into()),
            title: title.into(),
            summary: None,
            status: ConversationStatus::Active,
            created_at: String::new(),
            updated_at: String::new(),
        };
        store.save_conversation(conversation.clone())?;
        Ok(conversation)
    }

    pub fn append_user_message(
        store: &mut impl MessageStore,
        conversation_id: ConversationId,
        message_id: impl Into<String>,
        content: impl Into<String>,
        model_id: Option<String>,
        attachments: Vec<MessageAttachment>,
    ) -> Result<Message, String> {
        let message = Message {
            id: MessageId(message_id.into()),
            conversation_id,
            role: MessageRole::User,
            content: content.into(),
            model_id,
            attachments,
            created_at: String::new(),
        };
        store.append_message(message.clone())?;
        Ok(message)
    }

    pub fn append_assistant_message(
        store: &mut impl MessageStore,
        conversation_id: ConversationId,
        message_id: impl Into<String>,
        content: impl Into<String>,
        model_id: Option<String>,
        attachments: Vec<MessageAttachment>,
    ) -> Result<Message, String> {
        let message = Message {
            id: MessageId(message_id.into()),
            conversation_id,
            role: MessageRole::Assistant,
            content: content.into(),
            model_id,
            attachments,
            created_at: String::new(),
        };
        store.append_message(message.clone())?;
        Ok(message)
    }
}
