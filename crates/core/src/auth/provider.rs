use super::{AuthSession, Credential};

pub trait AuthProvider {
    fn id(&self) -> &str;

    fn login(&self, credential: Credential) -> Result<AuthSession, String>;

    fn logout(&self, session: &AuthSession) -> Result<(), String>;

    fn refresh(&self, session: &AuthSession) -> Result<AuthSession, String>;

    fn validate(&self, credential: &Credential) -> Result<(), String>;
}
