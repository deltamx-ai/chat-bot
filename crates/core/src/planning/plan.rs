use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    conversation::ConversationId,
    execution::{StepAction, Task, TaskStep, infer_task_kind},
};

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

impl ExecutionPlan {
    pub fn into_tasks(self, conversation_id: ConversationId) -> Vec<Task> {
        self.tasks
            .into_iter()
            .enumerate()
            .map(|(task_index, planned_task)| {
                let task_id = format!("task_{}_{}", self.id.0, task_index + 1);
                let task_kind = planned_task
                    .steps
                    .first()
                    .map(|step| infer_task_kind(&step.action))
                    .unwrap_or_else(|| crate::execution::TaskKind::Custom("empty".into()));

                let steps = planned_task
                    .steps
                    .into_iter()
                    .enumerate()
                    .map(|(step_index, planned_step)| {
                        let mut step = TaskStep::pending(
                            format!("step_{}_{}", task_id, step_index + 1),
                            crate::execution::TaskId(task_id.clone()),
                            step_index as u32,
                            planned_step.title,
                            planned_step.action,
                            planned_step.tool_name,
                            planned_step.input,
                        );
                        step.depends_on = planned_step
                            .depends_on
                            .into_iter()
                            .map(|dep| crate::execution::StepId(dep.0))
                            .collect();
                        step
                    })
                    .collect();

                Task::draft(
                    task_id,
                    conversation_id.clone(),
                    task_kind,
                    planned_task.title,
                    planned_task.goal,
                    Value::Null,
                )
                .with_steps(steps)
            })
            .collect()
    }
}

use super::strategy::ExecutionStrategy;
