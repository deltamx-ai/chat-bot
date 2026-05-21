use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{RunId, StepId, TaskId};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ApprovalRequestId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalRequestKind {
    PlanApproval,
    ToolApproval,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalRequestStatus {
    Pending,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: ApprovalRequestId,
    pub task_id: TaskId,
    pub run_id: RunId,
    pub step_id: Option<StepId>,
    pub kind: ApprovalRequestKind,
    pub status: ApprovalRequestStatus,
    pub title: String,
    pub payload: Value,
    pub decision_note: Option<String>,
    pub created_at: String,
    pub resolved_at: Option<String>,
}
