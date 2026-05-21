use std::error::Error;
use std::fmt;

use reqwest::StatusCode;

#[derive(Debug)]
pub enum AuthError {
    Network(String),
    Http { status: StatusCode, body: String },
    Decode(String),
    AuthorizationPending,
    SlowDown,
    ExpiredToken,
    AccessDenied,
    Forbidden,
    MissingToken,
    UnknownSession,
    Cancelled,
    Persistence(String),
    Other(String),
}

impl AuthError {
    pub fn is_retryable_pending(&self) -> bool {
        matches!(self, Self::AuthorizationPending | Self::SlowDown)
    }
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network(msg) => write!(f, "network error: {msg}"),
            Self::Http { status, body } => {
                let body = body.trim();
                if body.is_empty() {
                    write!(f, "http error: {status}")
                } else {
                    write!(f, "http error: {status}: {body}")
                }
            }
            Self::Decode(msg) => write!(f, "response decode error: {msg}"),
            Self::AuthorizationPending => write!(f, "authorization pending"),
            Self::SlowDown => write!(f, "polling too fast, slow down"),
            Self::ExpiredToken => write!(f, "device code expired"),
            Self::AccessDenied => write!(f, "access denied by user"),
            Self::Forbidden => write!(f, "forbidden"),
            Self::MissingToken => write!(f, "no token available"),
            Self::UnknownSession => write!(f, "unknown auth session"),
            Self::Cancelled => write!(f, "authentication cancelled"),
            Self::Persistence(msg) => write!(f, "credential storage error: {msg}"),
            Self::Other(msg) => write!(f, "{msg}"),
        }
    }
}

impl Error for AuthError {}

impl From<reqwest::Error> for AuthError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_decode() {
            Self::Decode(err.to_string())
        } else {
            Self::Network(err.to_string())
        }
    }
}
