use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identity {
    pub id: String,
    pub display_name: String,
    pub email: Option<String>,
    pub provider: String,
}
