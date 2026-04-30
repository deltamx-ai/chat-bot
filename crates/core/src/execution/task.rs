use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{conversation::ConversationId, planning::PlanId};

use super::{StepAction, TaskError, state::TaskTransitionError, step::TaskStep};

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
    pub fn draft(
        id: impl Into<String>,
        conversation_id: ConversationId,
        kind: TaskKind,
        title: impl Into<String>,
        goal: impl Into<String>,
        input: Value,
    ) -> Self {
        Self {
            id: TaskId(id.into()),
            conversation_id,
            parent_task_id: None,
            plan_id: None,
            kind,
            title: title.into(),
            goal: goal.into(),
            status: TaskStatus::Draft,
            priority: TaskPriority::Normal,
            assignee: None,
            steps: vec![],
            input,
            output: None,
            error: None,
            retry_count: 0,
            max_retries: 0,
            tags: vec![],
            created_at: String::new(),
            updated_at: String::new(),
            started_at: None,
            finished_at: None,
        }
    }

    pub fn with_steps(mut self, steps: Vec<TaskStep>) -> Self {
        self.steps = steps;
        self
    }

    pub fn transition_to(&mut self, next: TaskStatus) -> Result<(), TaskTransitionError> {
        super::state::ensure_task_transition(&self.status, &next)?;
        self.status = next;
        Ok(())
    }
}

pub fn infer_task_kind(action: &StepAction) -> TaskKind {
    match action {
        StepAction::Read => TaskKind::Research,
        StepAction::Search => TaskKind::Research,
        StepAction::Write => TaskKind::Write,
        StepAction::Validate => TaskKind::Validate,
        StepAction::Plan => TaskKind::Feature,
        StepAction::Custom(kind) => TaskKind::Custom(kind.clone()),
    }
}
