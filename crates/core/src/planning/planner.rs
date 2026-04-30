use serde_json::json;

use crate::execution::StepAction;

use super::{ExecutionPlan, ExecutionStrategy, PlanId, PlanRequest, PlannedStep, PlannedTask};

pub trait Planner {
    fn create_plan(&self, request: PlanRequest) -> Result<ExecutionPlan, PlanError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanError {
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct SimplePlanner;

impl Planner for SimplePlanner {
    fn create_plan(&self, request: PlanRequest) -> Result<ExecutionPlan, PlanError> {
        Ok(ExecutionPlan {
            id: PlanId("plan_default".into()),
            title: request.title.clone(),
            strategy: ExecutionStrategy::Sequential,
            tasks: vec![PlannedTask {
                title: request.title,
                goal: request.goal,
                steps: vec![
                    PlannedStep {
                        title: "Inspect input".into(),
                        action: StepAction::Read,
                        tool_name: "read".into(),
                        input: request.input.clone(),
                        depends_on: vec![],
                    },
                    PlannedStep {
                        title: "Validate request".into(),
                        action: StepAction::Validate,
                        tool_name: "validate".into(),
                        input: json!({ "target": "plan" }),
                        depends_on: vec![],
                    },
                ],
            }],
        })
    }
}
