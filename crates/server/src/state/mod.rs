use chatbot_core::{
    auth::AuthSession,
    conversation::{
        Conversation, ConversationId, ConversationService, ConversationStatus, ConversationStore,
        InMemoryConversationStore, Message, MessageAttachment, MessageStore,
    },
    execution::{
        EventStore, ExecutionContext, InMemoryTaskStore, SequentialTaskRunner, Task, TaskEvent,
        TaskId, TaskResult, TaskStore,
    },
    planning::{PlanRequest, Planner, SimplePlanner},
    provider::{ProviderCapability, ProviderKind},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

pub type CopilotAuthState = serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelConfig {
    pub id: String,
    pub label: String,
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub conversation_id: Option<String>,
    pub content: String,
    pub model_id: Option<String>,
    pub attachments: Vec<MessageAttachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageResponse {
    pub conversation: Conversation,
    pub user_message: Message,
    pub assistant_message: Message,
}

#[derive(Debug, Default, Clone)]
pub struct AppState {
    pub store: InMemoryTaskStore,
    pub copilot_session: Option<AuthSession>,
    pub conversations: InMemoryConversationStore,
}

impl AppState {
    pub fn demo() -> Self {
        let mut state = Self {
            store: InMemoryTaskStore::new(),
            copilot_session: None,
            conversations: InMemoryConversationStore::new(),
        };
        let _ = state.run_task("Demo task", "Create a task runtime demo payload");
        let _ = state.send_message(SendMessageRequest {
            conversation_id: Some("conv_default".into()),
            content: "请同步 skills 目录。".into(),
            model_id: Some("gpt-5.5".into()),
            attachments: vec![],
        });
        state
    }

    pub fn task(&self, task_id: &str) -> Option<Task> {
        self.store.load_task(&TaskId(task_id.into())).ok().flatten()
    }

    pub fn tasks(&self) -> Vec<Task> {
        self.store.list_tasks().unwrap_or_default()
    }

    pub fn events(&self, task_id: &str) -> Vec<TaskEvent> {
        self.store
            .list_events(&TaskId(task_id.into()))
            .unwrap_or_default()
    }

    pub fn models(&self) -> Vec<ModelConfig> {
        vec![
            ModelConfig {
                id: "gpt-5.5".into(),
                label: "GPT-5.5".into(),
                provider: "openai".into(),
            },
            ModelConfig {
                id: "claude-sonnet-4".into(),
                label: "Claude Sonnet 4".into(),
                provider: "anthropic".into(),
            },
            ModelConfig {
                id: "copilot-chat".into(),
                label: "GitHub Copilot Chat".into(),
                provider: "copilot".into(),
            },
        ]
    }

    pub fn list_conversations(&self) -> Vec<Conversation> {
        self.conversations.list_conversations().unwrap_or_default()
    }

    pub fn list_messages(&self, conversation_id: &str) -> Vec<Message> {
        self.conversations
            .list_messages(&ConversationId(conversation_id.into()))
            .unwrap_or_default()
    }

    pub fn run_task(&mut self, title: &str, goal: &str) -> Result<Vec<TaskResult>, String> {
        let planner = SimplePlanner;
        let plan = planner
            .create_plan(PlanRequest {
                title: title.into(),
                goal: goal.into(),
                input: json!({ "source": "server-run", "title": title }),
            })
            .map_err(|err| err.message)?;

        let tasks = plan.into_tasks(ConversationId("conv_server_demo".into()));
        let runner = SequentialTaskRunner::default();
        let mut results = Vec::with_capacity(tasks.len());

        for task in tasks {
            let result = runner
                .run_with_store(task, ExecutionContext::default(), &mut self.store)
                .map_err(|err| err.message)?;
            results.push(result);
        }

        Ok(results)
    }

    pub fn copilot_auth_state(&self) -> CopilotAuthState {
        json!({
          "provider": {
            "id": "copilot-github",
            "kind": ProviderKind::Copilot,
            "enabled": true,
            "base_url": "https://github.com/login/device",
            "capabilities": [ProviderCapability::Authentication, ProviderCapability::Chat]
          },
          "session": self.copilot_session
        })
    }

    pub fn apply_copilot_session(&mut self, session: AuthSession) {
        self.copilot_session = Some(session);
    }

    pub fn send_message(
        &mut self,
        request: SendMessageRequest,
    ) -> Result<SendMessageResponse, String> {
        let conversation_id = request
            .conversation_id
            .clone()
            .unwrap_or_else(|| "conv_default".into());
        let model_id = request.model_id.clone();
        let attachments = request.attachments.clone();

        let conversation = if let Some(existing) = self
            .conversations
            .load_conversation(&ConversationId(conversation_id.clone()))?
        {
            existing
        } else {
            let conversation = Conversation {
                id: ConversationId(conversation_id.clone()),
                title: "默认会话".into(),
                summary: None,
                status: ConversationStatus::Active,
                created_at: String::new(),
                updated_at: String::new(),
            };
            self.conversations.save_conversation(conversation.clone())?;
            conversation
        };

        let current_message_count = self.list_messages(&conversation_id).len();

        let user_message = ConversationService::append_user_message(
            &mut self.conversations,
            ConversationId(conversation_id.clone()),
            format!("msg_user_{}", current_message_count + 1),
            request.content.clone(),
            model_id.clone(),
            attachments.clone(),
        )?;

        let attachment_summary = if attachments.is_empty() {
            String::new()
        } else {
            format!(
                "，收到 {} 个附件：{}",
                attachments.len(),
                attachments
                    .iter()
                    .map(|item| item.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };

        let assistant_message = ConversationService::append_assistant_message(
            &mut self.conversations,
            ConversationId(conversation_id),
            format!("msg_assistant_{}", current_message_count + 2),
            format!(
                "已收到消息，当前模型：{}{}。",
                model_id.clone().unwrap_or_else(|| "未指定".into()),
                attachment_summary
            ),
            model_id,
            attachments,
        )?;

        Ok(SendMessageResponse {
            conversation,
            user_message,
            assistant_message,
        })
    }
}
