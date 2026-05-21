use std::path::Path;
use std::str::FromStr;

use async_trait::async_trait;
use chrono::Utc;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};

use super::{
    Conversation, ConversationId, ConversationStatus, ConversationStore, Message,
    MessageAttachment, MessageId, MessageRole,
};

const MIGRATION_SQL: &str = concat!(
    include_str!("../../migrations/0001_init.sql"),
    "\n",
    include_str!("../../migrations/0002_task_run_approval.sql"),
);

#[derive(Clone)]
pub struct SqliteConversationStore {
    pool: SqlitePool,
}

impl std::fmt::Debug for SqliteConversationStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SqliteConversationStore")
            .finish_non_exhaustive()
    }
}

impl SqliteConversationStore {
    pub async fn open(database_url: &str) -> Result<Self, String> {
        let options =
            SqliteConnectOptions::from_str(database_url).map_err(|err| err.to_string())?;
        let options = options.create_if_missing(true).foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(8)
            .connect_with(options)
            .await
            .map_err(|err| err.to_string())?;
        Self::migrate(&pool).await?;
        Ok(Self { pool })
    }

    pub async fn open_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|err| err.to_string())?;
        }
        let url = format!("sqlite://{}", path.display());
        Self::open(&url).await
    }

    pub async fn open_in_memory() -> Result<Self, String> {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .map_err(|err| err.to_string())?
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .map_err(|err| err.to_string())?;
        Self::migrate(&pool).await?;
        Ok(Self { pool })
    }

    async fn migrate(pool: &SqlitePool) -> Result<(), String> {
        sqlx::raw_sql(MIGRATION_SQL)
            .execute(pool)
            .await
            .map_err(|err| err.to_string())?;
        Ok(())
    }

    fn now_rfc3339() -> String {
        Utc::now().to_rfc3339()
    }

    fn status_to_str(status: &ConversationStatus) -> &'static str {
        match status {
            ConversationStatus::Active => "Active",
            ConversationStatus::Paused => "Paused",
            ConversationStatus::Archived => "Archived",
        }
    }

    fn status_from_str(raw: &str) -> ConversationStatus {
        match raw {
            "Paused" => ConversationStatus::Paused,
            "Archived" => ConversationStatus::Archived,
            _ => ConversationStatus::Active,
        }
    }

    fn role_to_str(role: &MessageRole) -> &'static str {
        match role {
            MessageRole::System => "System",
            MessageRole::User => "User",
            MessageRole::Assistant => "Assistant",
            MessageRole::Tool => "Tool",
        }
    }

    fn role_from_str(raw: &str) -> MessageRole {
        match raw {
            "System" => MessageRole::System,
            "Assistant" => MessageRole::Assistant,
            "Tool" => MessageRole::Tool,
            _ => MessageRole::User,
        }
    }

    async fn next_seq(&self, conversation_id: &str) -> Result<i64, String> {
        let row = sqlx::query(
            "SELECT COALESCE(MAX(seq), 0) + 1 AS next FROM messages WHERE conversation_id = ?",
        )
        .bind(conversation_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|err| err.to_string())?;
        row.try_get::<i64, _>("next").map_err(|err| err.to_string())
    }
}

#[async_trait]
impl ConversationStore for SqliteConversationStore {
    async fn save_conversation(&self, conversation: Conversation) -> Result<(), String> {
        let now = Self::now_rfc3339();
        let created_at = if conversation.created_at.is_empty() {
            now.clone()
        } else {
            conversation.created_at.clone()
        };
        let updated_at = if conversation.updated_at.is_empty() {
            now
        } else {
            conversation.updated_at.clone()
        };

        sqlx::query(
            "INSERT INTO conversations (id, title, summary, status, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                summary = excluded.summary,
                status = excluded.status,
                updated_at = excluded.updated_at",
        )
        .bind(&conversation.id.0)
        .bind(&conversation.title)
        .bind(conversation.summary.as_deref())
        .bind(Self::status_to_str(&conversation.status))
        .bind(&created_at)
        .bind(&updated_at)
        .execute(&self.pool)
        .await
        .map_err(|err| err.to_string())?;
        Ok(())
    }

