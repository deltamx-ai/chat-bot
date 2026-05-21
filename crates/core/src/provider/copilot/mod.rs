use std::sync::OnceLock;

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use reqwest::{Client, StatusCode};
use serde::Deserialize;

use crate::auth::{AuthChallenge, AuthError, Identity};

const GITHUB_DEVICE_CODE_URL: &str = "https://github.com/login/device/code";
const GITHUB_ACCESS_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
pub const GITHUB_DEVICE_VERIFY_URL: &str = "https://github.com/login/device";
const GITHUB_USER_URL: &str = "https://api.github.com/user";
const COPILOT_TOKEN_URL: &str = "https://api.github.com/copilot_internal/v2/token";

pub const COPILOT_CLIENT_ID: &str = "Iv1.b507a08c87ecfe98";
pub const COPILOT_OAUTH_SCOPE: &str = "read:user";
const USER_AGENT: &str = "chat-bot/0.1";
pub const PROVIDER_ID: &str = "copilot-github";

#[derive(Debug, Clone)]
pub struct DeviceCodeGrant {
    pub device_code: String,
    pub challenge: AuthChallenge,
}

#[derive(Debug, Clone)]
pub struct CopilotToken {
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub refresh_in: ChronoDuration,
}

#[derive(Deserialize)]
struct DeviceCodeBody {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: u64,
}

#[derive(Deserialize)]
struct AccessTokenBody {
    access_token: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Deserialize)]
struct CopilotTokenBody {
    token: String,
    expires_at: i64,
    refresh_in: i64,
}

#[derive(Deserialize)]
struct UserBody {
    id: u64,
    login: String,
    name: Option<String>,
    email: Option<String>,
}

pub struct CopilotAuthClient {
    http: Client,
}

impl CopilotAuthClient {
    pub fn new() -> Result<Self, AuthError> {
        let http = Client::builder()
            .user_agent(USER_AGENT)
            .https_only(true)
            .build()
            .map_err(|err| AuthError::Other(format!("build http client: {err}")))?;
        Ok(Self { http })
    }

    pub fn shared() -> Result<&'static Self, AuthError> {
        static CLIENT: OnceLock<CopilotAuthClient> = OnceLock::new();
        if let Some(client) = CLIENT.get() {
            return Ok(client);
        }
        let built = Self::new()?;
        let _ = CLIENT.set(built);
        CLIENT
            .get()
            .ok_or_else(|| AuthError::Other("copilot http client init race".into()))
    }

    pub async fn request_device_code(&self) -> Result<DeviceCodeGrant, AuthError> {
        let response = self
            .http
            .post(GITHUB_DEVICE_CODE_URL)
            .header("Accept", "application/json")
            .form(&[
                ("client_id", COPILOT_CLIENT_ID),
                ("scope", COPILOT_OAUTH_SCOPE),
            ])
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AuthError::Http { status, body });
        }

        let body: DeviceCodeBody = response.json().await?;

        let challenge = AuthChallenge {
            provider_id: PROVIDER_ID.into(),
            auth_url: GITHUB_DEVICE_VERIFY_URL.into(),
            user_code: body.user_code,
            device_code: String::new(),
            verification_uri: body.verification_uri,
            expires_in_seconds: body.expires_in,
            poll_interval_seconds: body.interval.max(1),
            can_copy_code: true,
            can_copy_url: true,
        };

        Ok(DeviceCodeGrant {
            device_code: body.device_code,
            challenge,
        })
    }

    pub async fn exchange_device_code(&self, device_code: &str) -> Result<String, AuthError> {
        let response = self
            .http
            .post(GITHUB_ACCESS_TOKEN_URL)
            .header("Accept", "application/json")
            .form(&[
                ("client_id", COPILOT_CLIENT_ID),
                ("device_code", device_code),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ])
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AuthError::Http { status, body });
        }

        let body: AccessTokenBody = response.json().await?;
        if let Some(token) = body.access_token {
            return Ok(token);
        }

        let error = body.error.as_deref().unwrap_or("");
        match error {
            "authorization_pending" => Err(AuthError::AuthorizationPending),
            "slow_down" => Err(AuthError::SlowDown),
            "expired_token" => Err(AuthError::ExpiredToken),
            "access_denied" => Err(AuthError::AccessDenied),
            other => {
                let message = body
                    .error_description
                    .unwrap_or_else(|| format!("oauth error: {other}"));
                Err(AuthError::Other(message))
            }
        }
    }

    pub async fn exchange_copilot_token(
        &self,
        github_token: &str,
    ) -> Result<CopilotToken, AuthError> {
        let response = self
            .http
            .get(COPILOT_TOKEN_URL)
            .bearer_auth(github_token)
            .header("Accept", "application/json")
            .header("Editor-Version", "chat-bot/0.1")
            .header("Editor-Plugin-Version", "chat-bot/0.1")
            .send()
            .await?;

        let status = response.status();
        if status == StatusCode::UNAUTHORIZED {
            return Err(AuthError::MissingToken);
        }
        if status == StatusCode::FORBIDDEN {
            return Err(AuthError::Forbidden);
        }
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AuthError::Http { status, body });
        }

        let body: CopilotTokenBody = response.json().await?;
        let expires_at = DateTime::<Utc>::from_timestamp(body.expires_at, 0)
            .ok_or_else(|| AuthError::Decode("invalid expires_at".into()))?;
        let refresh_in = ChronoDuration::seconds(body.refresh_in.max(60));
        Ok(CopilotToken {
            token: body.token,
            expires_at,
            refresh_in,
        })
    }

    pub async fn fetch_identity(&self, github_token: &str) -> Result<Identity, AuthError> {
        let response = self
            .http
            .get(GITHUB_USER_URL)
            .bearer_auth(github_token)
            .header("Accept", "application/vnd.github+json")
            .send()
            .await?;

        let status = response.status();
        if status == StatusCode::UNAUTHORIZED {
            return Err(AuthError::MissingToken);
        }
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AuthError::Http { status, body });
        }

        let body: UserBody = response.json().await?;
        Ok(Identity {
            id: body.id.to_string(),
            display_name: body.name.unwrap_or(body.login),
            email: body.email,
            provider: PROVIDER_ID.into(),
        })
    }
}
