use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use uuid::Uuid;

use chatbot_core::auth::{AuthError, AuthState, Identity};
use chatbot_core::provider::CopilotTokenSource;
use chatbot_core::provider::ProviderError;
use chatbot_core::provider::copilot::{CopilotAuthClient, GITHUB_DEVICE_VERIFY_URL, PROVIDER_ID};

use super::flow::{Flow, FlowStatus};
use super::storage::{FileStorage, PersistedSession};
use super::view::{CopilotAuthView, FlowSnapshot, ProviderDescriptor, PublicChallenge};

const MIN_REFRESH_SECS: i64 = 60;
const MAX_TRACKED_FLOWS: usize = 16;
const NEAR_EXPIRY_SECS: i64 = 30;
const UNAUTHORIZED_STATUS: u16 = 401;

#[derive(Debug, Clone)]
pub struct CopilotSession {
    pub github_token: String,
    pub copilot_token: String,
    pub copilot_expires_at: DateTime<Utc>,
    pub identity: Option<Identity>,
}

struct Inner {
    client: &'static CopilotAuthClient,
    flows: HashMap<String, Flow>,
    session: Option<CopilotSession>,
    refresh_task: Option<JoinHandle<()>>,
    storage: FileStorage,
}

impl Inner {
    fn evict_stale_flows(&mut self) {
        if self.flows.len() <= MAX_TRACKED_FLOWS {
            return;
        }
        let mut terminal: Vec<String> = self
            .flows
            .iter()
            .filter(|(_, flow)| flow.status.is_terminal())
            .map(|(id, _)| id.clone())
            .collect();
        terminal.sort();
        while self.flows.len() > MAX_TRACKED_FLOWS
            && let Some(id) = terminal.pop()
        {
            self.flows.remove(&id);
        }
    }
}

pub struct CopilotAuthService {
    inner: Mutex<Inner>,
}

impl std::fmt::Debug for CopilotAuthService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CopilotAuthService").finish_non_exhaustive()
    }
}

impl CopilotAuthService {
    pub fn new(storage: FileStorage) -> Result<Arc<Self>, AuthError> {
        let client = CopilotAuthClient::shared()?;
        let inner = Inner {
            client,
            flows: HashMap::new(),
            session: None,
            refresh_task: None,
            storage,
        };
        Ok(Arc::new(Self {
            inner: Mutex::new(inner),
        }))
    }

    pub async fn boot(self: &Arc<Self>) -> Result<(), AuthError> {
        let (client, storage) = {
            let guard = self.inner.lock().await;
            (guard.client, guard.storage.clone())
        };
        let Some(persisted) = storage.load().await? else {
            return Ok(());
        };

        let identity = match client.fetch_identity(&persisted.github_token).await {
            Ok(id) => Some(id),
            Err(AuthError::MissingToken | AuthError::Forbidden) => {
                storage.clear().await?;
                return Ok(());
            }
            Err(err) => return Err(err),
        };

        let copilot = match client.exchange_copilot_token(&persisted.github_token).await {
            Ok(token) => token,
            Err(AuthError::MissingToken | AuthError::Forbidden) => {
                storage.clear().await?;
                return Ok(());
            }
            Err(err) => return Err(err),
        };

        let session = CopilotSession {
            github_token: persisted.github_token,
            copilot_token: copilot.token,
            copilot_expires_at: copilot.expires_at,
            identity,
        };

        let mut guard = self.inner.lock().await;
        guard.session = Some(session);
        self.schedule_refresh(&mut guard, copilot.refresh_in);
        Ok(())
    }

    pub async fn begin_flow(self: &Arc<Self>) -> Result<PublicChallenge, AuthError> {
        let client = self.inner.lock().await.client;
        let grant = client.request_device_code().await?;

        let session_id = Uuid::new_v4().to_string();
        let expires_at = Utc::now()
            + ChronoDuration::seconds(
                i64::try_from(grant.challenge.expires_in_seconds).unwrap_or(0),
            );

        let flow = Flow {
            device_code: grant.device_code,
            user_code: grant.challenge.user_code.clone(),
            verification_uri: grant.challenge.verification_uri.clone(),
            expires_at,
            poll_interval_seconds: grant.challenge.poll_interval_seconds,
            status: FlowStatus::Pending,
        };

        let challenge = PublicChallenge {
            session_id: session_id.clone(),
            user_code: flow.user_code.clone(),
            verification_uri: flow.verification_uri.clone(),
            expires_in_seconds: i64::try_from(grant.challenge.expires_in_seconds).unwrap_or(0),
            poll_interval_seconds: flow.poll_interval_seconds,
        };

        {
            let mut guard = self.inner.lock().await;
            guard.flows.insert(session_id.clone(), flow);
            guard.evict_stale_flows();
        }

        let service = Arc::clone(self);
        tokio::spawn(async move {
            service.poll_flow_loop(session_id).await;
        });

        Ok(challenge)
    }

