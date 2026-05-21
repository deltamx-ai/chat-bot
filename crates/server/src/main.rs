mod auth;
mod bootstrap;
mod routes;
mod state;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::{
        IntoResponse, Response,
        sse::{Event, KeepAlive, Sse},
    },
    routing::{get, post},
};
use chatbot_core::conversation::{ConversationStore, SqliteConversationStore};
use chatbot_core::execution::SqliteExecutionStore;
use chatbot_core::provider::{
    CopilotTokenSource, Delta, GenerateRequest, ProviderError, ProviderKind, ProviderRegistry,
};
use serde::Deserialize;
use serde_json::json;
use tokio::sync::{Mutex, mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tower_http::cors::CorsLayer;

use crate::auth::{CopilotAuthService, FileStorage};
use crate::state::{
    AppState, ApprovalDecisionRequest, CreateTeammateRequest, SendMessageRequest,
    SendTeammateMessageRequest, list_approval_requests, list_teammates, run_to_dto,
    teammate_message_to_dto, teammate_to_dto,
};

pub type SharedState = Arc<Mutex<AppState>>;

#[derive(Debug, Deserialize)]
struct RunTaskRequest {
    title: String,
    goal: String,
}

#[derive(Debug, Deserialize, Default)]
struct CreateConversationRequest {
    #[serde(default)]
    title: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct UpdateConversationRequest {
    #[serde(default)]
    title: Option<String>,
}

#[tokio::main]
async fn main() {
    let conversations = build_conversation_store().await;
    let execution_store = build_execution_store().await;
    let copilot_service = build_copilot_service().await;

    let mut app_state = AppState::new(conversations.clone(), execution_store);
    if let Err(err) = app_state
        .run_task("Demo task", "Create a task runtime demo payload")
        .await
    {
        eprintln!("[demo] run_task failed: {err}");
    }
    if let Some(service) = copilot_service.clone() {
        app_state = app_state.with_copilot_auth(service);
    }
    let state: SharedState = Arc::new(Mutex::new(app_state));

    let app = Router::new()
        .route("/api/health", get(health_handler))
        .route("/api/tasks", get(list_tasks_handler).post(run_task_handler))
        .route("/api/tasks/{task_id}", get(task_detail_handler))
        .route("/api/tasks/{task_id}/events", get(task_events_handler))
        .route(
            "/api/tasks/{task_id}/runs",
            post(create_run_handler).get(list_runs_handler),
        )
        .route(
            "/api/approval-requests",
            get(list_approval_requests_handler),
        )
        .route(
            "/api/approval-requests/{approval_id}/approve",
            post(approve_request_handler),
        )
        .route(
            "/api/approval-requests/{approval_id}/reject",
            post(reject_request_handler),
        )
        .route(
            "/api/teammates",
            get(list_teammates_handler).post(create_teammate_handler),
        )
        .route(
            "/api/teammates/{teammate_id}/messages",
            get(read_teammate_inbox_handler).post(send_teammate_message_handler),
        )
        .route("/api/models", get(models_handler))
        .route(
            "/api/conversations",
            get(conversations_handler).post(create_conversation_handler),
        )
        .route(
            "/api/conversations/{conversation_id}",
            axum::routing::patch(rename_conversation_handler).delete(delete_conversation_handler),
        )
        .route(
            "/api/conversations/{conversation_id}/messages",
            get(messages_handler),
        )
        .route("/api/messages", post(send_message_handler))
        .route("/api/messages/stream", post(send_message_stream_handler))
        .route("/api/uploads", post(upload_file_handler))
        .merge(routes::auth::router())
        .with_state(state)
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8787")
        .await
        .expect("bind server listener");

    println!("{}", bootstrap::bootstrap_banner());
    println!("server listening on http://127.0.0.1:8787");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("serve axum app");
}

async fn build_conversation_store() -> Arc<dyn ConversationStore + Send + Sync> {
    let path = conversation_db_path();
    match SqliteConversationStore::open_file(&path).await {
        Ok(store) => {
            println!("[conversations] sqlite at {}", path.display());
            Arc::new(store)
        }
        Err(err) => {
            eprintln!("[conversations] sqlite open failed: {err}; falling back to in-memory");
            Arc::new(chatbot_core::conversation::InMemoryConversationStore::new())
        }
    }
}

async fn build_execution_store() -> SqliteExecutionStore {
    let path = execution_db_path();
    match SqliteExecutionStore::open_file(&path).await {
        Ok(store) => {
            println!("[execution] sqlite at {}", path.display());
            store
        }
        Err(err) => {
            panic!("[execution] sqlite open failed: {err}");
        }
    }
}

fn conversation_db_path() -> PathBuf {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local").join("share"))
        })
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("chat-bot").join("chat.db")
}

