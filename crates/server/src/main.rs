mod bootstrap;
mod routes;
mod state;

fn main() {
    let mut state = state::AppState::demo();
    println!("{}", bootstrap::bootstrap_banner());
    println!("{}", routes::health::health_json());
    println!("{}", routes::tasks::list_tasks_json(&state));
    println!(
        "{}",
        routes::tasks::run_task_json(
            &mut state,
            "Server run task",
            "Run task through server skeleton"
        )
    );
    println!("{}", routes::tasks::list_tasks_json(&state));
    println!(
        "{}",
        routes::tasks::get_task_json(&state, "task_plan_default_1")
    );
    println!(
        "{}",
        routes::tasks::get_task_events_json(&state, "task_plan_default_1")
    );
}
