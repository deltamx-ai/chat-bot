use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStrategy {
    Sequential,
    Parallel,
    RequiresConfirmation,
}
