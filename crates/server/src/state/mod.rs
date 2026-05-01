use core::{
    conversation::ConversationId,
    execution::{
        EventStore, InMemoryTaskStore, Task, TaskEvent, TaskEventKind, TaskId, TaskKind,
        TaskStatus, TaskStore,
    },
};
use serde_json::json;

#[derive(Debug, Default, Clone)]
pub struct AppState {
    pub store: InMemoryTaskStore,
}

impl AppState {
    pub fn demo() -> Self {
        let mut store = InMemoryTaskStore::new();
        let task = Task::draft(
            "task_demo_1",
            ConversationId("conv_demo_1".into()),
            TaskKind::Feature,
            "Demo task",
            "Create a task runtime demo payload",
            json!({"source": "server-demo"}),
        );

        let mut task = task;
        let _ = task.transition_to(TaskStatus::Pending);
        let _ = store.save_task(task.clone());

        let event = TaskEvent {
            id: store.next_event_id(&task.id),
            task_id: task.id.clone(),
            step_id: None,
            kind: TaskEventKind::TaskCreated,
            payload: json!({"status": "pending"}),
            created_at: String::new(),
        };
        let _ = store.append_event(event);

        Self { store }
    }

    pub fn task(&self, task_id: &str) -> Option<Task> {
        self.store.load_task(&TaskId(task_id.into())).ok().flatten()
    }

    pub fn tasks(&self) -> Vec<Task> {
        self.store.list_tasks().unwrap_or_default()
    }

    pub fn events(&self, task_id: &str) -> Vec<TaskEvent> {
        self.store
            .list_events(&TaskId(task_id.into()))
            .unwrap_or_default()
    }
}
