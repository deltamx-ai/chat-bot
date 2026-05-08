use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::{
    ConfirmationPolicy, ExecutionContext, InputMode, ShellOutput, Tool, ToolError, ToolOutput,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShellCommand {
    pub command: String,
    pub cwd: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub requires_confirmation: bool,
    pub stdin_mode: String,
}

pub struct ShellTool;

impl Tool for ShellTool {
    fn name(&self) -> &str {
        "shell"
    }

    fn description(&self) -> &str {
        "Execute a shell command with optional confirmation and input requirements."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": { "type": "string" },
                "cwd": { "type": ["string", "null"] },
                "timeout_seconds": { "type": ["integer", "null"] },
                "requires_confirmation": { "type": "boolean" },
                "stdin_mode": { "type": "string", "enum": ["none", "stdin"] }
            },
            "required": ["command"]
        })
    }

    fn call(&self, _ctx: ExecutionContext, input: Value) -> Result<ToolOutput, ToolError> {
        let command: ShellCommand = serde_json::from_value(input).map_err(|err| ToolError {
            code: "invalid_shell_input".into(),
            message: err.to_string(),
        })?;

        if command.requires_confirmation {
            return Err(ToolError {
                code: "confirmation_required".into(),
                message: "shell command requires user confirmation".into(),
            });
        }

        if command.stdin_mode == "stdin" {
            return Err(ToolError {
                code: "input_required".into(),
                message: "shell command requires stdin input before execution".into(),
            });
        }

        let mut cmd = Command::new("sh");
        cmd.arg("-lc").arg(&command.command);
        if let Some(cwd) = command.cwd.as_deref() {
            cmd.current_dir(cwd);
        }

        let output = cmd.output().map_err(|err| ToolError {
            code: "shell_spawn_failed".into(),
            message: err.to_string(),
        })?;

        let shell_output = ShellOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            success: output.status.success(),
        };

        Ok(ToolOutput {
            content: serde_json::to_value(shell_output).map_err(|err| ToolError {
                code: "shell_output_serialize_failed".into(),
                message: err.to_string(),
            })?,
        })
    }
}

pub fn shell_confirmation_policy(command: &ShellCommand) -> ConfirmationPolicy {
    if command.requires_confirmation {
        ConfirmationPolicy::Always
    } else {
        ConfirmationPolicy::Never
    }
}

pub fn shell_input_mode(command: &ShellCommand) -> InputMode {
    if command.stdin_mode == "stdin" {
        InputMode::Stdin
    } else {
        InputMode::None
    }
}
