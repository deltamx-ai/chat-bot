use serde::{Deserialize, Serialize};

use super::{ProviderCapability, ProviderKind};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub id: String,
    pub kind: ProviderKind,
    pub enabled: bool,
    pub base_url: Option<String>,
    pub capabilities: Vec<ProviderCapability>,
}
