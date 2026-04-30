use serde::{Deserialize, Serialize};

use crate::{WorkspaceId, auth::AuthSession, conversation::ConversationId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ExecutionContext {
    pub workspace: Option<WorkspaceId>,
    pub conversation_id: Option<ConversationId>,
    pub auth_session: Option<AuthSession>,
    pub provider_id: Option<String>,
    pub config_snapshot: Option<String>,
}
