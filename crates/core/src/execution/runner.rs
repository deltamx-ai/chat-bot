use super::{
    ExecutionContext, InMemoryToolRouter, StepStatus, Task, TaskError, TaskResult, TaskStatus,
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
}

impl TaskRunner for SequentialTaskRunner {
    fn run(&self, mut task: Task, ctx: ExecutionContext) -> Result<TaskResult, TaskError> {
        task.transition_to(TaskStatus::Running)
            .map_err(|err| TaskError {
                code: "invalid_task_transition".into(),
                message: format!("{:?} -> {:?}", err.from, err.to),
                detail: None,
                retriable: false,
            })?;

        for step in &mut task.steps {
            if step.status == StepStatus::Pending {
                step.transition_to(StepStatus::Ready)
                    .map_err(|err| TaskError {
                        code: "invalid_step_transition".into(),
                        message: format!("{:?} -> {:?}", err.from, err.to),
                        detail: None,
                        retriable: false,
                    })?;
            }

            step.transition_to(StepStatus::Running)
                .map_err(|err| TaskError {
                    code: "invalid_step_transition".into(),
                    message: format!("{:?} -> {:?}", err.from, err.to),
                    detail: None,
                    retriable: false,
                })?;

            match self
                .router
                .call(ctx.clone(), &step.tool_name, step.input.clone())
            {
                Ok(output) => {
                    step.output = Some(output.content.clone());
                    step.transition_to(StepStatus::Succeeded)
                        .map_err(|err| TaskError {
                            code: "invalid_step_transition".into(),
                            message: format!("{:?} -> {:?}", err.from, err.to),
                            detail: None,
                            retriable: false,
                        })?;
                    task.output = Some(output.content);
                }
                Err(err) => {
                    let task_error = TaskError {
                        code: err.code,
                        message: err.message,
                        detail: None,
                        retriable: false,
                    };
                    step.error = Some(task_error.clone());
                    let _ = step.transition_to(StepStatus::Failed);
                    task.error = Some(task_error.clone());
                    let _ = task.transition_to(TaskStatus::Failed);
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

        Ok(TaskResult {
            output: task.output,
            error: None,
        })
    }
}
