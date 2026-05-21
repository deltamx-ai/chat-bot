use std::collections::HashMap;

use super::{
    ApprovalRequest, ApprovalRequestId, EventId, Run, RunId, Task, TaskError, TaskEvent,
    TaskEventKind, TaskId,
};

pub trait TaskStore {
    fn save_task(&mut self, task: Task) -> Result<(), TaskError>;
    fn load_task(&self, id: &TaskId) -> Result<Option<Task>, TaskError>;
    fn list_tasks(&self) -> Result<Vec<Task>, TaskError>;
}

pub trait EventStore {
    fn append_event(&mut self, event: TaskEvent) -> Result<(), TaskError>;
    fn list_events(&self, task_id: &TaskId) -> Result<Vec<TaskEvent>, TaskError>;
}

pub trait RunStore {
    fn save_run(&mut self, run: Run) -> Result<(), TaskError>;
    fn load_run(&self, id: &RunId) -> Result<Option<Run>, TaskError>;
    fn list_runs_by_task(&self, task_id: &TaskId) -> Result<Vec<Run>, TaskError>;
}

pub trait ApprovalStore {
    fn save_approval_request(&mut self, request: ApprovalRequest) -> Result<(), TaskError>;
    fn load_approval_request(
        &self,
        id: &ApprovalRequestId,
    ) -> Result<Option<ApprovalRequest>, TaskError>;
    fn list_approval_requests(&self) -> Result<Vec<ApprovalRequest>, TaskError>;
}

#[derive(Debug, Default, Clone)]
pub struct InMemoryTaskStore {
    tasks: HashMap<String, Task>,
    events: HashMap<String, Vec<TaskEvent>>,
    runs: HashMap<String, Run>,
    approvals: HashMap<String, ApprovalRequest>,
}

impl InMemoryTaskStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn next_event_id(&self, task_id: &TaskId) -> EventId {
        let next_index = self
            .events
            .get(&task_id.0)
            .map(|events| events.len() + 1)
            .unwrap_or(1);
        EventId(format!("evt_{}_{}", task_id.0, next_index))
    }

    pub fn push_task_event(
        &mut self,
        task_id: &TaskId,
        step_id: Option<super::StepId>,
        kind: TaskEventKind,
        payload: serde_json::Value,
    ) -> Result<(), TaskError> {
        let event = TaskEvent {
            id: self.next_event_id(task_id),
            task_id: task_id.clone(),
            step_id,
            kind,
            payload,
            created_at: String::new(),
        };
        self.append_event(event)
    }
}

impl TaskStore for InMemoryTaskStore {
    fn save_task(&mut self, task: Task) -> Result<(), TaskError> {
        self.tasks.insert(task.id.0.clone(), task);
        Ok(())
    }

    fn load_task(&self, id: &TaskId) -> Result<Option<Task>, TaskError> {
        Ok(self.tasks.get(&id.0).cloned())
    }

    fn list_tasks(&self) -> Result<Vec<Task>, TaskError> {
        Ok(self.tasks.values().cloned().collect())
    }
}

impl EventStore for InMemoryTaskStore {
    fn append_event(&mut self, event: TaskEvent) -> Result<(), TaskError> {
        self.events
            .entry(event.task_id.0.clone())
            .or_default()
            .push(event);
        Ok(())
    }

    fn list_events(&self, task_id: &TaskId) -> Result<Vec<TaskEvent>, TaskError> {
        Ok(self.events.get(&task_id.0).cloned().unwrap_or_default())
    }
}

impl RunStore for InMemoryTaskStore {
    fn save_run(&mut self, run: Run) -> Result<(), TaskError> {
        self.runs.insert(run.id.0.clone(), run);
        Ok(())
    }

    fn load_run(&self, id: &RunId) -> Result<Option<Run>, TaskError> {
        Ok(self.runs.get(&id.0).cloned())
    }

    fn list_runs_by_task(&self, task_id: &TaskId) -> Result<Vec<Run>, TaskError> {
        Ok(self
            .runs
            .values()
            .filter(|run| run.task_id == *task_id)
            .cloned()
            .collect())
    }
}

impl ApprovalStore for InMemoryTaskStore {
    fn save_approval_request(&mut self, request: ApprovalRequest) -> Result<(), TaskError> {
        self.approvals.insert(request.id.0.clone(), request);
        Ok(())
    }

    fn load_approval_request(
        &self,
        id: &ApprovalRequestId,
    ) -> Result<Option<ApprovalRequest>, TaskError> {
        Ok(self.approvals.get(&id.0).cloned())
    }

    fn list_approval_requests(&self) -> Result<Vec<ApprovalRequest>, TaskError> {
        Ok(self.approvals.values().cloned().collect())
    }
}