fn execution_db_path() -> PathBuf {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local").join("share"))
        })
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("chat-bot").join("execution.db")
}

async fn build_copilot_service() -> Option<Arc<CopilotAuthService>> {
    let path = match FileStorage::default_path() {
        Ok(path) => path,
        Err(err) => {
            eprintln!("[copilot] storage path unavailable: {err}");
            return None;
        }
    };
    let storage = FileStorage::new(path);
    let service = match CopilotAuthService::new(storage) {
        Ok(service) => service,
        Err(err) => {
            eprintln!("[copilot] init failed: {err}");
            return None;
        }
    };
    if let Err(err) = service.boot().await {
        eprintln!("[copilot] boot failed: {err}");
    }
    Some(service)
}

async fn health_handler() -> impl IntoResponse {
    Json(json!({ "status": "ok", "service": "server" }))
}

async fn list_tasks_handler(State(state): State<SharedState>) -> impl IntoResponse {
    let app = state.lock().await;
    Json(app.tasks().await)
}

async fn task_detail_handler(
    State(state): State<SharedState>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let app = state.lock().await;
    Json(app.task(&task_id).await)
}

async fn task_events_handler(
    State(state): State<SharedState>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let app = state.lock().await;
    Json(app.events(&task_id).await)
}

async fn run_task_handler(
    State(state): State<SharedState>,
    Json(payload): Json<RunTaskRequest>,
) -> impl IntoResponse {
    let mut app = state.lock().await;
    match app.run_task(&payload.title, &payload.goal).await {
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

async fn create_run_handler(
    State(state): State<SharedState>,
    Path(task_id): Path<String>,
) -> Response {
    let mut app = state.lock().await;
    match app.create_run_for_task(&task_id).await {
        Ok(run) => (StatusCode::OK, Json(json!(run_to_dto(run)))).into_response(),
        Err(message) => error_response(StatusCode::BAD_REQUEST, message),
    }
}

async fn list_runs_handler(
    State(state): State<SharedState>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    let app = state.lock().await;
    Json(
        app.runs_by_task(&task_id)
            .await
            .into_iter()
            .map(run_to_dto)
            .collect::<Vec<_>>(),
    )
}

async fn list_approval_requests_handler(State(state): State<SharedState>) -> impl IntoResponse {
    let app = state.lock().await;
    Json(list_approval_requests(&app).await)
}

async fn approve_request_handler(
    State(state): State<SharedState>,
    Path(approval_id): Path<String>,
    payload: Option<Json<ApprovalDecisionRequest>>,
) -> Response {
    let mut app = state.lock().await;
    let note = payload.and_then(|Json(p)| p.decision_note);
    match app.resolve_approval_request(&approval_id, true, note).await {
        Ok(request) => {
            let dto = crate::state::list_approval_requests(&app)
                .await
                .into_iter()
                .find(|item| item.id == request.id.0)
                .expect("approval dto exists");
            (StatusCode::OK, Json(json!(dto))).into_response()
        }
        Err(message) => error_response(StatusCode::BAD_REQUEST, message),
    }
}

async fn reject_request_handler(
    State(state): State<SharedState>,
    Path(approval_id): Path<String>,
    payload: Option<Json<ApprovalDecisionRequest>>,
) -> Response {
    let mut app = state.lock().await;
    let note = payload.and_then(|Json(p)| p.decision_note);
    match app
        .resolve_approval_request(&approval_id, false, note)
        .await
    {
        Ok(request) => {
            let dto = crate::state::list_approval_requests(&app)
                .await
                .into_iter()
                .find(|item| item.id == request.id.0)
                .expect("approval dto exists");
            (StatusCode::OK, Json(json!(dto))).into_response()
        }
        Err(message) => error_response(StatusCode::BAD_REQUEST, message),
    }
}

async fn list_teammates_handler(State(state): State<SharedState>) -> impl IntoResponse {
    let app = state.lock().await;
    Json(list_teammates(&app).await)
}

async fn create_teammate_handler(
    State(state): State<SharedState>,
    Json(payload): Json<CreateTeammateRequest>,
) -> Response {
    let app = state.lock().await;
    match app.create_teammate(&payload.name, &payload.role).await {
        Ok(teammate) => (StatusCode::OK, Json(json!(teammate_to_dto(teammate)))).into_response(),
        Err(message) => error_response(StatusCode::BAD_REQUEST, message),
    }
}

async fn send_teammate_message_handler(
    State(state): State<SharedState>,
    Path(teammate_id): Path<String>,
    Json(payload): Json<SendTeammateMessageRequest>,
) -> Response {
    let app = state.lock().await;
    match app
        .send_teammate_message(&teammate_id, &payload.from_name, &payload.content)
        .await
    {
        Ok(message) => (
            StatusCode::OK,
            Json(json!(teammate_message_to_dto(message))),
        )
            .into_response(),
        Err(message) => error_response(StatusCode::BAD_REQUEST, message),
    }
}

async fn read_teammate_inbox_handler(
    State(state): State<SharedState>,
    Path(teammate_id): Path<String>,
) -> impl IntoResponse {
    let app = state.lock().await;
    let messages = app
        .read_teammate_inbox(&teammate_id)
        .await
        .unwrap_or_default();
    Json(
        messages
            .into_iter()
            .map(teammate_message_to_dto)
            .collect::<Vec<_>>(),
    )
}

async fn models_handler(State(state): State<SharedState>) -> impl IntoResponse {
    let state = state.lock().await;
    Json(state.models())
}

async fn conversations_handler(State(state): State<SharedState>) -> impl IntoResponse {
    let storage = {
        let guard = state.lock().await;
        guard.conversations()
    };
    Json(crate::state::list_conversations(storage.as_ref()).await)
}

async fn create_conversation_handler(
    State(state): State<SharedState>,
    payload: Option<Json<CreateConversationRequest>>,
) -> Response {
    let title = payload.and_then(|Json(p)| p.title);
    let storage = {
        let guard = state.lock().await;
        guard.conversations()
    };
    match crate::state::create_conversation(storage.as_ref(), title).await {
        Ok(conversation) => (StatusCode::OK, Json(json!(conversation))).into_response(),
        Err(message) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": message })),
        )
            .into_response(),
    }
}

async fn messages_handler(
    State(state): State<SharedState>,
    Path(conversation_id): Path<String>,
) -> impl IntoResponse {
    let storage = {
        let guard = state.lock().await;
        guard.conversations()
    };
    Json(crate::state::list_messages(storage.as_ref(), &conversation_id).await)
}

async fn rename_conversation_handler(
    State(state): State<SharedState>,
    Path(conversation_id): Path<String>,
    payload: Option<Json<UpdateConversationRequest>>,
) -> Response {
    let title = payload.and_then(|Json(p)| p.title);
    let Some(title) = title else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": "title is required" })),
        )
            .into_response();
    };
    let storage = {
        let guard = state.lock().await;
        guard.conversations()
    };
    match crate::state::rename_conversation(storage.as_ref(), &conversation_id, title).await {
        Ok(Some(conversation)) => (StatusCode::OK, Json(json!(conversation))).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "ok": false, "error": "conversation not found" })),
        )
            .into_response(),
        Err(message) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "error": message })),
        )
            .into_response(),
    }
}

