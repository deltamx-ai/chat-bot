use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{TaskError, TaskId, state::StepTransitionError};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StepId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepAction {
    Read,
    Search,
    Write,
    Validate,
    Plan,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepStatus {
    Pending,
    Ready,
    Running,
    Succeeded,
    Failed,
    Skipped,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskStep {
    pub id: StepId,
    pub task_id: TaskId,
    pub index: u32,
    pub title: String,
    pub action: StepAction,
    pub tool_name: String,
    pub status: StepStatus,
    pub input: Value,
    pub output: Option<Value>,
    pub error: Option<TaskError>,
    pub depends_on: Vec<StepId>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

impl TaskStep {
    pub fn transition_to(&mut self, next: StepStatus) -> Result<(), StepTransitionError> {
        super::state::ensure_step_transition(&self.status, &next)?;
        self.status = next;
        Ok(())
    }
}
