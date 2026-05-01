use serde_json::json;

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

pub fn run_task_json(state: &mut AppState, title: &str, goal: &str) -> String {
    let result = state.run_task(title, goal);
    serde_json::to_string_pretty(&match result {
        Ok(results) => json!({ "ok": true, "results": results }),
        Err(message) => json!({ "ok": false, "error": message }),
    })
    .expect("serialize task run result")
}
