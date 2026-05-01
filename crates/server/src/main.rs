mod bootstrap;
mod routes;
mod state;

fn main() {
    let state = state::AppState::demo();
    println!("{}", bootstrap::bootstrap_banner());
    println!("{}", routes::health::health_json());
    println!("{}", routes::tasks::list_tasks_json(&state));
    println!("{}", routes::tasks::get_task_json(&state, "task_demo_1"));
    println!(
        "{}",
        routes::tasks::get_task_events_json(&state, "task_demo_1")
    );
}