    async fn load_conversation(&self, id: &ConversationId) -> Result<Option<Conversation>, String> {
        let row = sqlx::query(
            "SELECT id, title, summary, status, created_at, updated_at
             FROM conversations WHERE id = ?",
        )
        .bind(&id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| err.to_string())?;
        let Some(row) = row else { return Ok(None) };
        Ok(Some(Conversation {
            id: ConversationId(row.try_get::<String, _>("id").map_err(|e| e.to_string())?),
            title: row
                .try_get::<String, _>("title")
                .map_err(|e| e.to_string())?,
            summary: row
                .try_get::<Option<String>, _>("summary")
                .map_err(|e| e.to_string())?,
            status: Self::status_from_str(
                &row.try_get::<String, _>("status")
                    .map_err(|e| e.to_string())?,
            ),
            created_at: row
                .try_get::<String, _>("created_at")
                .map_err(|e| e.to_string())?,
            updated_at: row
                .try_get::<String, _>("updated_at")
                .map_err(|e| e.to_string())?,
        }))
    }

    async fn list_conversations(&self) -> Result<Vec<Conversation>, String> {
        let rows = sqlx::query(
            "SELECT id, title, summary, status, created_at, updated_at
             FROM conversations ORDER BY updated_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|err| err.to_string())?;

        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            out.push(Conversation {
                id: ConversationId(row.try_get::<String, _>("id").map_err(|e| e.to_string())?),
                title: row
                    .try_get::<String, _>("title")
                    .map_err(|e| e.to_string())?,
                summary: row
                    .try_get::<Option<String>, _>("summary")
                    .map_err(|e| e.to_string())?,
                status: Self::status_from_str(
                    &row.try_get::<String, _>("status")
                        .map_err(|e| e.to_string())?,
                ),
                created_at: row
                    .try_get::<String, _>("created_at")
                    .map_err(|e| e.to_string())?,
                updated_at: row
                    .try_get::<String, _>("updated_at")
                    .map_err(|e| e.to_string())?,
            });
        }
        Ok(out)
    }

    async fn update_conversation(
        &self,
        id: &ConversationId,
        title: Option<String>,
        summary: Option<Option<String>>,
    ) -> Result<Option<Conversation>, String> {
        let now = Self::now_rfc3339();
        let summary_set = summary.is_some();
        let summary_value = summary.flatten();
        let result = sqlx::query(
            "UPDATE conversations
             SET title = COALESCE(?, title),
                 summary = CASE WHEN ? THEN ? ELSE summary END,
                 updated_at = ?
             WHERE id = ?",
        )
        .bind(title.as_deref())
        .bind(summary_set)
        .bind(summary_value.as_deref())
        .bind(&now)
        .bind(&id.0)
        .execute(&self.pool)
        .await
        .map_err(|err| err.to_string())?;
        if result.rows_affected() == 0 {
            return Ok(None);
        }
        self.load_conversation(id).await
    }

    async fn delete_conversation(&self, id: &ConversationId) -> Result<bool, String> {
        // SQLite needs PRAGMA foreign_keys = ON for cascade; we enabled it on connect.
        // But the connection-level pragma only affects the connection that ran it,
        // and the pool may hand out different connections. Defensively delete messages
        // first, then the conversation.
        sqlx::query("DELETE FROM messages WHERE conversation_id = ?")
            .bind(&id.0)
            .execute(&self.pool)
            .await
            .map_err(|err| err.to_string())?;
        let result = sqlx::query("DELETE FROM conversations WHERE id = ?")
            .bind(&id.0)
            .execute(&self.pool)
            .await
            .map_err(|err| err.to_string())?;
        Ok(result.rows_affected() > 0)
    }

    async fn append_message(&self, message: Message) -> Result<(), String> {
        let now = Self::now_rfc3339();
        let created_at = if message.created_at.is_empty() {
            now
        } else {
            message.created_at.clone()
        };
        let attachments = serde_json::to_string(&message.attachments).map_err(|e| e.to_string())?;
        let seq = self.next_seq(&message.conversation_id.0).await?;

        sqlx::query(
            "INSERT INTO messages (id, conversation_id, role, content, model_id, attachments, created_at, seq)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                role = excluded.role,
                content = excluded.content,
                model_id = excluded.model_id,
                attachments = excluded.attachments",
        )
        .bind(&message.id.0)
        .bind(&message.conversation_id.0)
        .bind(Self::role_to_str(&message.role))
        .bind(&message.content)
        .bind(message.model_id.as_deref())
        .bind(&attachments)
        .bind(&created_at)
        .bind(seq)
        .execute(&self.pool)
        .await
        .map_err(|err| err.to_string())?;

        sqlx::query("UPDATE conversations SET updated_at = ? WHERE id = ?")
            .bind(Self::now_rfc3339())
            .bind(&message.conversation_id.0)
            .execute(&self.pool)
            .await
            .map_err(|err| err.to_string())?;
        Ok(())
    }

    async fn list_messages(
        &self,
        conversation_id: &ConversationId,
    ) -> Result<Vec<Message>, String> {
        let rows = sqlx::query(
            "SELECT id, conversation_id, role, content, model_id, attachments, created_at
             FROM messages WHERE conversation_id = ? ORDER BY seq ASC",
        )
        .bind(&conversation_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|err| err.to_string())?;

        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            let attachments_json: String = row
                .try_get::<String, _>("attachments")
                .map_err(|e| e.to_string())?;
            let attachments: Vec<MessageAttachment> =
                serde_json::from_str(&attachments_json).map_err(|e| e.to_string())?;
            out.push(Message {
                id: MessageId(row.try_get::<String, _>("id").map_err(|e| e.to_string())?),
                conversation_id: ConversationId(
                    row.try_get::<String, _>("conversation_id")
                        .map_err(|e| e.to_string())?,
                ),
                role: Self::role_from_str(
                    &row.try_get::<String, _>("role")
                        .map_err(|e| e.to_string())?,
                ),
                content: row
                    .try_get::<String, _>("content")
                    .map_err(|e| e.to_string())?,
                model_id: row
                    .try_get::<Option<String>, _>("model_id")
                    .map_err(|e| e.to_string())?,
                attachments,
                created_at: row
                    .try_get::<String, _>("created_at")
                    .map_err(|e| e.to_string())?,
            });
        }
        Ok(out)
    }

    async fn get_message(&self, id: &MessageId) -> Result<Option<Message>, String> {
        let row = sqlx::query(
            "SELECT id, conversation_id, role, content, model_id, attachments, created_at
             FROM messages WHERE id = ?",
        )
        .bind(&id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| err.to_string())?;
        let Some(row) = row else { return Ok(None) };
        let attachments_json: String = row
            .try_get::<String, _>("attachments")
            .map_err(|e| e.to_string())?;
        let attachments: Vec<MessageAttachment> =
            serde_json::from_str(&attachments_json).map_err(|e| e.to_string())?;
        Ok(Some(Message {
            id: MessageId(row.try_get::<String, _>("id").map_err(|e| e.to_string())?),
            conversation_id: ConversationId(
                row.try_get::<String, _>("conversation_id")
                    .map_err(|e| e.to_string())?,
            ),
            role: Self::role_from_str(
                &row.try_get::<String, _>("role")
                    .map_err(|e| e.to_string())?,
            ),
            content: row
                .try_get::<String, _>("content")
                .map_err(|e| e.to_string())?,
            model_id: row
                .try_get::<Option<String>, _>("model_id")
                .map_err(|e| e.to_string())?,
            attachments,
            created_at: row
                .try_get::<String, _>("created_at")
                .map_err(|e| e.to_string())?,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup() -> SqliteConversationStore {
        SqliteConversationStore::open_in_memory().await.unwrap()
    }

    #[tokio::test]
    async fn save_and_load_conversation() {
        let store = setup().await;
        let conv = Conversation {
            id: ConversationId("c1".into()),
            title: "Hello".into(),
            summary: None,
            status: ConversationStatus::Active,
            created_at: String::new(),
            updated_at: String::new(),
        };
        store.save_conversation(conv.clone()).await.unwrap();
        let loaded = store
            .load_conversation(&ConversationId("c1".into()))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(loaded.id.0, "c1");
        assert_eq!(loaded.title, "Hello");
        assert!(!loaded.created_at.is_empty());
    }

    #[tokio::test]
    async fn append_and_list_messages_preserves_order() {
        let store = setup().await;
        let conv_id = ConversationId("c1".into());
        store
            .save_conversation(Conversation {
                id: conv_id.clone(),
                title: "T".into(),
                summary: None,
                status: ConversationStatus::Active,
                created_at: String::new(),
                updated_at: String::new(),
            })
            .await
            .unwrap();
        for i in 0..5 {
            store
                .append_message(Message {
                    id: MessageId(format!("m{i}")),
                    conversation_id: conv_id.clone(),
                    role: if i % 2 == 0 {
                        MessageRole::User
                    } else {
                        MessageRole::Assistant
                    },
                    content: format!("c{i}"),
                    model_id: None,
                    attachments: Vec::new(),
                    created_at: String::new(),
                })
                .await
                .unwrap();
        }
        let listed = store.list_messages(&conv_id).await.unwrap();
        assert_eq!(listed.len(), 5);
        for (i, msg) in listed.iter().enumerate() {
            assert_eq!(msg.content, format!("c{i}"));
        }
    }

    #[tokio::test]
    async fn load_missing_conversation_returns_none() {
        let store = setup().await;
        let result = store
            .load_conversation(&ConversationId("nope".into()))
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn list_conversations_orders_by_updated_at_desc() {
        let store = setup().await;
        let a = Conversation {
            id: ConversationId("a".into()),
            title: "a".into(),
            summary: None,
            status: ConversationStatus::Active,
            created_at: "2026-01-01T00:00:00+00:00".into(),
            updated_at: "2026-01-01T00:00:00+00:00".into(),
        };
        let b = Conversation {
            id: ConversationId("b".into()),
            title: "b".into(),
            summary: None,
            status: ConversationStatus::Active,
            created_at: "2026-01-02T00:00:00+00:00".into(),
            updated_at: "2026-01-02T00:00:00+00:00".into(),
        };
        store.save_conversation(a).await.unwrap();
        store.save_conversation(b).await.unwrap();
        let list = store.list_conversations().await.unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].id.0, "b");
        assert_eq!(list[1].id.0, "a");
    }

    #[tokio::test]
    async fn attachments_round_trip_as_json() {
        let store = setup().await;
        let conv_id = ConversationId("c1".into());
        store
            .save_conversation(Conversation {
                id: conv_id.clone(),
                title: "T".into(),
                summary: None,
                status: ConversationStatus::Active,
                created_at: String::new(),
                updated_at: String::new(),
            })
            .await
            .unwrap();
        let attachment = MessageAttachment {
            id: "att1".into(),
            name: "doc.pdf".into(),
            kind: "file".into(),
            mime_type: Some("application/pdf".into()),
            path: None,
            size_bytes: Some(1024),
        };
        store
            .append_message(Message {
                id: MessageId("m1".into()),
                conversation_id: conv_id.clone(),
                role: MessageRole::User,
                content: "see file".into(),
                model_id: None,
                attachments: vec![attachment.clone()],
                created_at: String::new(),
            })
            .await
            .unwrap();
        let listed = store.list_messages(&conv_id).await.unwrap();
        assert_eq!(listed[0].attachments, vec![attachment]);
    }

    #[tokio::test]
    async fn update_conversation_renames_title() {
        let store = setup().await;
        let conv_id = ConversationId("c1".into());
        store
            .save_conversation(Conversation {
                id: conv_id.clone(),
                title: "Old".into(),
                summary: None,
                status: ConversationStatus::Active,
                created_at: String::new(),
                updated_at: String::new(),
            })
            .await
            .unwrap();
        let updated = store
            .update_conversation(&conv_id, Some("New".into()), None)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.title, "New");
        let reloaded = store.load_conversation(&conv_id).await.unwrap().unwrap();
        assert_eq!(reloaded.title, "New");
    }

    #[tokio::test]
    async fn update_conversation_returns_none_for_missing() {
        let store = setup().await;
        let result = store
            .update_conversation(&ConversationId("ghost".into()), Some("x".into()), None)
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn delete_conversation_cascades_messages() {
        let store = setup().await;
        let conv_id = ConversationId("c1".into());
        store
            .save_conversation(Conversation {
                id: conv_id.clone(),
                title: "T".into(),
                summary: None,
                status: ConversationStatus::Active,
                created_at: String::new(),
                updated_at: String::new(),
            })
            .await
            .unwrap();
        store
            .append_message(Message {
                id: MessageId("m1".into()),
                conversation_id: conv_id.clone(),
                role: MessageRole::User,
                content: "hi".into(),
                model_id: None,
                attachments: Vec::new(),
                created_at: String::new(),
            })
            .await
            .unwrap();
        let deleted = store.delete_conversation(&conv_id).await.unwrap();
        assert!(deleted);
        assert!(store.load_conversation(&conv_id).await.unwrap().is_none());
        let listed = store.list_messages(&conv_id).await.unwrap();
        assert!(listed.is_empty());
    }

    #[tokio::test]
    async fn delete_missing_conversation_returns_false() {
        let store = setup().await;
        let deleted = store
            .delete_conversation(&ConversationId("ghost".into()))
            .await
            .unwrap();
        assert!(!deleted);
    }
}
