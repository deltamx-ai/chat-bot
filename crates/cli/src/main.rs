use arboard::Clipboard;
use chatbot_core::{
    conversation::{ConversationId, MessageAttachment},
    execution::{ExecutionContext, InMemoryTaskStore, SequentialTaskRunner, TaskStore},
    planning::{PlanRequest, Planner, SimplePlanner},
    provider::copilot::CopilotAuthClient,
};
use serde_json::json;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("message") => message_command(&args[2..]).await,
        Some("auth") => auth_command(&args[2..]).await,
        _ => println!("usage: cli <message|auth> ..."),
    }
}

async fn auth_command(args: &[String]) {
    match args.first().map(String::as_str) {
        Some("copilot") => auth_copilot().await,
        _ => eprintln!("usage: cli auth copilot"),
    }
}

async fn auth_copilot() {
    let client = match CopilotAuthClient::new() {
        Ok(client) => client,
        Err(error) => {
            eprintln!("auth failed: {error}");
            std::process::exit(1);
        }
    };
    match client.request_device_code().await {
        Ok(grant) => {
            if let Ok(mut clipboard) = Clipboard::new() {
                let _ = clipboard.set_text(grant.challenge.user_code.clone());
            }
            let _ = open::that(grant.challenge.verification_uri.clone());
            println!("Opened browser: {}", grant.challenge.verification_uri);
            println!("Copied code: {}", grant.challenge.user_code);
            println!(
                "{}",
                serde_json::to_string_pretty(&grant.challenge).expect("serialize challenge")
            );
        }
        Err(error) => {
            eprintln!("auth failed: {error}");
            std::process::exit(1);
        }
    }
}

async fn message_command(args: &[String]) {
    if args.is_empty() {
        eprintln!("usage: cli message <content> [--model MODEL] [--file PATH]...");
        return;
    }

    let content = args[0].clone();
    let mut model_id: Option<String> = None;
    let mut files: Vec<String> = Vec::new();
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--model" => {
                if let Some(value) = args.get(index + 1) {
                    model_id = Some(value.clone());
                    index += 2;
                } else {
                    eprintln!("--model needs a value");
                    return;
                }
            }
            "--file" => {
                if let Some(value) = args.get(index + 1) {
                    files.push(value.clone());
                    index += 2;
                } else {
                    eprintln!("--file needs a path");
                    return;
                }
            }
            _ => {
                index += 1;
            }
        }
    }

    let attachments: Vec<MessageAttachment> = files
        .iter()
        .enumerate()
        .map(|(i, path)| MessageAttachment {
            id: format!("cli_file_{}", i + 1),
            name: std::path::Path::new(path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(path)
                .to_string(),
            kind: "file".into(),
            mime_type: None,
            path: Some(path.clone()),
            size_bytes: std::fs::metadata(path).ok().map(|meta| meta.len()),
        })
        .collect();

    let client = reqwest::Client::new();
    let response = client
        .post("http://127.0.0.1:8787/api/messages")
        .json(&json!({
            "conversation_id": "conv_cli_message",
            "content": content,
            "model_id": model_id,
            "attachments": attachments,
        }))
        .send()
        .await
        .expect("send message request")
        .json::<serde_json::Value>()
        .await
        .expect("deserialize message response");

    println!(
        "{}",
        serde_json::to_string_pretty(&response).expect("serialize response")
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

#[allow(dead_code)]
fn task_list() {
    let store = demo_store();
    let tasks = store.list_tasks().expect("list tasks");
    println!(
        "{}",
        serde_json::to_string_pretty(&tasks).expect("serialize task list")
    );
}
