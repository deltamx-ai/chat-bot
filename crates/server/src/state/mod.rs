use core::{
    conversation::ConversationId,
    execution::{
        EventStore, ExecutionContext, InMemoryTaskStore, SequentialTaskRunner, Task, TaskEvent,
        TaskId, TaskResult, TaskStore,
    },
    planning::{PlanRequest, Planner, SimplePlanner},
};
use serde_json::json;

#[derive(Debug, Default, Clone)]
pub struct AppState {
    pub store: InMemoryTaskStore,
}

impl AppState {
    pub fn demo() -> Self {
        let mut state = Self {
            store: InMemoryTaskStore::new(),
        };
        let _ = state.run_task("Demo task", "Create a task runtime demo payload");
        state
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

    pub fn run_task(&mut self, title: &str, goal: &str) -> Result<Vec<TaskResult>, String> {
        let planner = SimplePlanner;
        let plan = planner
            .create_plan(PlanRequest {
                title: title.into(),
                goal: goal.into(),
                input: json!({ "source": "server-run", "title": title }),
            })
            .map_err(|err| err.message)?;

        let tasks = plan.into_tasks(ConversationId("conv_server_demo".into()));
        let runner = SequentialTaskRunner::default();
        let mut results = Vec::with_capacity(tasks.len());

        for task in tasks {
            let result = runner
                .run_with_store(task, ExecutionContext::default(), &mut self.store)
                .map_err(|err| err.message)?;
            results.push(result);
        }

        Ok(results)
    }
}
