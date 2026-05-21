use std::sync::Arc;

use chatbot_core::{
    conversation::{
        Conversation, ConversationId, ConversationService, ConversationStatus, ConversationStore,
        Message, MessageAttachment, MessageRole,
    },
    execution::{
        ApprovalRequest, ApprovalRequestId, ApprovalRequestKind, ApprovalRequestStatus,
        ExecutionContext, InMemoryTaskStore, InMemoryToolRouter, Run, RunId, RunStatus,
        SequentialTaskRunner, SqliteExecutionStore, StepStatus, Task, TaskEvent, TaskEventKind,
        TaskId, TaskResult, TaskStatus, Teammate, TeammateId, TeammateMessage, TeammateMessageId,
        TeammateMessageStatus, TeammateStatus,
    },
    planning::{PlanRequest, Planner, SimplePlanner},
    provider::{ChatRole, ChatTurn, ModelSelection},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::auth::{CopilotAuthService, RateLimiter};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunDto {
    pub id: String,
    pub task_id: String,
    pub conversation_id: String,
    pub status: String,
    pub current_step_index: Option<u32>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequestDto {
    pub id: String,
    pub task_id: String,
    pub run_id: String,
    pub step_id: Option<String>,
    pub kind: String,
    pub status: String,
    pub title: String,
    pub payload: serde_json::Value,
    pub decision_note: Option<String>,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApprovalDecisionRequest {
    pub decision_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeammateDto {
    pub id: String,
    pub name: String,
    pub role: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeammateMessageDto {
    pub id: String,
    pub teammate_id: String,
    pub from_name: String,
    pub content: String,
    pub status: String,
    pub created_at: String,
    pub read_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTeammateRequest {
    pub name: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendTeammateMessageRequest {
    pub from_name: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct PreparedMessage {
    pub conversation: Conversation,
    pub user_message: Message,
    pub conversation_id: String,
    pub assistant_message_id: String,
    pub selection: ModelSelection,
    pub content: String,
    pub attachments: Vec<MessageAttachment>,
    pub history: Vec<ChatTurn>,
}

const HISTORY_MAX_TURNS: usize = 30;

fn history_from_messages(messages: &[Message], cap: usize) -> Vec<ChatTurn> {
    let start = messages.len().saturating_sub(cap);
    messages[start..]
        .iter()
        .filter_map(|message| {
            let role = match message.role {
                MessageRole::System => ChatRole::System,
                MessageRole::User => ChatRole::User,
                MessageRole::Assistant => ChatRole::Assistant,
                MessageRole::Tool => return None,
            };
            if message.content.trim().is_empty() {
                return None;
            }
            Some(ChatTurn::new(role, message.content.clone()))
        })
        .collect()
}

#[derive(Clone)]
pub struct AppState {
    pub runtime_store: InMemoryTaskStore,
    pub durable_store: SqliteExecutionStore,
    pub conversations: Arc<dyn ConversationStore + Send + Sync>,
    pub copilot_auth: Option<Arc<CopilotAuthService>>,
    pub copilot_limiter: Arc<RateLimiter>,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("runtime_store", &self.runtime_store)
            .field("durable_store", &self.durable_store)
            .field("copilot_auth", &self.copilot_auth.is_some())
            .finish_non_exhaustive()
    }
}

impl AppState {
    fn default_router() -> InMemoryToolRouter {
        chatbot_core::execution::ToolRegistry::default_router()
    }

    async fn append_durable_event(
        &self,
        run_id: &RunId,
        task_id: &TaskId,
        step_id: Option<chatbot_core::execution::StepId>,
        kind: TaskEventKind,
        payload: serde_json::Value,
    ) -> Result<(), String> {
        self.durable_store
            .append_event(
                TaskEvent {
                    id: chatbot_core::execution::EventId(format!(
                        "evt_{}_{}",
                        task_id.0,
                        short_random_suffix()
                    )),
                    task_id: task_id.clone(),
                    step_id,
                    kind,
                    payload,
                    created_at: chrono::Utc::now().to_rfc3339(),
                },
                Some(run_id),
            )
            .await
    }

    async fn execute_run_steps(&mut self, run: &mut Run, task: &mut Task) -> Result<(), String> {
        let router = Self::default_router();
        let ctx = ExecutionContext {
            conversation_id: Some(task.conversation_id.clone()),
            ..ExecutionContext::default()
        };

        for step_index in 0..task.steps.len() {
            let step = &task.steps[step_index];
            if matches!(
                step.status,
                StepStatus::Succeeded | StepStatus::Skipped | StepStatus::Cancelled
            ) {
                continue;
            }

            if task.steps[step_index].status == StepStatus::Pending {
                task.steps[step_index].status = StepStatus::Ready;
                self.append_durable_event(
                    &run.id,
                    &task.id,
                    Some(task.steps[step_index].id.clone()),
                    TaskEventKind::StepReady,
                    json!({ "event": "step.ready", "tool": task.steps[step_index].tool_name }),
                )
                .await?;
            }

            task.steps[step_index].status = StepStatus::Running;
            run.current_step_index = Some(step_index as u32);
            self.durable_store.save_run(run.clone()).await?;
            self.durable_store.save_task(task.clone()).await?;
            self.append_durable_event(
                &run.id,
                &task.id,
                Some(task.steps[step_index].id.clone()),
                TaskEventKind::StepStarted,
                json!({ "event": "step.started", "tool": task.steps[step_index].tool_name }),
            )
            .await?;

            let step_input = task.steps[step_index].input.clone();
            let tool_name = task.steps[step_index].tool_name.clone();
            let step_id = task.steps[step_index].id.clone();

            match router.call(ctx.clone(), &tool_name, step_input) {
                Ok(output) => {
                    task.steps[step_index].output = Some(output.content.clone());
                    task.steps[step_index].status = StepStatus::Succeeded;
                    task.output = Some(output.content);
                    self.durable_store.save_task(task.clone()).await?;
                    self.append_durable_event(
                        &run.id,
                        &task.id,
                        Some(step_id),
                        TaskEventKind::StepSucceeded,
                        json!({ "event": "step.succeeded", "tool": tool_name }),
                    )
                    .await?;
                }
                Err(err) => {
                    let task_error = chatbot_core::execution::TaskError {
                        code: err.code,
                        message: err.message,
                        detail: None,
                        retriable: false,
                    };
                    task.steps[step_index].error = Some(task_error.clone());
                    task.steps[step_index].status = StepStatus::Failed;
                    task.error = Some(task_error.clone());
                    task.status = TaskStatus::Failed;
                    run.status = RunStatus::Failed;
                    run.error = Some(task_error.clone());
                    run.finished_at = Some(chrono::Utc::now().to_rfc3339());
                    self.durable_store.save_task(task.clone()).await?;
                    self.durable_store.save_run(run.clone()).await?;
                    self.append_durable_event(
                        &run.id,
                        &task.id,
                        Some(step_id),
                        TaskEventKind::StepFailed,
                        json!({ "event": "step.failed", "tool": tool_name, "error": task_error.message }),
                    )
                    .await?;
                    self.append_durable_event(
                        &run.id,
                        &task.id,
                        None,
                        TaskEventKind::TaskFailed,
                        json!({ "event": "run.failed", "error": task.error.as_ref().map(|e| e.message.clone()) }),
                    )
                    .await?;
                    return Err(task
                        .error
                        .as_ref()
                        .map(|e| e.message.clone())
                        .unwrap_or_else(|| "step failed".into()));
                }
            }
        }

        task.status = TaskStatus::Succeeded;
        task.finished_at = Some(chrono::Utc::now().to_rfc3339());
        run.status = RunStatus::Succeeded;
        run.finished_at = Some(chrono::Utc::now().to_rfc3339());
        self.durable_store.save_task(task.clone()).await?;
        self.durable_store.save_run(run.clone()).await?;
        self.append_durable_event(
            &run.id,
            &task.id,
            None,
            TaskEventKind::TaskSucceeded,
            json!({ "event": "run.succeeded" }),
        )
        .await?;
        Ok(())
    }

    pub fn new(
        conversations: Arc<dyn ConversationStore + Send + Sync>,
        durable_store: SqliteExecutionStore,
    ) -> Self {
        Self {
            runtime_store: InMemoryTaskStore::new(),
            durable_store,
            conversations,
            copilot_auth: None,
            copilot_limiter: Arc::new(RateLimiter::default()),
        }
    }

    pub fn with_copilot_auth(mut self, service: Arc<CopilotAuthService>) -> Self {
        self.copilot_auth = Some(service);
        self
    }

    pub fn copilot_auth(&self) -> Option<Arc<CopilotAuthService>> {
        self.copilot_auth.clone()
    }

    pub fn conversations(&self) -> Arc<dyn ConversationStore + Send + Sync> {
        Arc::clone(&self.conversations)
    }

    pub async fn task(&self, task_id: &str) -> Option<Task> {
        self.durable_store
            .load_task(&TaskId(task_id.into()))
            .await
            .ok()
            .flatten()
    }

    pub async fn tasks(&self) -> Vec<Task> {
        self.durable_store.list_tasks().await.unwrap_or_default()
    }

    pub async fn events(&self, task_id: &str) -> Vec<TaskEvent> {
        self.durable_store
            .list_events(&TaskId(task_id.into()))
            .await
            .unwrap_or_default()
    }

    pub async fn runs_by_task(&self, task_id: &str) -> Vec<Run> {
        self.durable_store
            .list_runs_by_task(&TaskId(task_id.into()))
            .await
            .unwrap_or_default()
    }

    pub async fn approval_requests(&self) -> Vec<ApprovalRequest> {
        self.durable_store
            .list_approval_requests()
            .await
            .unwrap_or_default()
    }

    pub async fn teammates(&self) -> Vec<Teammate> {
        self.durable_store
            .list_teammates()
            .await
            .unwrap_or_default()
    }

    pub async fn teammate_messages(&self, teammate_id: &str) -> Vec<TeammateMessage> {
        self.durable_store
            .list_teammate_messages(&TeammateId(teammate_id.into()))
            .await
            .unwrap_or_default()
    }

    pub async fn create_teammate(&self, name: &str, role: &str) -> Result<Teammate, String> {
        let now = chrono::Utc::now().to_rfc3339();
        let teammate = Teammate {
            id: TeammateId(format!("tm_{}_{}", name, short_random_suffix())),
            name: name.into(),
            role: role.into(),
            status: TeammateStatus::Idle,
            created_at: now.clone(),
            updated_at: now,
        };
        self.durable_store.save_teammate(teammate.clone()).await?;
        Ok(teammate)
    }

    pub async fn send_teammate_message(
        &self,
        teammate_id: &str,
        from_name: &str,
        content: &str,
    ) -> Result<TeammateMessage, String> {
        let message = TeammateMessage {
            id: TeammateMessageId(format!("msg_tm_{}_{}", teammate_id, short_random_suffix())),
            teammate_id: TeammateId(teammate_id.into()),
            from_name: from_name.into(),
            content: content.into(),
            status: TeammateMessageStatus::Unread,
            created_at: chrono::Utc::now().to_rfc3339(),
            read_at: None,
        };
        self.durable_store
            .save_teammate_message(message.clone())
            .await?;
        Ok(message)
    }

    pub async fn read_teammate_inbox(
        &self,
        teammate_id: &str,
    ) -> Result<Vec<TeammateMessage>, String> {
        let teammate_id = TeammateId(teammate_id.into());
        let messages = self
            .durable_store
            .list_teammate_messages(&teammate_id)
            .await?;
        self.durable_store
            .mark_teammate_messages_read(&teammate_id)
            .await?;
        Ok(messages)
    }

    pub async fn create_run_for_task(&mut self, task_id: &str) -> Result<Run, String> {
        let now = chrono::Utc::now().to_rfc3339();
        let Some(mut task) = self
            .durable_store
            .load_task(&TaskId(task_id.into()))
            .await?
        else {
            return Err("task not found".into());
        };
        task.status = TaskStatus::Blocked;
        task.started_at = Some(now.clone());
        task.updated_at = now.clone();
        self.durable_store.save_task(task.clone()).await?;

        let run = Run {
            id: RunId(format!("run_{}_{}", task.id.0, short_random_suffix())),
            task_id: task.id.clone(),
            conversation_id: task.conversation_id.clone(),
            status: RunStatus::WaitingApproval,
            current_step_index: Some(0),
            started_at: Some(now.clone()),
            finished_at: None,
            error: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        self.durable_store.save_run(run.clone()).await?;
        self.durable_store
            .append_event(
                TaskEvent {
                    id: chatbot_core::execution::EventId(format!(
                        "evt_{}_{}",
                        task.id.0,
                        short_random_suffix()
                    )),
                    task_id: task.id.clone(),
                    step_id: None,
                    kind: TaskEventKind::TaskCreated,
                    payload: json!({
                        "event": "run.created",
                        "run_id": run.id.0,
                        "status": format!("{:?}", run.status)
                    }),
                    created_at: now.clone(),
                },
                Some(&run.id),
            )
            .await?;

        let approval = ApprovalRequest {
            id: ApprovalRequestId(format!("apr_{}_{}", task.id.0, short_random_suffix())),
            task_id: task.id.clone(),
            run_id: run.id.clone(),
            step_id: None,
            kind: ApprovalRequestKind::PlanApproval,
            status: ApprovalRequestStatus::Pending,
            title: format!("审批任务计划：{}", task.title),
            payload: json!({
                "task_id": task.id.0,
                "task_title": task.title,
                "goal": task.goal,
                "summary": "Run created and waiting for approval before execution"
            }),
            decision_note: None,
            created_at: now.clone(),
            resolved_at: None,
        };
        self.durable_store
            .save_approval_request(approval.clone())
            .await?;
        self.durable_store
            .append_event(
                TaskEvent {
                    id: chatbot_core::execution::EventId(format!(
                        "evt_{}_{}",
                        task.id.0,
                        short_random_suffix()
                    )),
                    task_id: task.id.clone(),
                    step_id: None,
                    kind: TaskEventKind::TaskBlocked,
                    payload: json!({
                        "event": "approval.requested",
                        "approval_id": approval.id.0,
                        "approval_kind": format!("{:?}", approval.kind),
                        "reason": "waiting_approval",
                        "run_id": run.id.0
                    }),
                    created_at: now,
                },
                Some(&run.id),
            )
            .await?;
        Ok(run)
    }

    pub async fn resolve_approval_request(
        &mut self,
        approval_id: &str,
        approve: bool,
        decision_note: Option<String>,
    ) -> Result<ApprovalRequest, String> {
        let Some(mut approval) = self
            .durable_store
            .load_approval_request(&ApprovalRequestId(approval_id.into()))
            .await?
        else {
            return Err("approval request not found".into());
        };
        if approval.status != ApprovalRequestStatus::Pending {
            return Err("approval request already resolved".into());
        }

        let now = chrono::Utc::now().to_rfc3339();
        approval.status = if approve {
            ApprovalRequestStatus::Approved
        } else {
            ApprovalRequestStatus::Rejected
        };
        approval.decision_note = decision_note.clone();
        approval.resolved_at = Some(now.clone());
        self.durable_store
            .save_approval_request(approval.clone())
            .await?;

        let Some(mut run) = self.durable_store.load_run(&approval.run_id).await? else {
            return Err("run not found".into());
        };
        run.status = if approve {
            RunStatus::Running
        } else {
            RunStatus::Failed
        };
        run.updated_at = now.clone();
        if !approve {
            run.finished_at = Some(now.clone());
        }
        self.durable_store.save_run(run.clone()).await?;

        let Some(mut task) = self.durable_store.load_task(&approval.task_id).await? else {
            return Err("task not found".into());
        };
        task.status = if approve {
            TaskStatus::Running
        } else {
            TaskStatus::Failed
        };
        task.updated_at = now.clone();
        if !approve {
            task.finished_at = Some(now.clone());
        }
        self.durable_store.save_task(task.clone()).await?;
        self.durable_store
            .append_event(
                TaskEvent {
                    id: chatbot_core::execution::EventId(format!(
                        "evt_{}_{}",
                        task.id.0,
                        short_random_suffix()
                    )),
                    task_id: task.id.clone(),
                    step_id: None,
                    kind: if approve {
                        TaskEventKind::TaskStarted
                    } else {
                        TaskEventKind::TaskFailed
                    },
                    payload: json!({
                        "event": if approve { "approval.approved" } else { "approval.rejected" },
                        "approval_id": approval.id.0,
                        "run_id": run.id.0,
                        "decision": if approve { "approved" } else { "rejected" },
                        "note": decision_note
                    }),
                    created_at: now.clone(),
                },
                Some(&run.id),
            )
            .await?;

        if approve {
            self.append_durable_event(
                &run.id,
                &task.id,
                None,
                TaskEventKind::TaskStarted,
                json!({ "event": "run.resumed", "run_id": run.id.0 }),
            )
            .await?;
            self.execute_run_steps(&mut run, &mut task).await?;
        }

        Ok(approval)
    }

    pub fn models(&self) -> Vec<ModelConfig> {
        vec![
            ModelConfig {
                id: "claude-sonnet-4".into(),
                label: "Claude Sonnet 4".into(),
                provider: "anthropic".into(),
            },
            ModelConfig {
                id: "claude-opus-4".into(),
                label: "Claude Opus 4".into(),
                provider: "anthropic".into(),
            },
            ModelConfig {
                id: "claude-haiku-4-5".into(),
                label: "Claude Haiku 4.5".into(),
                provider: "anthropic".into(),
            },
            ModelConfig {
                id: "openai:gpt-4o".into(),
                label: "OpenAI · GPT-4o".into(),
                provider: "openai".into(),
            },
            ModelConfig {
                id: "openai:gpt-4o-mini".into(),
                label: "OpenAI · GPT-4o mini".into(),
                provider: "openai".into(),
            },
            ModelConfig {
                id: "openai:o3-mini".into(),
                label: "OpenAI · o3-mini".into(),
                provider: "openai".into(),
            },
            ModelConfig {
                id: "copilot:gpt-4o".into(),
                label: "Copilot · GPT-4o".into(),
                provider: "copilot".into(),
            },
            ModelConfig {
                id: "copilot:gpt-4o-mini".into(),
                label: "Copilot · GPT-4o mini".into(),
                provider: "copilot".into(),
            },
            ModelConfig {
                id: "copilot:claude-3.5-sonnet".into(),
                label: "Copilot · Claude 3.5 Sonnet".into(),
                provider: "copilot".into(),
            },
            ModelConfig {
                id: "copilot:claude-3.7-sonnet".into(),
                label: "Copilot · Claude 3.7 Sonnet".into(),
                provider: "copilot".into(),
            },
            ModelConfig {
                id: "copilot:o3-mini".into(),
                label: "Copilot · o3-mini".into(),
                provider: "copilot".into(),
            },
        ]
    }

    pub async fn run_task(&mut self, title: &str, goal: &str) -> Result<Vec<TaskResult>, String> {
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
                .run_with_store(
                    task.clone(),
                    ExecutionContext::default(),
                    &mut self.runtime_store,
                )
                .map_err(|err| err.message)?;
            self.durable_store.save_task(task).await?;
            results.push(result);
        }

        Ok(results)
    }
}

pub async fn list_conversations(
    storage: &(dyn ConversationStore + Send + Sync),
) -> Vec<Conversation> {
    storage.list_conversations().await.unwrap_or_default()
}

pub async fn list_messages(
    storage: &(dyn ConversationStore + Send + Sync),
    conversation_id: &str,
) -> Vec<Message> {
    storage
        .list_messages(&ConversationId(conversation_id.into()))
        .await
        .unwrap_or_default()
}

pub async fn create_conversation(
    storage: &(dyn ConversationStore + Send + Sync),
    title: Option<String>,
) -> Result<Conversation, String> {
    let id = format!(
        "conv_{}_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0),
        short_random_suffix()
    );
    let conversation = Conversation {
        id: ConversationId(id),
        title: title.unwrap_or_else(|| "新会话".into()),
        summary: None,
        status: ConversationStatus::Active,
        created_at: String::new(),
        updated_at: String::new(),
    };
    storage.save_conversation(conversation.clone()).await?;
    Ok(conversation)
}

pub async fn rename_conversation(
    storage: &(dyn ConversationStore + Send + Sync),
    id: &str,
    title: String,
) -> Result<Option<Conversation>, String> {
    storage
        .update_conversation(&ConversationId(id.into()), Some(title), None)
        .await
}

pub async fn delete_conversation(
    storage: &(dyn ConversationStore + Send + Sync),
    id: &str,
) -> Result<bool, String> {
    storage
        .delete_conversation(&ConversationId(id.into()))
        .await
}

pub async fn prepare_send_message(
    storage: &(dyn ConversationStore + Send + Sync),
    request: SendMessageRequest,
) -> Result<PreparedMessage, String> {
    let conversation_id = request
        .conversation_id
        .clone()
        .unwrap_or_else(|| "conv_default".into());
    let model_id = request.model_id.clone();
    let attachments = request.attachments.clone();

    let conversation = if let Some(existing) = storage
        .load_conversation(&ConversationId(conversation_id.clone()))
        .await?
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
        storage.save_conversation(conversation.clone()).await?;
        conversation
    };

    let prior_messages = list_messages(storage, &conversation_id).await;
    let history = history_from_messages(&prior_messages, HISTORY_MAX_TURNS);
    let current_message_count = prior_messages.len();

    let user_message = ConversationService::append_user_message(
        storage,
        ConversationId(conversation_id.clone()),
        format!("msg_user_{}", current_message_count + 1),
        request.content.clone(),
        model_id.clone(),
        attachments.clone(),
    )
    .await?;

    let selection =
        ModelSelection::new(model_id.unwrap_or_default()).map_err(|err| err.to_string())?;

    Ok(PreparedMessage {
        conversation,
        user_message,
        conversation_id,
        assistant_message_id: format!("msg_assistant_{}", current_message_count + 2),
        selection,
        content: request.content,
        attachments,
        history,
    })
}

pub async fn finalize_send_message(
    storage: &(dyn ConversationStore + Send + Sync),
    prepared: PreparedMessage,
    assistant_content: String,
) -> Result<SendMessageResponse, String> {
    let assistant_message = ConversationService::append_assistant_message(
        storage,
        ConversationId(prepared.conversation_id),
        prepared.assistant_message_id,
        assistant_content,
        Some(prepared.selection.model_id),
        prepared.attachments,
    )
    .await?;

    Ok(SendMessageResponse {
        conversation: prepared.conversation,
        user_message: prepared.user_message,
        assistant_message,
    })
}

pub async fn list_approval_requests(state: &AppState) -> Vec<ApprovalRequestDto> {
    state
        .approval_requests()
        .await
        .into_iter()
        .map(approval_to_dto)
        .collect()
}

fn approval_to_dto(approval: ApprovalRequest) -> ApprovalRequestDto {
    ApprovalRequestDto {
        id: approval.id.0,
        task_id: approval.task_id.0,
        run_id: approval.run_id.0,
        step_id: approval.step_id.map(|step| step.0),
        kind: format!("{:?}", approval.kind),
        status: format!("{:?}", approval.status),
        title: approval.title,
        payload: approval.payload,
        decision_note: approval.decision_note,
        created_at: approval.created_at,
        resolved_at: approval.resolved_at,
    }
}

pub fn run_to_dto(run: Run) -> RunDto {
    RunDto {
        id: run.id.0,
        task_id: run.task_id.0,
        conversation_id: run.conversation_id.0,
        status: format!("{:?}", run.status),
        current_step_index: run.current_step_index,
        started_at: run.started_at,
        finished_at: run.finished_at,
    }
}

pub async fn list_teammates(state: &AppState) -> Vec<TeammateDto> {
    state
        .teammates()
        .await
        .into_iter()
        .map(teammate_to_dto)
        .collect()
}

pub fn teammate_to_dto(teammate: Teammate) -> TeammateDto {
    TeammateDto {
        id: teammate.id.0,
        name: teammate.name,
        role: teammate.role,
        status: format!("{:?}", teammate.status),
        created_at: teammate.created_at,
        updated_at: teammate.updated_at,
    }
}

pub fn teammate_message_to_dto(message: TeammateMessage) -> TeammateMessageDto {
    TeammateMessageDto {
        id: message.id.0,
        teammate_id: message.teammate_id.0,
        from_name: message.from_name,
        content: message.content,
        status: format!("{:?}", message.status),
        created_at: message.created_at,
        read_at: message.read_at,
    }
}

fn short_random_suffix() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    format!("{nanos:06x}")
}

#[cfg(test)]
mod tests {
    use super::{AppState, HISTORY_MAX_TURNS, history_from_messages};
    use chatbot_core::conversation::{
        ConversationId, InMemoryConversationStore, Message, MessageAttachment, MessageId,
        MessageRole,
    };
    use chatbot_core::execution::{
        SqliteExecutionStore, StepStatus, Task, TaskKind, TaskPriority, TaskStatus,
    };
    use chatbot_core::provider::ChatRole;

    fn make_message(role: MessageRole, id: &str, content: &str) -> Message {
        Message {
            id: MessageId(id.into()),
            conversation_id: ConversationId("conv".into()),
            role,
            content: content.into(),
            model_id: None,
            attachments: Vec::<MessageAttachment>::new(),
            created_at: String::new(),
        }
    }

    async fn make_state() -> AppState {
        let conversations = std::sync::Arc::new(InMemoryConversationStore::new());
        let durable_store = SqliteExecutionStore::open_in_memory().await.unwrap();
        AppState::new(conversations, durable_store)
    }

    #[test]
    fn skips_tool_and_empty_messages() {
        let messages = vec![
            make_message(MessageRole::Tool, "m1", "tool"),
            make_message(MessageRole::User, "m2", "   "),
            make_message(MessageRole::User, "m3", "real"),
        ];
        let turns = history_from_messages(&messages, HISTORY_MAX_TURNS);
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].role, ChatRole::User);
        assert_eq!(turns[0].content, "real");
    }

    #[test]
    fn caps_to_last_n_turns() {
        let messages: Vec<Message> = (0..50)
            .map(|i| {
                let role = if i % 2 == 0 {
                    MessageRole::User
                } else {
                    MessageRole::Assistant
                };
                make_message(role, &format!("m{i}"), &format!("c{i}"))
            })
            .collect();
        let turns = history_from_messages(&messages, 10);
        assert_eq!(turns.len(), 10);
        assert_eq!(turns[0].content, "c40");
        assert_eq!(turns[9].content, "c49");
    }

    #[test]
    fn empty_history_returns_empty() {
        let turns = history_from_messages(&[], 10);
        assert!(turns.is_empty());
    }

    #[test]
    fn preserves_role_mapping() {
        let messages = vec![
            make_message(MessageRole::System, "m1", "sys"),
            make_message(MessageRole::User, "m2", "u"),
            make_message(MessageRole::Assistant, "m3", "a"),
        ];
        let turns = history_from_messages(&messages, 10);
        assert_eq!(turns[0].role, ChatRole::System);
        assert_eq!(turns[1].role, ChatRole::User);
        assert_eq!(turns[2].role, ChatRole::Assistant);
    }

    #[tokio::test]
    async fn prepare_send_message_creates_conversation_lazily_and_returns_history() {
        let storage = InMemoryConversationStore::new();
        let req = super::SendMessageRequest {
            conversation_id: Some("c-new".into()),
            content: "hi".into(),
            model_id: Some("copilot-chat".into()),
            attachments: vec![],
        };
        let prepared = super::prepare_send_message(&storage, req).await.unwrap();
        assert_eq!(prepared.conversation_id, "c-new");
        assert!(prepared.history.is_empty());
        let listed = super::list_messages(&storage, "c-new").await;
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].content, "hi");
    }

    #[tokio::test]
    async fn prepare_send_message_collects_history_on_subsequent_turn() {
        let storage = InMemoryConversationStore::new();
        let req1 = super::SendMessageRequest {
            conversation_id: Some("c1".into()),
            content: "first".into(),
            model_id: Some("copilot-chat".into()),
            attachments: vec![],
        };
        let prepared1 = super::prepare_send_message(&storage, req1).await.unwrap();
        super::finalize_send_message(&storage, prepared1, "first reply".into())
            .await
            .unwrap();
        let req2 = super::SendMessageRequest {
            conversation_id: Some("c1".into()),
            content: "second".into(),
            model_id: Some("copilot-chat".into()),
            attachments: vec![],
        };
        let prepared2 = super::prepare_send_message(&storage, req2).await.unwrap();
        assert_eq!(prepared2.history.len(), 2);
        assert_eq!(prepared2.history[0].content, "first");
        assert_eq!(prepared2.history[1].content, "first reply");
    }

    #[tokio::test]
    async fn create_run_for_task_creates_pending_approval() {
        let mut state = make_state().await;
        let mut task = Task::draft(
            "task_test_1",
            ConversationId("conv_test".into()),
            TaskKind::Feature,
            "Test task",
            "Need approval",
            serde_json::Value::Null,
        );
        task.status = TaskStatus::Pending;
        task.priority = TaskPriority::Normal;
        state.durable_store.save_task(task).await.unwrap();

        let run = state.create_run_for_task("task_test_1").await.unwrap();
        assert_eq!(format!("{:?}", run.status), "WaitingApproval");
        let approvals = state.approval_requests().await;
        assert_eq!(approvals.len(), 1);
        assert_eq!(format!("{:?}", approvals[0].status), "Pending");
    }

    #[tokio::test]
    async fn resolve_approval_request_updates_run_and_task() {
        let mut state = make_state().await;
        let mut task = Task::draft(
            "task_test_2",
            ConversationId("conv_test".into()),
            TaskKind::Feature,
            "Test task",
            "Need approval",
            serde_json::Value::Null,
        )
        .with_steps(vec![
            chatbot_core::execution::TaskStep::pending(
                "step_test_2_1",
                chatbot_core::execution::TaskId("task_test_2".into()),
                0,
                "Inspect input",
                chatbot_core::execution::StepAction::Read,
                "read",
                serde_json::json!({ "path": "README.md" }),
            ),
            chatbot_core::execution::TaskStep::pending(
                "step_test_2_2",
                chatbot_core::execution::TaskId("task_test_2".into()),
                1,
                "Validate request",
                chatbot_core::execution::StepAction::Validate,
                "validate",
                serde_json::json!({ "target": "plan" }),
            ),
        ]);
        task.status = TaskStatus::Pending;
        task.priority = TaskPriority::Normal;
        state.durable_store.save_task(task).await.unwrap();

        state.create_run_for_task("task_test_2").await.unwrap();
        let approval_id = state.approval_requests().await[0].id.0.clone();
        let resolved = state
            .resolve_approval_request(&approval_id, true, Some("ok".into()))
            .await
            .unwrap();
        assert_eq!(format!("{:?}", resolved.status), "Approved");
        let task = state.task("task_test_2").await.unwrap();
        assert_eq!(task.status, TaskStatus::Succeeded);
        assert!(
            task.steps
                .iter()
                .all(|step| step.status == StepStatus::Succeeded)
        );
        let events = state.events("task_test_2").await;
        assert!(events.iter().any(|event| event.payload.get("event")
            == Some(&serde_json::Value::String("approval.requested".into()))));
        assert!(events.iter().any(|event| event.payload.get("event")
            == Some(&serde_json::Value::String("approval.approved".into()))));
        assert!(events.iter().any(|event| event.payload.get("event")
            == Some(&serde_json::Value::String("run.resumed".into()))));
        assert!(events.iter().any(|event| event.payload.get("event")
            == Some(&serde_json::Value::String("step.started".into()))));
        assert!(events.iter().any(|event| event.payload.get("event")
            == Some(&serde_json::Value::String("step.succeeded".into()))));
        assert!(events.iter().any(|event| event.payload.get("event")
            == Some(&serde_json::Value::String("run.succeeded".into()))));
    }
}
