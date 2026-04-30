use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::execution::StepAction;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlanId(pub String);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanRequest {
    pub title: String,
    pub goal: String,
    pub input: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub id: PlanId,
    pub title: String,
    pub tasks: Vec<PlannedTask>,
    pub strategy: ExecutionStrategy,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlannedTask {
    pub title: String,
    pub goal: String,
    pub steps: Vec<PlannedStep>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlannedStep {
    pub title: String,
    pub action: StepAction,
    pub tool_name: String,
    pub input: Value,
    pub depends_on: Vec<StepDependency>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StepDependency(pub String);

use super::strategy::ExecutionStrategy;
