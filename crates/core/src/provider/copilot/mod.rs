use std::{thread, time::Duration};

use reqwest::Client;
use serde::Deserialize;

use crate::auth::{
    AuthChallenge, AuthMethod, AuthProvider, AuthSession, AuthState, Credential, CredentialKind,
};

const GITHUB_DEVICE_CODE_URL: &str = "https://github.com/login/device/code";
const GITHUB_ACCESS_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
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

#[derive(Debug, Deserialize)]
struct GitHubAccessTokenResponse {
    access_token: Option<String>,
    _token_type: Option<String>,
    _scope: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
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

    pub fn poll_access_token(&self, challenge: &AuthChallenge) -> Result<AuthSession, String> {
        let client = reqwest::blocking::Client::new();
        let max_attempts = usize::try_from(
            (challenge.expires_in_seconds / challenge.poll_interval_seconds).max(1),
        )
        .unwrap_or(180);

        for _ in 0..max_attempts {
            let response = client
                .post(GITHUB_ACCESS_TOKEN_URL)
                .header("Accept", "application/json")
                .header("Content-Type", "application/x-www-form-urlencoded")
                .form(&[
                    ("client_id", COPILOT_CLIENT_ID),
                    ("device_code", challenge.device_code.as_str()),
                    ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ])
                .send()
                .map_err(|err| format!("access token request failed: {err}"))?;

            let payload: GitHubAccessTokenResponse = response
                .json()
                .map_err(|err| format!("invalid access token response: {err}"))?;

            if let Some(access_token) = payload.access_token {
                return Ok(AuthSession {
                    provider_id: self.id().into(),
                    method: AuthMethod::OAuth,
                    state: AuthState::Authenticated,
                    identity: None,
                    credentials: vec![Credential {
                        kind: CredentialKind::AccessToken,
                        value: access_token,
                    }],
                    challenge: Some(challenge.clone()),
                });
            }

            match payload.error.as_deref() {
                Some("authorization_pending") => {
                    thread::sleep(Duration::from_secs(challenge.poll_interval_seconds));
                }
                Some("slow_down") => {
                    thread::sleep(Duration::from_secs(challenge.poll_interval_seconds + 5));
                }
                Some("expired_token") => return Err("device code expired".into()),
                Some(other) => {
                    return Err(payload
                        .error_description
                        .unwrap_or_else(|| format!("oauth error: {other}")));
                }
                None => return Err("missing access token in oauth response".into()),
            }
        }

        Err("timed out waiting for device authorization".into())
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
            | CredentialKind::SessionToken
            | CredentialKind::AccessToken => Ok(()),
            _ => Err("unsupported copilot credential kind".into()),
        }
    }

    fn begin_device_flow(&self) -> Result<AuthChallenge, String> {
        Err("use async device flow request".into())
    }
}
