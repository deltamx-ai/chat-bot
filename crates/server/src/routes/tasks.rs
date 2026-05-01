use crate::state::AppState;

pub fn list_tasks_json(state: &AppState) -> String {
    serde_json::to_string_pretty(&state.tasks()).expect("serialize task list")
}

pub fn get_task_json(state: &AppState, task_id: &str) -> String {
    serde_json::to_string_pretty(&state.task(task_id)).expect("serialize task detail")
}

pub fn get_task_events_json(state: &AppState, task_id: &str) -> String {
    serde_json::to_string_pretty(&state.events(task_id)).expect("serialize task events")
}