async fn delete_conversation_handler(
    State(state): State<SharedState>,
    Path(conversation_id): Path<String>,
) -> Response {
    let storage = {
        let guard = state.lock().await;
        guard.conversations()
    };
    match crate::state::delete_conversation(storage.as_ref(), &conversation_id).await {
        Ok(true) => (StatusCode::OK, Json(json!({ "ok": true }))).into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "ok": false, "error": "conversation not found" })),
        )
            .into_response(),
        Err(message) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "error": message })),
        )
            .into_response(),
    }
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
) -> Response {
    let (storage, copilot_source_provider) = {
        let guard = state.lock().await;
        let copilot_source = guard.copilot_auth();
        (guard.conversations(), copilot_source)
    };

    let prepared = match crate::state::prepare_send_message(storage.as_ref(), payload).await {
        Ok(prepared) => prepared,
        Err(message) => return error_response(StatusCode::BAD_REQUEST, message),
    };

    let copilot_source =
        copilot_token_source_for_kind(&copilot_source_provider, prepared.selection.kind);
    let provider = ProviderRegistry::provider_for(prepared.selection.kind, copilot_source);
    let assistant_content = match provider
        .generate(&GenerateRequest {
            model_id: &prepared.selection.model_id,
            prompt: &prepared.content,
            attachments: &prepared.attachments,
            history: &prepared.history,
        })
        .await
    {
        Ok(response) => response.content,
        Err(error) => return error_response(provider_status(&error), error.to_string()),
    };

    let response =
        match crate::state::finalize_send_message(storage.as_ref(), prepared, assistant_content)
            .await
        {
            Ok(response) => response,
            Err(message) => return error_response(StatusCode::INTERNAL_SERVER_ERROR, message),
        };

    (StatusCode::OK, Json(json!(response))).into_response()
}

