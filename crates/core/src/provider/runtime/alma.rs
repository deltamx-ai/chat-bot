use std::process::Stdio;
use std::time::Duration;

use async_trait::async_trait;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio::time::timeout;

use super::super::error::ProviderError;
use super::{ChatRole, ChatTurn, GenerateRequest, GenerateResponse, Provider, compose_prompt};

const DEFAULT_TIMEOUT_SECS: u64 = 120;

pub struct AlmaProvider {
    timeout: Duration,
}

impl AlmaProvider {
    pub fn with_timeout(timeout: Duration) -> Self {
        Self { timeout }
    }
}

impl Default for AlmaProvider {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
        }
    }
}

#[async_trait]
impl Provider for AlmaProvider {
    async fn generate(
        &self,
        request: &GenerateRequest<'_>,
    ) -> Result<GenerateResponse, ProviderError> {
        let new_turn = compose_prompt(request.prompt, request.attachments);
        let prompt = compose_history_prompt(request.history, &new_turn);
        let content = run_alma(request.model_id, &prompt, self.timeout).await?;
        Ok(GenerateResponse { content })
    }
}

fn compose_history_prompt(history: &[ChatTurn], new_user_prompt: &str) -> String {
    let mut sections: Vec<String> = Vec::with_capacity(history.len() + 1);
    for turn in history {
        if matches!(turn.role, ChatRole::Tool) {
            continue;
        }
        if turn.content.trim().is_empty() {
            continue;
        }
        let label = match turn.role {
            ChatRole::System => "System",
            ChatRole::User => "User",
            ChatRole::Assistant => "Assistant",
            ChatRole::Tool => continue,
        };
        sections.push(format!("{label}: {}", turn.content));
    }
    sections.push(format!("User: {new_user_prompt}"));
    sections.join("\n\n")
}

async fn run_alma(
    model_id: &str,
    prompt: &str,
    deadline: Duration,
) -> Result<String, ProviderError> {
    let mut child = Command::new("alma")
        .arg("run")
        .arg("--no-stream")
        .arg("--raw")
        .arg("-m")
        .arg(model_id)
        .arg(prompt)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|err| ProviderError::Spawn(err.to_string()))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| ProviderError::Io("stdout pipe not captured".into()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| ProviderError::Io("stderr pipe not captured".into()))?;

    let collect = async {
        let (stdout_str, stderr_str) =
            tokio::try_join!(read_to_string(stdout), read_to_string(stderr))?;
        let status = child
            .wait()
            .await
            .map_err(|err| ProviderError::Wait(err.to_string()))?;
        Ok::<_, ProviderError>((stdout_str, stderr_str, status))
    };

    let (stdout_str, stderr_str, status) = match timeout(deadline, collect).await {
        Ok(result) => result?,
        Err(_) => return Err(ProviderError::Timeout(deadline)),
    };

    if !status.success() {
        return Err(ProviderError::NonZeroExit {
            code: status.code(),
            stderr: stderr_str,
        });
    }

    let reply = stdout_str.trim().to_string();
    if reply.is_empty() {
        return Err(ProviderError::EmptyOutput);
    }
    Ok(reply)
}

async fn read_to_string<R>(mut reader: R) -> Result<String, ProviderError>
where
    R: AsyncReadExt + Unpin,
{
    let mut buf = String::new();
    reader
        .read_to_string(&mut buf)
        .await
        .map_err(|err| ProviderError::Io(err.to_string()))?;
    Ok(buf)
}
