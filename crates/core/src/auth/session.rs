use serde::{Deserialize, Serialize};

use super::{AuthMethod, AuthState, Credential, Identity};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthSession {
    pub provider_id: String,
    pub method: AuthMethod,
    pub state: AuthState,
    pub identity: Option<Identity>,
    pub credentials: Vec<Credential>,
}
