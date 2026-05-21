use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use chatbot_core::auth::{AuthError, Identity};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedSession {
    pub github_token: String,
    #[serde(default)]
    pub identity: Option<Identity>,
}

#[derive(Debug, Clone)]
pub struct FileStorage {
    path: PathBuf,
}

impl FileStorage {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn default_path() -> Result<PathBuf, AuthError> {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
            .ok_or_else(|| AuthError::Persistence("HOME and XDG_CONFIG_HOME both unset".into()))?;
        Ok(base.join("chat-bot").join("copilot.json"))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub async fn load(&self) -> Result<Option<PersistedSession>, AuthError> {
        match tokio::fs::read(&self.path).await {
            Ok(bytes) => {
                let session = serde_json::from_slice::<PersistedSession>(&bytes)
                    .map_err(|err| AuthError::Persistence(format!("decode failed: {err}")))?;
                Ok(Some(session))
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(AuthError::Persistence(err.to_string())),
        }
    }

    pub async fn save(&self, session: &PersistedSession) -> Result<(), AuthError> {
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|err| AuthError::Persistence(err.to_string()))?;
        }
        let bytes = serde_json::to_vec_pretty(session)
            .map_err(|err| AuthError::Persistence(err.to_string()))?;
        tokio::fs::write(&self.path, &bytes)
            .await
            .map_err(|err| AuthError::Persistence(err.to_string()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            tokio::fs::set_permissions(&self.path, perms)
                .await
                .map_err(|err| AuthError::Persistence(err.to_string()))?;
        }
        Ok(())
    }

    pub async fn clear(&self) -> Result<(), AuthError> {
        match tokio::fs::remove_file(&self.path).await {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(AuthError::Persistence(err.to_string())),
        }
    }
}
