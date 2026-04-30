//! Provider registry and provider-side integration contracts.

mod config;
mod registry;
mod types;

pub use config::ProviderConfig;
pub use registry::ProviderRegistry;
pub use types::{ProviderCapability, ProviderDescriptor, ProviderKind};
