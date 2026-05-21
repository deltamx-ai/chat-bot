use chrono::{DateTime, Utc};
use serde::Serialize;

use chatbot_core::auth::{AuthState, Identity};

#[derive(Debug, Clone, Serialize)]
pub struct PublicChallenge {
    pub session_id: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in_seconds: i64,
    pub poll_interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum FlowSnapshot {
    Pending {
        user_code: String,
        verification_uri: String,
        expires_in_seconds: i64,
        poll_interval_seconds: u64,
    },
    Authenticated {
        identity: Option<Identity>,
    },
    Failed {
        error: String,
    },
    Cancelled,
    Expired,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderDescriptor {
    pub id: String,
    pub display_name: String,
    pub verification_uri: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CopilotAuthView {
    pub provider: ProviderDescriptor,
    pub state: AuthState,
    pub identity: Option<Identity>,
    pub copilot_token_expires_at: Option<DateTime<Utc>>,
}
