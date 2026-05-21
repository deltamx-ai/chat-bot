use serde::{Deserialize, Serialize};

use crate::conversation::ConversationId;

use super::{TaskError, TaskId};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RunId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunStatus {
    Pending,
    Running,
    WaitingApproval,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Run {
    pub id: RunId,
    pub task_id: TaskId,
    pub conversation_id: ConversationId,
    pub status: RunStatus,
    pub current_step_index: Option<u32>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub error: Option<TaskError>,
    pub created_at: String,
    pub updated_at: String,
}
