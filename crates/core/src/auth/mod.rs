//! Unified authentication contracts shared by all providers.

mod error;
mod identity;
mod provider;
mod session;
mod types;

pub use error::AuthError;
pub use identity::Identity;
pub use provider::AuthProvider;
pub use session::AuthSession;
pub use types::{AuthChallenge, AuthMethod, AuthState, Credential, CredentialKind};
