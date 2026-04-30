use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
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
