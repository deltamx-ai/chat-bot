use chatbot_core::{
    auth::{AuthProvider, Credential, CredentialKind},
    conversation::ConversationId,
    execution::{ExecutionContext, InMemoryTaskStore, SequentialTaskRunner, TaskStore},
    planning::{PlanRequest, Planner, SimplePlanner},
    provider::copilot::CopilotAuthProvider,
};
use serde_json::json;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("task") => handle_task_command(&args[2..]),
        Some("auth") => handle_auth_command(&args[2..]),
        _ => print_health(),
    }
}

fn print_health() {
    let health = chatbot_core::health();
    println!(
        "{}",
        serde_json::to_string_pretty(&health).expect("serialize health response")
    );
}

fn handle_task_command(args: &[String]) {
    match args.first().map(String::as_str) {
        Some("list") => task_list(),
        Some("show") => {
            if let Some(task_id) = args.get(1) {
                task_show(task_id);
            } else {
                eprintln!("usage: cli task show <task-id>");
            }
        }
        Some("run") => {
            let title = args.get(1).cloned().unwrap_or_else(|| "Ad-hoc task".into());
            task_run(&title);
        }
        _ => eprintln!("usage: cli task <list|show|run>"),
    }
}

fn handle_auth_command(args: &[String]) {
    match args.first().map(String::as_str) {
        Some("copilot") => auth_copilot(),
        _ => eprintln!("usage: cli auth copilot"),
    }
}

fn auth_copilot() {
    let provider = CopilotAuthProvider::default();
    let session = provider
        .login(Credential {
            kind: CredentialKind::DeviceCode,
            value: "request-device-flow".into(),
        })
        .expect("begin copilot auth");
    println!(
        "{}",
        serde_json::to_string_pretty(&session).expect("serialize auth session")
    );
}

fn demo_store() -> InMemoryTaskStore {
    let planner = SimplePlanner;
    let plan = planner
        .create_plan(PlanRequest {
            title: "Demo task".into(),
            goal: "Show a CLI task runtime example".into(),
            input: json!({ "source": "cli-demo" }),
        })
        .expect("create demo plan");

    let tasks = plan.into_tasks(ConversationId("conv_cli_demo".into()));
    let runner = SequentialTaskRunner::default();
    let mut store = InMemoryTaskStore::new();

    for task in tasks {
        let _ = runner.run_with_store(task, ExecutionContext::default(), &mut store);
    }

    store
}

fn task_list() {
    let store = demo_store();
    let tasks = store.list_tasks().expect("list tasks");
    println!(
        "{}",
        serde_json::to_string_pretty(&tasks).expect("serialize task list")
    );
}

fn task_show(task_id: &str) {
    let store = demo_store();
    let task = store
        .load_task(&chatbot_core::execution::TaskId(task_id.into()))
        .expect("load task");
    println!(
        "{}",
        serde_json::to_string_pretty(&task).expect("serialize task detail")
    );
}

fn task_run(title: &str) {
    let planner = SimplePlanner;
    let plan = planner
        .create_plan(PlanRequest {
            title: title.into(),
            goal: format!("Run task from CLI: {title}"),
            input: json!({ "source": "cli-run", "title": title }),
        })
        .expect("create plan");

    let tasks = plan.into_tasks(ConversationId("conv_cli_run".into()));
    let runner = SequentialTaskRunner::default();
    let mut store = InMemoryTaskStore::new();

    for task in tasks {
        let result = runner.run_with_store(task, ExecutionContext::default(), &mut store);
        println!(
            "{}",
            serde_json::to_string_pretty(&result).expect("serialize run result")
        );
    }
}
