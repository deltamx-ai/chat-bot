mod bootstrap;
mod routes;
mod state;

use std::sync::{Arc, Mutex};

use axum::{
    Json, Router,
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use chatbot_core::auth::AuthProvider;
use serde::Deserialize;
use serde_json::json;
use tower_http::cors::CorsLayer;

use crate::state::{AppState, SendMessageRequest};

type SharedState = Arc<Mutex<AppState>>;

#[derive(Debug, Deserialize)]
struct RunTaskRequest {
    title: String,
    goal: String,
}

#[tokio::main]
async fn main() {
    let state: SharedState = Arc::new(Mutex::new(AppState::demo()));

    let app = Router::new()
        .route("/api/health", get(health_handler))
        .route("/api/tasks", get(list_tasks_handler).post(run_task_handler))
        .route("/api/tasks/{task_id}", get(task_detail_handler))
        .route("/api/tasks/{task_id}/events", get(task_events_handler))
        .route("/api/auth/copilot", get(copilot_auth_handler))
        .route("/api/auth/copilot/begin", post(begin_copilot_auth_handler))
        .route("/api/models", get(models_handler))
        .route("/api/conversations", get(conversations_handler))
        .route(
            "/api/conversations/{conversation_id}/messages",
            get(messages_handler),
        )
        .route("/api/messages", post(send_message_handler))
        .route("/api/uploads", post(upload_file_handler))
        .with_state(state)
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8787")
        .await
        .expect("bind server listener");

    println!("{}", bootstrap::bootstrap_banner());
    println!("server listening on http://127.0.0.1:8787");

    axum::serve(listener, app).await.expect("serve axum app");
}

async fn health_handler() -> impl IntoResponse {
    Json(json!({ "status": "ok", "service": "server" }))
}

async fn list_tasks_handler(State(state): State<SharedState>) -> impl IntoResponse {
    let state = state.lock().expect("lock state");
    Json(state.tasks())
}

async fn task_detail_handler(
    State(state): State<SharedState>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let state = state.lock().expect("lock state");
    Json(state.task(&task_id))
}

async fn task_events_handler(
    State(state): State<SharedState>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let state = state.lock().expect("lock state");
    Json(state.events(&task_id))
}

async fn run_task_handler(
    State(state): State<SharedState>,
    Json(payload): Json<RunTaskRequest>,
) -> impl IntoResponse {
    let mut state = state.lock().expect("lock state");
    match state.run_task(&payload.title, &payload.goal) {
        Ok(results) => (
            StatusCode::OK,
            Json(json!({ "ok": true, "results": results })),
        )
            .into_response(),
        Err(message) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": message })),
        )
            .into_response(),
    }
}

async fn copilot_auth_handler(State(state): State<SharedState>) -> impl IntoResponse {
    let state = state.lock().expect("lock state");
    Json(state.copilot_auth_state())
}

async fn begin_copilot_auth_handler(State(state): State<SharedState>) -> impl IntoResponse {
    let provider = chatbot_core::provider::copilot::CopilotAuthProvider;
    match provider.request_device_code_async().await {
        Ok(challenge) => {
            let session = chatbot_core::auth::AuthSession {
                provider_id: provider.id().into(),
                method: chatbot_core::auth::AuthMethod::DeviceCode,
                state: chatbot_core::auth::AuthState::Pending,
                identity: None,
                credentials: vec![chatbot_core::auth::Credential {
                    kind: chatbot_core::auth::CredentialKind::DeviceCode,
                    value: challenge.device_code.clone(),
                }],
                challenge: Some(challenge),
            };
            {
                let mut app_state = state.lock().expect("lock state");
                app_state.apply_copilot_session(session.clone());
            }
            (
                StatusCode::OK,
                Json(json!({ "ok": true, "session": session })),
            )
                .into_response()
        }
        Err(message) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": message })),
        )
            .into_response(),
    }
}

async fn models_handler(State(state): State<SharedState>) -> impl IntoResponse {
    let state = state.lock().expect("lock state");
    Json(state.models())
}

async fn conversations_handler(State(state): State<SharedState>) -> impl IntoResponse {
    let state = state.lock().expect("lock state");
    Json(state.list_conversations())
}

async fn messages_handler(
    State(state): State<SharedState>,
    Path(conversation_id): Path<String>,
) -> impl IntoResponse {
    let state = state.lock().expect("lock state");
    Json(state.list_messages(&conversation_id))
}

async fn upload_file_handler(
    State(_state): State<SharedState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut uploaded = Vec::new();
    let upload_dir = std::path::Path::new("/tmp/chat-bot-uploads");
    if let Err(error) = tokio::fs::create_dir_all(upload_dir).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "error": error.to_string() })),
        )
            .into_response();
    }

    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = field
            .file_name()
            .map(ToString::to_string)
            .unwrap_or_else(|| "upload.bin".into());
        let content_type = field.content_type().map(ToString::to_string);
        let data: bytes::Bytes = match field.bytes().await {
            Ok(bytes) => bytes,
            Err(error) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "ok": false, "error": error.to_string() })),
                )
                    .into_response();
            }
        };
        let safe_name = format!("{}-{}", uuid_like(), file_name.replace('/', "_"));
        let path = upload_dir.join(&safe_name);
        if let Err(error) = tokio::fs::write(&path, &data).await {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "ok": false, "error": error.to_string() })),
            )
                .into_response();
        }
        let kind = if content_type
            .as_deref()
            .unwrap_or("application/octet-stream")
            .starts_with("image/")
        {
            "image"
        } else {
            "file"
        };
        uploaded.push(json!({
            "id": safe_name,
            "name": file_name,
            "kind": kind,
            "mime_type": content_type,
            "path": path.display().to_string(),
            "size_bytes": data.len(),
        }));
    }

    (
        StatusCode::OK,
        Json(json!({ "ok": true, "files": uploaded })),
    )
        .into_response()
}

fn uuid_like() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("upload-{now}")
}

async fn send_message_handler(
    State(state): State<SharedState>,
    Json(payload): Json<SendMessageRequest>,
) -> impl IntoResponse {
    let mut state = state.lock().expect("lock state");
    match state.send_message(payload) {
        Ok(response) => (StatusCode::OK, Json(json!(response))).into_response(),
        Err(message) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": message })),
        )
            .into_response(),
    }
}
