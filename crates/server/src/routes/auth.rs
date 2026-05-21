use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{ConnectInfo, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde_json::json;

use chatbot_core::auth::AuthError;

use crate::SharedState;
use crate::auth::{CopilotAuthService, RateLimiter};

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/api/auth/copilot", get(state_handler))
        .route("/api/auth/copilot/begin", post(begin_handler))
        .route("/api/auth/copilot/poll/{session_id}", get(poll_handler))
        .route(
            "/api/auth/copilot/cancel/{session_id}",
            post(cancel_handler),
        )
        .route("/api/auth/copilot/logout", post(logout_handler))
}

async fn state_handler(State(state): State<SharedState>) -> Response {
    let Some(service) = take_service(&state).await else {
        return missing_service();
    };
    let view = service.current_view().await;
    (StatusCode::OK, Json(view)).into_response()
}

async fn begin_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<SharedState>,
) -> Response {
    let (service, limiter) = take_service_and_limiter(&state).await;
    let Some(service) = service else {
        return missing_service();
    };
    if !limiter.allow(addr.ip()) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({ "ok": false, "error": "rate limited" })),
        )
            .into_response();
    }

    match service.begin_flow().await {
        Ok(challenge) => (
            StatusCode::OK,
            Json(json!({ "ok": true, "challenge": challenge })),
        )
            .into_response(),
        Err(err) => auth_error_response(&err),
    }
}

async fn poll_handler(
    State(state): State<SharedState>,
    Path(session_id): Path<String>,
) -> Response {
    let Some(service) = take_service(&state).await else {
        return missing_service();
    };
    let snapshot = service.poll_flow(&session_id).await;
    (StatusCode::OK, Json(snapshot)).into_response()
}

async fn cancel_handler(
    State(state): State<SharedState>,
    Path(session_id): Path<String>,
) -> Response {
    let Some(service) = take_service(&state).await else {
        return missing_service();
    };
    match service.cancel_flow(&session_id).await {
        Ok(()) => (StatusCode::OK, Json(json!({ "ok": true }))).into_response(),
        Err(err) => auth_error_response(&err),
    }
}

async fn logout_handler(State(state): State<SharedState>) -> Response {
    let Some(service) = take_service(&state).await else {
        return missing_service();
    };
    match service.logout().await {
        Ok(()) => (StatusCode::OK, Json(json!({ "ok": true }))).into_response(),
        Err(err) => auth_error_response(&err),
    }
}

async fn take_service(state: &SharedState) -> Option<Arc<CopilotAuthService>> {
    let guard = state.lock().await;
    guard.copilot_auth()
}

async fn take_service_and_limiter(
    state: &SharedState,
) -> (Option<Arc<CopilotAuthService>>, Arc<RateLimiter>) {
    let guard = state.lock().await;
    (guard.copilot_auth(), guard.copilot_limiter.clone())
}

fn missing_service() -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({ "ok": false, "error": "copilot auth not configured" })),
    )
        .into_response()
}

fn auth_error_response(err: &AuthError) -> Response {
    let status = match err {
        AuthError::UnknownSession => StatusCode::NOT_FOUND,
        AuthError::Cancelled | AuthError::AccessDenied | AuthError::Forbidden => {
            StatusCode::FORBIDDEN
        }
        AuthError::AuthorizationPending | AuthError::SlowDown | AuthError::ExpiredToken => {
            StatusCode::CONFLICT
        }
        AuthError::Network(_) | AuthError::Http { .. } | AuthError::Decode(_) => {
            StatusCode::BAD_GATEWAY
        }
        AuthError::Persistence(_) => StatusCode::INTERNAL_SERVER_ERROR,
        AuthError::MissingToken => StatusCode::UNAUTHORIZED,
        AuthError::Other(_) => StatusCode::BAD_REQUEST,
    };
    (
        status,
        Json(json!({ "ok": false, "error": err.to_string() })),
    )
        .into_response()
}
