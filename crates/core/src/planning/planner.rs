use super::{ExecutionPlan, PlanRequest};

pub trait Planner {
    fn create_plan(&self, request: PlanRequest) -> Result<ExecutionPlan, PlanError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanError {
    pub message: String,
}
