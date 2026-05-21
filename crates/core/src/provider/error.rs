use std::error::Error;
use std::fmt;
use std::time::Duration;

#[derive(Debug)]
pub enum ProviderError {
    MissingModel,
    UnknownProvider(String),
    MissingToken,
    Unauthorized(String),
    Network(String),
    Spawn(String),
    Wait(String),
    Io(String),
    Timeout(Duration),
    NonZeroExit { code: Option<i32>, stderr: String },
    EmptyOutput,
    Cancelled,
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingModel => write!(f, "no model selected"),
            Self::UnknownProvider(name) => write!(f, "unknown provider: {name}"),
            Self::MissingToken => write!(f, "no auth token available for provider"),
            Self::Unauthorized(msg) => {
                let msg = msg.trim();
                if msg.is_empty() {
                    write!(f, "provider rejected the token")
                } else {
                    write!(f, "provider rejected the token: {msg}")
                }
            }
            Self::Network(msg) => write!(f, "network error: {msg}"),
            Self::Spawn(msg) => write!(f, "failed to spawn provider process: {msg}"),
            Self::Wait(msg) => write!(f, "failed to wait on provider process: {msg}"),
            Self::Io(msg) => write!(f, "provider io error: {msg}"),
            Self::Timeout(d) => write!(f, "provider call timed out after {} seconds", d.as_secs()),
            Self::NonZeroExit { code, stderr } => {
                let code = code
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "<signal>".into());
                let stderr = stderr.trim();
                if stderr.is_empty() {
                    write!(f, "provider exited with status {code}")
                } else {
                    write!(f, "provider exited with status {code}: {stderr}")
                }
            }
            Self::EmptyOutput => write!(f, "provider returned empty output"),
            Self::Cancelled => write!(f, "provider call cancelled"),
        }
    }
}

impl Error for ProviderError {}
