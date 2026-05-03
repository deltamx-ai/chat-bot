use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthMethod {
    ApiKey,
    OAuth,
    DeviceCode,
    SessionToken,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CredentialKind {
    ApiKey,
    AccessToken,
    RefreshToken,
    SessionToken,
    DeviceCode,
    UserCode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthState {
    Unauthenticated,
    Pending,
    Authenticated,
    Expired,
    Revoked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Credential {
    pub kind: CredentialKind,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthChallenge {
    pub provider_id: String,
    pub auth_url: String,
    pub user_code: String,
    pub device_code: String,
    pub verification_uri: String,
    pub expires_in_seconds: u64,
    pub poll_interval_seconds: u64,
    pub can_copy_code: bool,
    pub can_copy_url: bool,
}
