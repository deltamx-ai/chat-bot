mod bootstrap;
mod routes;
mod state;

use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use chatbot_core::auth::AuthProvider;
use serde::Deserialize;
use serde_json::json;
use tower_http::cors::CorsLayer;

use crate::state::AppState;

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
    let provider = chatbot_core::provider::copilot::CopilotAuthProvider::default();
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
