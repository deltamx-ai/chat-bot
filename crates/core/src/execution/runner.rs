use serde_json::json;

use super::{
    ExecutionContext, InMemoryTaskStore, InMemoryToolRouter, StepStatus, Task, TaskError,
    TaskEventKind, TaskResult, TaskStatus, TaskStore,
};

pub trait TaskRunner {
    fn run(&self, task: Task, ctx: ExecutionContext) -> Result<TaskResult, TaskError>;
}

pub struct SequentialTaskRunner {
    router: InMemoryToolRouter,
}

impl SequentialTaskRunner {
    pub fn new(router: InMemoryToolRouter) -> Self {
        Self { router }
    }

    pub fn run_with_store(
        &self,
        mut task: Task,
        ctx: ExecutionContext,
        store: &mut InMemoryTaskStore,
    ) -> Result<TaskResult, TaskError> {
        store.save_task(task.clone())?;
        store.push_task_event(
            &task.id,
            None,
            TaskEventKind::TaskCreated,
            json!({ "status": format!("{:?}", task.status) }),
        )?;

        if task.status == TaskStatus::Draft {
            task.transition_to(TaskStatus::Pending)
                .map_err(|err| TaskError {
                    code: "invalid_task_transition".into(),
                    message: format!("{:?} -> {:?}", err.from, err.to),
                    detail: None,
                    retriable: false,
                })?;
            store.save_task(task.clone())?;
        }

        task.transition_to(TaskStatus::Running)
            .map_err(|err| TaskError {
                code: "invalid_task_transition".into(),
                message: format!("{:?} -> {:?}", err.from, err.to),
                detail: None,
                retriable: false,
            })?;
        store.save_task(task.clone())?;
        store.push_task_event(&task.id, None, TaskEventKind::TaskStarted, json!({}))?;

        for step_index in 0..task.steps.len() {
            let step_id;
            let tool_name;
            let step_input;

            {
                let step = &mut task.steps[step_index];

                if step.status == StepStatus::Pending {
                    step.transition_to(StepStatus::Ready)
                        .map_err(|err| TaskError {
                            code: "invalid_step_transition".into(),
                            message: format!("{:?} -> {:?}", err.from, err.to),
                            detail: None,
                            retriable: false,
                        })?;
                    store.push_task_event(
                        &task.id,
                        Some(step.id.clone()),
                        TaskEventKind::StepReady,
                        json!({ "tool": step.tool_name }),
                    )?;
                }

                step.transition_to(StepStatus::Running)
                    .map_err(|err| TaskError {
                        code: "invalid_step_transition".into(),
                        message: format!("{:?} -> {:?}", err.from, err.to),
                        detail: None,
                        retriable: false,
                    })?;
                store.push_task_event(
                    &task.id,
                    Some(step.id.clone()),
                    TaskEventKind::StepStarted,
                    json!({ "tool": step.tool_name }),
                )?;

                step_id = step.id.clone();
                tool_name = step.tool_name.clone();
                step_input = step.input.clone();
            }

            match self.router.call(ctx.clone(), &tool_name, step_input) {
                Ok(output) => {
                    {
                        let step = &mut task.steps[step_index];
                        step.output = Some(output.content.clone());
                        step.transition_to(StepStatus::Succeeded)
                            .map_err(|err| TaskError {
                                code: "invalid_step_transition".into(),
                                message: format!("{:?} -> {:?}", err.from, err.to),
                                detail: None,
                                retriable: false,
                            })?;
                    }
                    task.output = Some(output.content);
                    store.push_task_event(
                        &task.id,
                        Some(step_id),
                        TaskEventKind::StepSucceeded,
                        json!({ "tool": tool_name }),
                    )?;
                    store.save_task(task.clone())?;
                }
                Err(err) => {
                    let task_error = TaskError {
                        code: err.code,
                        message: err.message,
                        detail: None,
                        retriable: false,
                    };
                    {
                        let step = &mut task.steps[step_index];
                        step.error = Some(task_error.clone());
                        let _ = step.transition_to(StepStatus::Failed);
                    }
                    task.error = Some(task_error.clone());
                    let _ = task.transition_to(TaskStatus::Failed);
                    store.save_task(task.clone())?;
                    store.push_task_event(
                        &task.id,
                        Some(step_id),
                        TaskEventKind::StepFailed,
                        json!({ "error": task_error.message.clone(), "tool": tool_name }),
                    )?;
                    store.push_task_event(
                        &task.id,
                        None,
                        TaskEventKind::TaskFailed,
                        json!({ "error": task_error.message.clone() }),
                    )?;
                    return Err(task_error);
                }
            }
        }

        task.transition_to(TaskStatus::Succeeded)
            .map_err(|err| TaskError {
                code: "invalid_task_transition".into(),
                message: format!("{:?} -> {:?}", err.from, err.to),
                detail: None,
                retriable: false,
            })?;
        store.save_task(task.clone())?;
        store.push_task_event(&task.id, None, TaskEventKind::TaskSucceeded, json!({}))?;

        Ok(TaskResult {
            output: task.output,
            error: None,
        })
    }
}

impl Default for SequentialTaskRunner {
    fn default() -> Self {
        Self::new(super::registry::ToolRegistry::default_router())
    }
}

impl TaskRunner for SequentialTaskRunner {
    fn run(&self, task: Task, ctx: ExecutionContext) -> Result<TaskResult, TaskError> {
        let mut store = InMemoryTaskStore::new();
        self.run_with_store(task, ctx, &mut store)
    }
}
