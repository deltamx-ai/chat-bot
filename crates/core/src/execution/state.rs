use super::{StepStatus, TaskStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskTransitionError {
    pub from: TaskStatus,
    pub to: TaskStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepTransitionError {
    pub from: StepStatus,
    pub to: StepStatus,
}

pub fn ensure_task_transition(
    from: &TaskStatus,
    to: &TaskStatus,
) -> Result<(), TaskTransitionError> {
    let allowed = matches!(
        (from, to),
        (TaskStatus::Draft, TaskStatus::Pending)
            | (TaskStatus::Draft, TaskStatus::Cancelled)
            | (TaskStatus::Pending, TaskStatus::Running)
            | (TaskStatus::Pending, TaskStatus::Cancelled)
            | (TaskStatus::Running, TaskStatus::Succeeded)
            | (TaskStatus::Running, TaskStatus::Failed)
            | (TaskStatus::Running, TaskStatus::Blocked)
            | (TaskStatus::Running, TaskStatus::Cancelled)
            | (TaskStatus::Blocked, TaskStatus::Pending)
            | (TaskStatus::Blocked, TaskStatus::Cancelled)
            | (TaskStatus::Failed, TaskStatus::Pending)
            | (TaskStatus::Succeeded, TaskStatus::Archived)
            | (TaskStatus::Failed, TaskStatus::Archived)
            | (TaskStatus::Cancelled, TaskStatus::Archived)
    );

    if allowed {
        Ok(())
    } else {
        Err(TaskTransitionError {
            from: from.clone(),
            to: to.clone(),
        })
    }
}

pub fn ensure_step_transition(
    from: &StepStatus,
    to: &StepStatus,
) -> Result<(), StepTransitionError> {
    let allowed = matches!(
        (from, to),
        (StepStatus::Pending, StepStatus::Ready)
            | (StepStatus::Ready, StepStatus::Running)
            | (StepStatus::Ready, StepStatus::Skipped)
            | (StepStatus::Ready, StepStatus::Cancelled)
            | (StepStatus::Running, StepStatus::Succeeded)
            | (StepStatus::Running, StepStatus::Failed)
            | (StepStatus::Running, StepStatus::Cancelled)
            | (StepStatus::Failed, StepStatus::Ready)
    );

    if allowed {
        Ok(())
    } else {
        Err(StepTransitionError {
            from: from.clone(),
            to: to.clone(),
        })
    }
}
