use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub enum FlowStatus {
    Pending,
    Authenticated,
    Failed(String),
    Cancelled,
    Expired,
}

impl FlowStatus {
    pub fn is_terminal(&self) -> bool {
        !matches!(self, Self::Pending)
    }
}

#[derive(Debug, Clone)]
pub struct Flow {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_at: DateTime<Utc>,
    pub poll_interval_seconds: u64,
    pub status: FlowStatus,
}
