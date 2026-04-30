pub mod artifact;
pub mod auth;
pub mod config;
pub mod conversation;
pub mod execution;
pub mod memory;
pub mod planning;
pub mod provider;
pub mod workspace;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
}

pub fn health() -> HealthResponse {
    HealthResponse {
        status: "ok".into(),
        service: "core".into(),
    }
}
