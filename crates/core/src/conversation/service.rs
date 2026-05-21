use super::{
    Conversation, ConversationId, ConversationStatus, ConversationStore, Message,
    MessageAttachment, MessageId, MessageRole,
};

pub struct ConversationService;

impl ConversationService {
    pub async fn create_conversation<S: ConversationStore + ?Sized>(
        store: &S,
        id: impl Into<String> + Send,
        title: impl Into<String> + Send,
    ) -> Result<Conversation, String> {
        let conversation = Conversation {
            id: ConversationId(id.into()),
            title: title.into(),
            summary: None,
            status: ConversationStatus::Active,
            created_at: String::new(),
            updated_at: String::new(),
        };
        store.save_conversation(conversation.clone()).await?;
        Ok(conversation)
    }

    pub async fn append_user_message<S: ConversationStore + ?Sized>(
        store: &S,
        conversation_id: ConversationId,
        message_id: impl Into<String> + Send,
        content: impl Into<String> + Send,
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
        store.append_message(message.clone()).await?;
        Ok(message)
    }

    pub async fn append_assistant_message<S: ConversationStore + ?Sized>(
        store: &S,
        conversation_id: ConversationId,
        message_id: impl Into<String> + Send,
        content: impl Into<String> + Send,
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
        store.append_message(message.clone()).await?;
        Ok(message)
    }
}