async fn send_message_stream_handler(
    State(state): State<SharedState>,
    Json(payload): Json<SendMessageRequest>,
) -> Response {
    let (storage, copilot_service) = {
        let guard = state.lock().await;
        (guard.conversations(), guard.copilot_auth())
    };

    let prepared = match crate::state::prepare_send_message(storage.as_ref(), payload).await {
        Ok(prepared) => prepared,
        Err(message) => return error_response(StatusCode::BAD_REQUEST, message),
    };

    let copilot_source = copilot_token_source_for_kind(&copilot_service, prepared.selection.kind);

    let (event_tx, event_rx) = mpsc::channel::<Result<Event, std::convert::Infallible>>(32);

    tokio::spawn(async move {
        run_stream_task(storage, prepared, copilot_source, event_tx).await;
    });

    Sse::new(ReceiverStream::new(event_rx))
        .keep_alive(KeepAlive::default())
        .into_response()
}

async fn run_stream_task(
    storage: Arc<dyn ConversationStore + Send + Sync>,
    prepared: crate::state::PreparedMessage,
    copilot_source: Option<Arc<dyn CopilotTokenSource>>,
    event_tx: mpsc::Sender<Result<Event, std::convert::Infallible>>,
) {
    let kind = prepared.selection.kind;
    let provider = ProviderRegistry::provider_for(kind, copilot_source);
    let (delta_tx, mut delta_rx) = mpsc::channel::<Delta>(32);

    let forward_tx = event_tx.clone();
    let forward = tokio::spawn(async move {
        while let Some(delta) = delta_rx.recv().await {
            let event = Event::default()
                .event("delta")
                .data(json!({ "content": delta.content }).to_string());
            if forward_tx.send(Ok(event)).await.is_err() {
                break;
            }
        }
    });

    let request = GenerateRequest {
        model_id: &prepared.selection.model_id,
        prompt: &prepared.content,
        attachments: &prepared.attachments,
        history: &prepared.history,
    };
    let result = provider.generate_stream(&request, delta_tx).await;
    let _ = forward.await;

    match result {
        Ok(response) => {
            let finalize_result =
                crate::state::finalize_send_message(storage.as_ref(), prepared, response.content)
                    .await;
            let event = match finalize_result {
                Ok(payload) => Event::default()
                    .event("done")
                    .data(json!(payload).to_string()),
                Err(message) => Event::default()
                    .event("error")
                    .data(json!({ "error": message }).to_string()),
            };
            let _ = event_tx.send(Ok(event)).await;
        }
        Err(err) => {
            let event = Event::default()
                .event("error")
                .data(json!({ "error": err.to_string() }).to_string());
            let _ = event_tx.send(Ok(event)).await;
        }
    }
}

fn copilot_token_source_for_kind(
    service: &Option<Arc<CopilotAuthService>>,
    kind: ProviderKind,
) -> Option<Arc<dyn CopilotTokenSource>> {
    if !matches!(kind, ProviderKind::Copilot) {
        return None;
    }
    service
        .clone()
        .map(|service| service as Arc<dyn CopilotTokenSource>)
}

fn error_response(status: StatusCode, message: String) -> Response {
    (status, Json(json!({ "ok": false, "error": message }))).into_response()
}

fn provider_status(error: &ProviderError) -> StatusCode {
    match error {
        ProviderError::MissingModel | ProviderError::UnknownProvider(_) => StatusCode::BAD_REQUEST,
        ProviderError::MissingToken | ProviderError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
        _ => StatusCode::BAD_GATEWAY,
    }
}
