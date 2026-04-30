use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{conversation::ConversationId, planning::PlanId};

use super::{TaskError, state::TaskTransitionError, step::TaskStep};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskKind {
    Bugfix,
    Feature,
    Research,
    Refactor,
    Validate,
    Write,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Draft,
    Pending,
    Running,
    Blocked,
    Succeeded,
    Failed,
    Cancelled,
    Archived,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Urgent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssigneeKind {
    Agent,
    Runner,
    Provider,
    User,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskAssignee {
    pub kind: AssigneeKind,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub conversation_id: ConversationId,
    pub parent_task_id: Option<TaskId>,
    pub plan_id: Option<PlanId>,
    pub kind: TaskKind,
    pub title: String,
    pub goal: String,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub assignee: Option<TaskAssignee>,
    pub steps: Vec<TaskStep>,
    pub input: Value,
    pub output: Option<Value>,
    pub error: Option<TaskError>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

impl Task {
    pub fn transition_to(&mut self, next: TaskStatus) -> Result<(), TaskTransitionError> {
        super::state::ensure_task_transition(&self.status, &next)?;
        self.status = next;
        Ok(())
    }
}
