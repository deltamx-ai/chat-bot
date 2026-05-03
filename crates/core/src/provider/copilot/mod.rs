use reqwest::Client;
use serde::Deserialize;

use crate::auth::{
    AuthChallenge, AuthMethod, AuthProvider, AuthSession, AuthState, Credential, CredentialKind,
};

const GITHUB_DEVICE_CODE_URL: &str = "https://github.com/login/device/code";
const GITHUB_DEVICE_VERIFY_URL: &str = "https://github.com/login/device";
pub const COPILOT_CLIENT_ID: &str = "Iv1.b507a08c87ecfe98";

#[derive(Debug, Deserialize)]
struct GitHubDeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: u64,
}

pub struct CopilotAuthProvider;

impl CopilotAuthProvider {
    pub fn new() -> Self {
        Self
    }

    pub async fn request_device_code_async(&self) -> Result<AuthChallenge, String> {
        let client = Client::new();
        let response = client
            .post(GITHUB_DEVICE_CODE_URL)
            .header("Accept", "application/json")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[("client_id", COPILOT_CLIENT_ID), ("scope", "read:email")])
            .send()
            .await
            .map_err(|err| format!("device code request failed: {err}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("device code request failed: {status} {body}"));
        }

        let payload: GitHubDeviceCodeResponse = response
            .json()
            .await
            .map_err(|err| format!("invalid device code response: {err}"))?;

        Ok(AuthChallenge {
            provider_id: self.id().into(),
            auth_url: GITHUB_DEVICE_VERIFY_URL.into(),
            user_code: payload.user_code,
            device_code: payload.device_code,
            verification_uri: payload.verification_uri,
            expires_in_seconds: payload.expires_in,
            poll_interval_seconds: payload.interval,
            can_copy_code: true,
            can_copy_url: true,
        })
    }
}

impl Default for CopilotAuthProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthProvider for CopilotAuthProvider {
    fn id(&self) -> &str {
        "copilot-github"
    }

    fn login(&self, credential: Credential) -> Result<AuthSession, String> {
        self.validate(&credential)?;
        Ok(AuthSession {
            provider_id: self.id().into(),
            method: AuthMethod::DeviceCode,
            state: AuthState::Pending,
            identity: None,
            credentials: vec![credential],
            challenge: Some(AuthChallenge {
                provider_id: self.id().into(),
                auth_url: GITHUB_DEVICE_VERIFY_URL.into(),
                user_code: String::new(),
                device_code: String::new(),
                verification_uri: GITHUB_DEVICE_VERIFY_URL.into(),
                expires_in_seconds: 900,
                poll_interval_seconds: 5,
                can_copy_code: true,
                can_copy_url: true,
            }),
        })
    }

    fn logout(&self, session: &AuthSession) -> Result<(), String> {
        if session.provider_id != self.id() {
            return Err("session provider mismatch".into());
        }
        Ok(())
    }

    fn refresh(&self, session: &AuthSession) -> Result<AuthSession, String> {
        if session.provider_id != self.id() {
            return Err("session provider mismatch".into());
        }
        Ok(session.clone())
    }

    fn validate(&self, credential: &Credential) -> Result<(), String> {
        match credential.kind {
            CredentialKind::DeviceCode
            | CredentialKind::UserCode
            | CredentialKind::SessionToken => Ok(()),
            _ => Err("unsupported copilot credential kind".into()),
        }
    }

    fn begin_device_flow(&self) -> Result<AuthChallenge, String> {
        Err("use async device flow request".into())
    }
}