    async fn poll_flow_loop(self: Arc<Self>, session_id: String) {
        loop {
            let (client, device_code, interval, expires_at) = {
                let guard = self.inner.lock().await;
                let Some(flow) = guard.flows.get(&session_id) else {
                    return;
                };
                if !matches!(flow.status, FlowStatus::Pending) {
                    return;
                }
                (
                    guard.client,
                    flow.device_code.clone(),
                    flow.poll_interval_seconds,
                    flow.expires_at,
                )
            };

            if Utc::now() >= expires_at {
                self.set_flow_status(&session_id, FlowStatus::Expired).await;
                return;
            }

            tokio::time::sleep(Duration::from_secs(interval)).await;

            {
                let guard = self.inner.lock().await;
                let Some(flow) = guard.flows.get(&session_id) else {
                    return;
                };
                if !matches!(flow.status, FlowStatus::Pending) {
                    return;
                }
            }

            match client.exchange_device_code(&device_code).await {
                Ok(github_token) => {
                    if let Err(err) = self.complete_flow(&session_id, github_token).await {
                        self.set_flow_status(&session_id, FlowStatus::Failed(err.to_string()))
                            .await;
                    }
                    return;
                }
                Err(AuthError::AuthorizationPending) => continue,
                Err(AuthError::SlowDown) => {
                    let mut guard = self.inner.lock().await;
                    if let Some(flow) = guard.flows.get_mut(&session_id) {
                        flow.poll_interval_seconds = flow.poll_interval_seconds.saturating_add(5);
                    }
                }
                Err(AuthError::ExpiredToken) => {
                    self.set_flow_status(&session_id, FlowStatus::Expired).await;
                    return;
                }
                Err(AuthError::AccessDenied) => {
                    self.set_flow_status(
                        &session_id,
                        FlowStatus::Failed("access denied by user".into()),
                    )
                    .await;
                    return;
                }
                Err(err) => {
                    self.set_flow_status(&session_id, FlowStatus::Failed(err.to_string()))
                        .await;
                    return;
                }
            }
        }
    }

    async fn complete_flow(
        self: &Arc<Self>,
        session_id: &str,
        github_token: String,
    ) -> Result<(), AuthError> {
        let (client, storage) = {
            let guard = self.inner.lock().await;
            (guard.client, guard.storage.clone())
        };

        let identity = client.fetch_identity(&github_token).await.ok();
        let copilot = client.exchange_copilot_token(&github_token).await?;

        storage
            .save(&PersistedSession {
                github_token: github_token.clone(),
                identity: identity.clone(),
            })
            .await?;

        let session = CopilotSession {
            github_token,
            copilot_token: copilot.token,
            copilot_expires_at: copilot.expires_at,
            identity,
        };

        let mut guard = self.inner.lock().await;
        guard.session = Some(session);
        if let Some(flow) = guard.flows.get_mut(session_id) {
            flow.status = FlowStatus::Authenticated;
            flow.device_code.clear();
        }
        self.schedule_refresh(&mut guard, copilot.refresh_in);
        Ok(())
    }

