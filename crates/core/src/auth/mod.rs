//! Unified authentication contracts shared by all providers.

mod identity;
mod provider;
mod session;
mod types;

pub use identity::Identity;
pub use provider::AuthProvider;
pub use session::AuthSession;
pub use types::{AuthMethod, AuthState, Credential, CredentialKind};
