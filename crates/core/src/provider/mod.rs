//! Provider registry and provider-side integration contracts.

mod config;
pub mod copilot;
mod error;
mod registry;
pub mod runtime;
mod types;

pub use config::ProviderConfig;
pub use error::ProviderError;
pub use registry::{ModelSelection, ProviderRegistry};
pub use runtime::{
    AnthropicProvider, ChatRole, ChatTurn, CopilotTokenSource, Delta, GenerateRequest,
    GenerateResponse, OpenAiProvider, Provider,
};
pub use types::{ProviderCapability, ProviderDescriptor, ProviderKind};
