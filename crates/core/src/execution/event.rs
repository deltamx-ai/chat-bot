use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{StepId, TaskId};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskEventKind {
    TaskCreated,
    TaskStarted,
    TaskBlocked,
    TaskSucceeded,
    TaskFailed,
    TaskCancelled,
    StepReady,
    StepStarted,
    StepSucceeded,
    StepFailed,
    StepSkipped,
    ArtifactProduced,
    RetryScheduled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskEvent {
    pub id: EventId,
    pub task_id: TaskId,
    pub step_id: Option<StepId>,
    pub kind: TaskEventKind,
    pub payload: Value,
    pub created_at: String,
}