    fn schedule_refresh(self: &Arc<Self>, inner: &mut Inner, refresh_in: ChronoDuration) {
        if let Some(existing) = inner.refresh_task.take() {
            existing.abort();
        }
        let secs = refresh_in.num_seconds().max(MIN_REFRESH_SECS) as u64;
        let service = Arc::clone(self);
        let handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(secs)).await;
            let _ = service.refresh_copilot_token().await;
        });
        inner.refresh_task = Some(handle);
    }

    async fn refresh_copilot_token(self: Arc<Self>) -> Result<(), AuthError> {
        let (client, github_token) = {
            let guard = self.inner.lock().await;
            let Some(session) = guard.session.as_ref() else {
                return Err(AuthError::MissingToken);
            };
            (guard.client, session.github_token.clone())
        };

        match client.exchange_copilot_token(&github_token).await {
            Ok(copilot) => {
                let mut guard = self.inner.lock().await;
                if let Some(session) = guard.session.as_mut() {
                    session.copilot_token = copilot.token;
                    session.copilot_expires_at = copilot.expires_at;
                }
                self.schedule_refresh(&mut guard, copilot.refresh_in);
                Ok(())
            }
            Err(AuthError::MissingToken | AuthError::Forbidden) => {
                self.logout().await?;
                Err(AuthError::MissingToken)
            }
            Err(err) => Err(err),
        }
    }

    pub async fn poll_flow(&self, session_id: &str) -> FlowSnapshot {
        let guard = self.inner.lock().await;
        let Some(flow) = guard.flows.get(session_id) else {
            return FlowSnapshot::Unknown;
        };
        match &flow.status {
            FlowStatus::Pending => {
                let expires_in = (flow.expires_at - Utc::now()).num_seconds().max(0);
                FlowSnapshot::Pending {
                    user_code: flow.user_code.clone(),
                    verification_uri: flow.verification_uri.clone(),
                    expires_in_seconds: expires_in,
                    poll_interval_seconds: flow.poll_interval_seconds,
                }
            }
            FlowStatus::Authenticated => FlowSnapshot::Authenticated {
                identity: guard
                    .session
                    .as_ref()
                    .and_then(|session| session.identity.clone()),
            },
            FlowStatus::Failed(message) => FlowSnapshot::Failed {
                error: message.clone(),
            },
            FlowStatus::Cancelled => FlowSnapshot::Cancelled,
            FlowStatus::Expired => FlowSnapshot::Expired,
        }
    }

    pub async fn cancel_flow(&self, session_id: &str) -> Result<(), AuthError> {
        let mut guard = self.inner.lock().await;
        match guard.flows.get_mut(session_id) {
            Some(flow) if matches!(flow.status, FlowStatus::Pending) => {
                flow.status = FlowStatus::Cancelled;
                flow.device_code.clear();
                Ok(())
            }
            Some(_) => Err(AuthError::Other("flow is no longer pending".into())),
            None => Err(AuthError::UnknownSession),
        }
    }

    pub async fn current_view(&self) -> CopilotAuthView {
        let guard = self.inner.lock().await;
        match guard.session.as_ref() {
            Some(session) => CopilotAuthView {
                provider: provider_descriptor(),
                state: AuthState::Authenticated,
                identity: session.identity.clone(),
                copilot_token_expires_at: Some(session.copilot_expires_at),
            },
            None => CopilotAuthView {
                provider: provider_descriptor(),
                state: AuthState::Unauthenticated,
                identity: None,
                copilot_token_expires_at: None,
            },
        }
    }

    pub async fn logout(&self) -> Result<(), AuthError> {
        let mut guard = self.inner.lock().await;
        guard.session = None;
        if let Some(handle) = guard.refresh_task.take() {
            handle.abort();
        }
        guard.storage.clear().await
    }

    pub async fn copilot_token(&self) -> Option<String> {
        let guard = self.inner.lock().await;
        guard
            .session
            .as_ref()
            .map(|session| session.copilot_token.clone())
    }

    pub async fn ensure_token(&self) -> Result<String, AuthError> {
        let near_expiry = {
            let guard = self.inner.lock().await;
            let Some(session) = guard.session.as_ref() else {
                return Err(AuthError::MissingToken);
            };
            let remaining = session.copilot_expires_at - Utc::now();
            if remaining > ChronoDuration::seconds(NEAR_EXPIRY_SECS) {
                return Ok(session.copilot_token.clone());
            }
            true
        };
        if near_expiry {
            return self.refresh_now().await;
        }
        unreachable!("near_expiry was true above")
    }

    pub async fn refresh_now(&self) -> Result<String, AuthError> {
        let (client, github_token) = {
            let guard = self.inner.lock().await;
            let Some(session) = guard.session.as_ref() else {
                return Err(AuthError::MissingToken);
            };
            (guard.client, session.github_token.clone())
        };
        let copilot = client.exchange_copilot_token(&github_token).await?;
        let mut guard = self.inner.lock().await;
        if let Some(session) = guard.session.as_mut() {
            session.copilot_token = copilot.token.clone();
            session.copilot_expires_at = copilot.expires_at;
        }
        Ok(copilot.token)
    }

    async fn set_flow_status(&self, session_id: &str, status: FlowStatus) {
        let mut guard = self.inner.lock().await;
        if let Some(flow) = guard.flows.get_mut(session_id) {
            flow.status = status;
            flow.device_code.clear();
        }
    }
}

fn provider_descriptor() -> ProviderDescriptor {
    ProviderDescriptor {
        id: PROVIDER_ID.into(),
        display_name: "GitHub Copilot".into(),
        verification_uri: GITHUB_DEVICE_VERIFY_URL.into(),
    }
}

#[async_trait]
impl CopilotTokenSource for CopilotAuthService {
    async fn copilot_token(&self) -> Result<String, ProviderError> {
        self.ensure_token().await.map_err(map_auth_error)
    }

    async fn refresh_copilot_token(&self) -> Result<String, ProviderError> {
        self.refresh_now().await.map_err(map_auth_error)
    }
}

fn map_auth_error(err: AuthError) -> ProviderError {
    match err {
        AuthError::MissingToken => ProviderError::MissingToken,
        AuthError::Forbidden => ProviderError::Unauthorized(err.to_string()),
        AuthError::Http { status, body } if status.as_u16() == UNAUTHORIZED_STATUS => {
            ProviderError::Unauthorized(body)
        }
        AuthError::Network(msg) => ProviderError::Network(msg),
        other => ProviderError::Network(other.to_string()),
    }
}
