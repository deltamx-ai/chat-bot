use super::AuthMethod;

pub trait AuthProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn methods(&self) -> &'static [AuthMethod];
}
