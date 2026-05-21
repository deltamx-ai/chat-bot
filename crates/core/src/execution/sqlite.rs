use std::path::Path;
use std::str::FromStr;

use chrono::Utc;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};

use crate::conversation::ConversationId;

use super::{
    ApprovalRequest, ApprovalRequestId, ApprovalRequestKind, ApprovalRequestStatus, EventId, Run,
    RunId, RunStatus, StepAction, StepId, StepStatus, Task, TaskError, TaskEvent, TaskEventKind,
    TaskId, TaskPriority, TaskStatus, TaskStep, Teammate, TeammateId, TeammateMessage,
    TeammateMessageId, TeammateMessageStatus, TeammateStatus,
};

const MIGRATION_SQL: &str = concat!(
    include_str!("../../migrations/0001_init.sql"),
    "\n",
    include_str!("../../migrations/0002_task_run_approval.sql"),
);

#[derive(Clone)]
pub struct SqliteExecutionStore {
    pool: SqlitePool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::{StepAction, StepStatus, TaskKind};

    #[tokio::test]
    async fn task_steps_round_trip() {
        let store = SqliteExecutionStore::open_in_memory().await.unwrap();
        let task = Task::draft(
            "task_steps_1",
            ConversationId("conv_steps".into()),
            TaskKind::Feature,
            "Task with steps",
            "Persist steps",
            serde_json::Value::Null,
        )
        .with_steps(vec![
            TaskStep {
                id: StepId("step_1".into()),
                task_id: TaskId("task_steps_1".into()),
                index: 0,
                title: "Read file".into(),
                action: StepAction::Read,
                tool_name: "read".into(),
                status: StepStatus::Succeeded,
                input: serde_json::json!({ "path": "README.md" }),
                output: Some(serde_json::json!({ "ok": true })),
                error: None,
                depends_on: vec![],
                created_at: String::new(),
                started_at: None,
                finished_at: None,
            },
            TaskStep {
                id: StepId("step_2".into()),
                task_id: TaskId("task_steps_1".into()),
                index: 1,
                title: "Validate".into(),
                action: StepAction::Validate,
                tool_name: "validate".into(),
                status: StepStatus::Pending,
                input: serde_json::json!({ "target": "plan" }),
                output: None,
                error: None,
                depends_on: vec![StepId("step_1".into())],
                created_at: String::new(),
                started_at: None,
                finished_at: None,
            },
        ]);

        store.save_task(task).await.unwrap();
        let loaded = store
            .load_task(&TaskId("task_steps_1".into()))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(loaded.steps.len(), 2);
        assert_eq!(loaded.steps[0].title, "Read file");
        assert_eq!(loaded.steps[0].status, StepStatus::Succeeded);
        assert_eq!(loaded.steps[1].depends_on, vec![StepId("step_1".into())]);
    }
}

impl std::fmt::Debug for SqliteExecutionStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SqliteExecutionStore")
            .finish_non_exhaustive()
    }
}

impl SqliteExecutionStore {
    pub async fn open_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|err| err.to_string())?;
        }
        let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))
            .map_err(|err| err.to_string())?
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(8)
            .connect_with(options)
            .await
            .map_err(|err| err.to_string())?;
        sqlx::raw_sql(MIGRATION_SQL)
            .execute(&pool)
            .await
            .map_err(|err| err.to_string())?;
        Ok(Self { pool })
    }

    pub async fn open_in_memory() -> Result<Self, String> {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .map_err(|err| err.to_string())?
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .map_err(|err| err.to_string())?;
        sqlx::raw_sql(MIGRATION_SQL)
            .execute(&pool)
            .await
            .map_err(|err| err.to_string())?;
        Ok(Self { pool })
    }

    fn now_rfc3339() -> String {
        Utc::now().to_rfc3339()
    }

    fn task_status_to_str(status: &TaskStatus) -> &'static str {
        match status {
            TaskStatus::Draft => "Draft",
            TaskStatus::Pending => "Pending",
            TaskStatus::Running => "Running",
            TaskStatus::Blocked => "Blocked",
            TaskStatus::Succeeded => "Succeeded",
            TaskStatus::Failed => "Failed",
            TaskStatus::Cancelled => "Cancelled",
            TaskStatus::Archived => "Archived",
        }
    }

    fn task_status_from_str(raw: &str) -> TaskStatus {
        match raw {
            "Pending" => TaskStatus::Pending,
            "Running" => TaskStatus::Running,
            "Blocked" => TaskStatus::Blocked,
            "Succeeded" => TaskStatus::Succeeded,
            "Failed" => TaskStatus::Failed,
            "Cancelled" => TaskStatus::Cancelled,
            "Archived" => TaskStatus::Archived,
            _ => TaskStatus::Draft,
        }
    }

    fn task_priority_to_str(priority: &TaskPriority) -> &'static str {
        match priority {
            TaskPriority::Low => "Low",
            TaskPriority::Normal => "Normal",
            TaskPriority::High => "High",
            TaskPriority::Urgent => "Urgent",
        }
    }

    fn task_priority_from_str(raw: &str) -> TaskPriority {
        match raw {
            "Low" => TaskPriority::Low,
            "High" => TaskPriority::High,
            "Urgent" => TaskPriority::Urgent,
            _ => TaskPriority::Normal,
        }
    }

    fn run_status_to_str(status: &RunStatus) -> &'static str {
        match status {
            RunStatus::Pending => "Pending",
            RunStatus::Running => "Running",
            RunStatus::WaitingApproval => "WaitingApproval",
            RunStatus::Succeeded => "Succeeded",
            RunStatus::Failed => "Failed",
            RunStatus::Cancelled => "Cancelled",
        }
    }

    fn run_status_from_str(raw: &str) -> RunStatus {
        match raw {
            "Running" => RunStatus::Running,
            "WaitingApproval" => RunStatus::WaitingApproval,
            "Succeeded" => RunStatus::Succeeded,
            "Failed" => RunStatus::Failed,
            "Cancelled" => RunStatus::Cancelled,
            _ => RunStatus::Pending,
        }
    }

    fn approval_kind_to_str(kind: &ApprovalRequestKind) -> &'static str {
        match kind {
            ApprovalRequestKind::PlanApproval => "PlanApproval",
            ApprovalRequestKind::ToolApproval => "ToolApproval",
        }
    }

    fn approval_kind_from_str(raw: &str) -> ApprovalRequestKind {
        match raw {
            "ToolApproval" => ApprovalRequestKind::ToolApproval,
            _ => ApprovalRequestKind::PlanApproval,
        }
    }

    fn approval_status_to_str(status: &ApprovalRequestStatus) -> &'static str {
        match status {
            ApprovalRequestStatus::Pending => "Pending",
            ApprovalRequestStatus::Approved => "Approved",
            ApprovalRequestStatus::Rejected => "Rejected",
        }
    }

    fn approval_status_from_str(raw: &str) -> ApprovalRequestStatus {
        match raw {
            "Approved" => ApprovalRequestStatus::Approved,
            "Rejected" => ApprovalRequestStatus::Rejected,
            _ => ApprovalRequestStatus::Pending,
        }
    }

    fn teammate_status_to_str(status: &TeammateStatus) -> &'static str {
        match status {
            TeammateStatus::Idle => "Idle",
            TeammateStatus::Working => "Working",
            TeammateStatus::Shutdown => "Shutdown",
        }
    }

    fn teammate_status_from_str(raw: &str) -> TeammateStatus {
        match raw {
            "Working" => TeammateStatus::Working,
            "Shutdown" => TeammateStatus::Shutdown,
            _ => TeammateStatus::Idle,
        }
    }

    fn teammate_message_status_to_str(status: &TeammateMessageStatus) -> &'static str {
        match status {
            TeammateMessageStatus::Unread => "Unread",
            TeammateMessageStatus::Read => "Read",
        }
    }

    fn teammate_message_status_from_str(raw: &str) -> TeammateMessageStatus {
        match raw {
            "Read" => TeammateMessageStatus::Read,
            _ => TeammateMessageStatus::Unread,
        }
    }

    fn step_action_to_str(action: &StepAction) -> String {
        match action {
            StepAction::Read => "Read".into(),
            StepAction::Search => "Search".into(),
            StepAction::Write => "Write".into(),
            StepAction::Validate => "Validate".into(),
            StepAction::Plan => "Plan".into(),
            StepAction::Shell => "Shell".into(),
            StepAction::Custom(value) => format!("Custom:{value}"),
        }
    }

    fn step_action_from_str(raw: &str) -> StepAction {
        match raw {
            "Read" => StepAction::Read,
            "Search" => StepAction::Search,
            "Write" => StepAction::Write,
            "Validate" => StepAction::Validate,
            "Plan" => StepAction::Plan,
            "Shell" => StepAction::Shell,
            _ if raw.starts_with("Custom:") => StepAction::Custom(raw[7..].to_string()),
            other => StepAction::Custom(other.to_string()),
        }
    }

    fn step_status_to_str(status: &StepStatus) -> &'static str {
        match status {
            StepStatus::Pending => "Pending",
            StepStatus::Ready => "Ready",
            StepStatus::Running => "Running",
            StepStatus::AwaitingConfirmation => "AwaitingConfirmation",
            StepStatus::AwaitingInput => "AwaitingInput",
            StepStatus::Succeeded => "Succeeded",
            StepStatus::Failed => "Failed",
            StepStatus::Skipped => "Skipped",
            StepStatus::Cancelled => "Cancelled",
        }
    }

    fn step_status_from_str(raw: &str) -> StepStatus {
        match raw {
            "Ready" => StepStatus::Ready,
            "Running" => StepStatus::Running,
            "AwaitingConfirmation" => StepStatus::AwaitingConfirmation,
            "AwaitingInput" => StepStatus::AwaitingInput,
            "Succeeded" => StepStatus::Succeeded,
            "Failed" => StepStatus::Failed,
            "Skipped" => StepStatus::Skipped,
            "Cancelled" => StepStatus::Cancelled,
            _ => StepStatus::Pending,
        }
    }

    fn event_kind_to_str(kind: &TaskEventKind) -> &'static str {
        match kind {
            TaskEventKind::TaskCreated => "TaskCreated",
            TaskEventKind::TaskStarted => "TaskStarted",
            TaskEventKind::TaskBlocked => "TaskBlocked",
            TaskEventKind::TaskSucceeded => "TaskSucceeded",
            TaskEventKind::TaskFailed => "TaskFailed",
            TaskEventKind::TaskCancelled => "TaskCancelled",
            TaskEventKind::StepReady => "StepReady",
            TaskEventKind::StepStarted => "StepStarted",
            TaskEventKind::StepSucceeded => "StepSucceeded",
            TaskEventKind::StepFailed => "StepFailed",
            TaskEventKind::StepSkipped => "StepSkipped",
            TaskEventKind::ArtifactProduced => "ArtifactProduced",
            TaskEventKind::RetryScheduled => "RetryScheduled",
        }
    }

    fn event_kind_from_str(raw: &str) -> TaskEventKind {
        match raw {
            "TaskStarted" => TaskEventKind::TaskStarted,
            "TaskBlocked" => TaskEventKind::TaskBlocked,
            "TaskSucceeded" => TaskEventKind::TaskSucceeded,
            "TaskFailed" => TaskEventKind::TaskFailed,
            "TaskCancelled" => TaskEventKind::TaskCancelled,
            "StepReady" => TaskEventKind::StepReady,
            "StepStarted" => TaskEventKind::StepStarted,
            "StepSucceeded" => TaskEventKind::StepSucceeded,
            "StepFailed" => TaskEventKind::StepFailed,
            "StepSkipped" => TaskEventKind::StepSkipped,
            "ArtifactProduced" => TaskEventKind::ArtifactProduced,
            "RetryScheduled" => TaskEventKind::RetryScheduled,
            _ => TaskEventKind::TaskCreated,
        }
    }

    fn encode_task_error(error: &Option<TaskError>) -> Result<Option<String>, String> {
        error
            .as_ref()
            .map(|value| serde_json::to_string(value).map_err(|err| err.to_string()))
            .transpose()
    }

    fn decode_task_error(value: Option<String>) -> Result<Option<TaskError>, String> {
        value
            .map(|raw| serde_json::from_str(&raw).map_err(|err| err.to_string()))
            .transpose()
    }

    async fn replace_task_steps(&self, task_id: &TaskId, steps: &[TaskStep]) -> Result<(), String> {
        sqlx::query("DELETE FROM task_steps WHERE task_id = ?")
            .bind(&task_id.0)
            .execute(&self.pool)
            .await
            .map_err(|err| err.to_string())?;

        for step in steps {
            sqlx::query(
                "INSERT INTO task_steps (id, task_id, step_index, title, action, tool_name, status, input, output, error, depends_on, created_at, started_at, finished_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(&step.id.0)
            .bind(&step.task_id.0)
            .bind(i64::from(step.index))
            .bind(&step.title)
            .bind(Self::step_action_to_str(&step.action))
            .bind(&step.tool_name)
            .bind(Self::step_status_to_str(&step.status))
            .bind(serde_json::to_string(&step.input).map_err(|err| err.to_string())?)
            .bind(step.output.as_ref().map(serde_json::to_string).transpose().map_err(|err| err.to_string())?)
            .bind(Self::encode_task_error(&step.error)?)
            .bind(serde_json::to_string(&step.depends_on.iter().map(|id| id.0.clone()).collect::<Vec<_>>()).map_err(|err| err.to_string())?)
            .bind(if step.created_at.is_empty() { Self::now_rfc3339() } else { step.created_at.clone() })
            .bind(step.started_at.as_deref())
            .bind(step.finished_at.as_deref())
            .execute(&self.pool)
            .await
            .map_err(|err| err.to_string())?;
        }

        Ok(())
    }

    async fn load_task_steps(&self, task_id: &TaskId) -> Result<Vec<TaskStep>, String> {
        let rows = sqlx::query(
            "SELECT id, task_id, step_index, title, action, tool_name, status, input, output, error, depends_on, created_at, started_at, finished_at
             FROM task_steps WHERE task_id = ? ORDER BY step_index ASC"
        )
        .bind(&task_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|err| err.to_string())?;

        let mut steps = Vec::with_capacity(rows.len());
        for row in rows {
            let input: String = row.try_get("input").map_err(|err| err.to_string())?;
            let output: Option<String> = row.try_get("output").map_err(|err| err.to_string())?;
            let depends_on: String = row.try_get("depends_on").map_err(|err| err.to_string())?;
            let depends_on_ids: Vec<String> =
                serde_json::from_str(&depends_on).map_err(|err| err.to_string())?;
            steps.push(TaskStep {
                id: StepId(
                    row.try_get::<String, _>("id")
                        .map_err(|err| err.to_string())?,
                ),
                task_id: TaskId(
                    row.try_get::<String, _>("task_id")
                        .map_err(|err| err.to_string())?,
                ),
                index: row
                    .try_get::<i64, _>("step_index")
                    .map_err(|err| err.to_string())? as u32,
                title: row.try_get("title").map_err(|err| err.to_string())?,
                action: Self::step_action_from_str(
                    &row.try_get::<String, _>("action")
                        .map_err(|err| err.to_string())?,
                ),
                tool_name: row.try_get("tool_name").map_err(|err| err.to_string())?,
                status: Self::step_status_from_str(
                    &row.try_get::<String, _>("status")
                        .map_err(|err| err.to_string())?,
                ),
                input: serde_json::from_str(&input).map_err(|err| err.to_string())?,
                output: output
                    .map(|raw| serde_json::from_str(&raw).map_err(|err| err.to_string()))
                    .transpose()?,
                error: Self::decode_task_error(
                    row.try_get("error").map_err(|err| err.to_string())?,
                )?,
                depends_on: depends_on_ids.into_iter().map(StepId).collect(),
                created_at: row.try_get("created_at").map_err(|err| err.to_string())?,
                started_at: row.try_get("started_at").map_err(|err| err.to_string())?,
                finished_at: row.try_get("finished_at").map_err(|err| err.to_string())?,
            });
        }
        Ok(steps)
    }

    pub async fn save_task(&self, task: Task) -> Result<(), String> {
        let now = Self::now_rfc3339();
        let created_at = if task.created_at.is_empty() {
            now.clone()
        } else {
            task.created_at.clone()
        };
        let updated_at = if task.updated_at.is_empty() {
            now.clone()
        } else {
            task.updated_at.clone()
        };
        sqlx::query(
            "INSERT INTO tasks (id, conversation_id, parent_task_id, plan_id, kind, title, goal, status, priority, assignee_kind, assignee_id, input, output, error, retry_count, max_retries, tags, created_at, updated_at, started_at, finished_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                goal = excluded.goal,
                status = excluded.status,
                priority = excluded.priority,
                output = excluded.output,
                error = excluded.error,
                retry_count = excluded.retry_count,
                max_retries = excluded.max_retries,
                tags = excluded.tags,
                started_at = excluded.started_at,
                finished_at = excluded.finished_at,
                updated_at = excluded.updated_at"
        )
        .bind(&task.id.0)
        .bind(&task.conversation_id.0)
        .bind(task.parent_task_id.as_ref().map(|id| id.0.as_str()))
        .bind(task.plan_id.as_ref().map(|id| id.0.as_str()))
        .bind(format!("{:?}", task.kind))
        .bind(&task.title)
        .bind(&task.goal)
        .bind(Self::task_status_to_str(&task.status))
        .bind(Self::task_priority_to_str(&task.priority))
        .bind(task.assignee.as_ref().map(|assignee| format!("{:?}", assignee.kind)))
        .bind(task.assignee.as_ref().map(|assignee| assignee.id.as_str()))
        .bind(serde_json::to_string(&task.input).map_err(|err| err.to_string())?)
        .bind(task.output.as_ref().map(serde_json::to_string).transpose().map_err(|err| err.to_string())?)
        .bind(Self::encode_task_error(&task.error)?)
        .bind(i64::from(task.retry_count))
        .bind(i64::from(task.max_retries))
        .bind(serde_json::to_string(&task.tags).map_err(|err| err.to_string())?)
        .bind(&created_at)
        .bind(&updated_at)
        .bind(task.started_at.as_deref())
        .bind(task.finished_at.as_deref())
        .execute(&self.pool)
        .await
        .map_err(|err| err.to_string())?;
        self.replace_task_steps(&task.id, &task.steps).await?;
        Ok(())
    }

    pub async fn load_task(&self, id: &TaskId) -> Result<Option<Task>, String> {
        let row = sqlx::query(
            "SELECT id, conversation_id, parent_task_id, plan_id, kind, title, goal, status, priority, input, output, error, retry_count, max_retries, tags, created_at, updated_at, started_at, finished_at
             FROM tasks WHERE id = ?"
        )
        .bind(&id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| err.to_string())?;
        let Some(row) = row else { return Ok(None) };
        let input: String = row.try_get("input").map_err(|err| err.to_string())?;
        let output: Option<String> = row.try_get("output").map_err(|err| err.to_string())?;
        let tags: String = row.try_get("tags").map_err(|err| err.to_string())?;
        let steps = self.load_task_steps(id).await?;
        Ok(Some(Task {
            id: TaskId(
                row.try_get::<String, _>("id")
                    .map_err(|err| err.to_string())?,
            ),
            conversation_id: ConversationId(
                row.try_get::<String, _>("conversation_id")
                    .map_err(|err| err.to_string())?,
            ),
            parent_task_id: row
                .try_get::<Option<String>, _>("parent_task_id")
                .map_err(|err| err.to_string())?
                .map(TaskId),
            plan_id: row
                .try_get::<Option<String>, _>("plan_id")
                .map_err(|err| err.to_string())?
                .map(crate::planning::PlanId),
            kind: match row
                .try_get::<String, _>("kind")
                .map_err(|err| err.to_string())?
                .as_str()
            {
                "Bugfix" => super::TaskKind::Bugfix,
                "Feature" => super::TaskKind::Feature,
                "Research" => super::TaskKind::Research,
                "Refactor" => super::TaskKind::Refactor,
                "Validate" => super::TaskKind::Validate,
                "Write" => super::TaskKind::Write,
                other => super::TaskKind::Custom(other.to_string()),
            },
            title: row.try_get("title").map_err(|err| err.to_string())?,
            goal: row.try_get("goal").map_err(|err| err.to_string())?,
            status: Self::task_status_from_str(
                &row.try_get::<String, _>("status")
                    .map_err(|err| err.to_string())?,
            ),
            priority: Self::task_priority_from_str(
                &row.try_get::<String, _>("priority")
                    .map_err(|err| err.to_string())?,
            ),
            assignee: None,
            steps,
            input: serde_json::from_str(&input).map_err(|err| err.to_string())?,
            output: output
                .map(|raw| serde_json::from_str(&raw).map_err(|err| err.to_string()))
                .transpose()?,
            error: Self::decode_task_error(row.try_get("error").map_err(|err| err.to_string())?)?,
            retry_count: row
                .try_get::<i64, _>("retry_count")
                .map_err(|err| err.to_string())? as u32,
            max_retries: row
                .try_get::<i64, _>("max_retries")
                .map_err(|err| err.to_string())? as u32,
            tags: serde_json::from_str(&tags).map_err(|err| err.to_string())?,
            created_at: row.try_get("created_at").map_err(|err| err.to_string())?,
            updated_at: row.try_get("updated_at").map_err(|err| err.to_string())?,
            started_at: row.try_get("started_at").map_err(|err| err.to_string())?,
            finished_at: row.try_get("finished_at").map_err(|err| err.to_string())?,
        }))
    }

    pub async fn list_tasks(&self) -> Result<Vec<Task>, String> {
        let rows = sqlx::query("SELECT id FROM tasks ORDER BY updated_at DESC")
            .fetch_all(&self.pool)
            .await
            .map_err(|err| err.to_string())?;
        let mut tasks = Vec::with_capacity(rows.len());
        for row in rows {
            let id = TaskId(
                row.try_get::<String, _>("id")
                    .map_err(|err| err.to_string())?,
            );
            if let Some(task) = self.load_task(&id).await? {
                tasks.push(task);
            }
        }
        Ok(tasks)
    }

    pub async fn save_run(&self, run: Run) -> Result<(), String> {
        let now = Self::now_rfc3339();
        let created_at = if run.created_at.is_empty() {
            now.clone()
        } else {
            run.created_at.clone()
        };
        let updated_at = if run.updated_at.is_empty() {
            now.clone()
        } else {
            run.updated_at.clone()
        };
        sqlx::query(
            "INSERT INTO runs (id, task_id, conversation_id, status, current_step_index, error, created_at, updated_at, started_at, finished_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                status = excluded.status,
                current_step_index = excluded.current_step_index,
                error = excluded.error,
                updated_at = excluded.updated_at,
                started_at = excluded.started_at,
                finished_at = excluded.finished_at"
        )
        .bind(&run.id.0)
        .bind(&run.task_id.0)
        .bind(&run.conversation_id.0)
        .bind(Self::run_status_to_str(&run.status))
        .bind(run.current_step_index.map(i64::from))
        .bind(Self::encode_task_error(&run.error)?)
        .bind(&created_at)
        .bind(&updated_at)
        .bind(run.started_at.as_deref())
        .bind(run.finished_at.as_deref())
        .execute(&self.pool)
        .await
        .map_err(|err| err.to_string())?;
        Ok(())
    }

    pub async fn load_run(&self, id: &RunId) -> Result<Option<Run>, String> {
        let row = sqlx::query("SELECT id, task_id, conversation_id, status, current_step_index, error, created_at, updated_at, started_at, finished_at FROM runs WHERE id = ?")
            .bind(&id.0)
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| err.to_string())?;
        let Some(row) = row else { return Ok(None) };
        Ok(Some(Run {
            id: RunId(row.try_get("id").map_err(|err| err.to_string())?),
            task_id: TaskId(row.try_get("task_id").map_err(|err| err.to_string())?),
            conversation_id: ConversationId(
                row.try_get("conversation_id")
                    .map_err(|err| err.to_string())?,
            ),
            status: Self::run_status_from_str(
                &row.try_get::<String, _>("status")
                    .map_err(|err| err.to_string())?,
            ),
            current_step_index: row
                .try_get::<Option<i64>, _>("current_step_index")
                .map_err(|err| err.to_string())?
                .map(|value| value as u32),
            error: Self::decode_task_error(row.try_get("error").map_err(|err| err.to_string())?)?,
            created_at: row.try_get("created_at").map_err(|err| err.to_string())?,
            updated_at: row.try_get("updated_at").map_err(|err| err.to_string())?,
            started_at: row.try_get("started_at").map_err(|err| err.to_string())?,
            finished_at: row.try_get("finished_at").map_err(|err| err.to_string())?,
        }))
    }

    pub async fn list_runs_by_task(&self, task_id: &TaskId) -> Result<Vec<Run>, String> {
        let rows = sqlx::query("SELECT id FROM runs WHERE task_id = ? ORDER BY created_at DESC")
            .bind(&task_id.0)
            .fetch_all(&self.pool)
            .await
            .map_err(|err| err.to_string())?;
        let mut runs = Vec::with_capacity(rows.len());
        for row in rows {
            let id = RunId(
                row.try_get::<String, _>("id")
                    .map_err(|err| err.to_string())?,
            );
            if let Some(run) = self.load_run(&id).await? {
                runs.push(run);
            }
        }
        Ok(runs)
    }

    pub async fn save_approval_request(&self, request: ApprovalRequest) -> Result<(), String> {
        sqlx::query(
            "INSERT INTO approval_requests (id, task_id, run_id, step_id, kind, status, title, payload, decision_note, created_at, resolved_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                status = excluded.status,
                decision_note = excluded.decision_note,
                payload = excluded.payload,
                resolved_at = excluded.resolved_at"
        )
        .bind(&request.id.0)
        .bind(&request.task_id.0)
        .bind(&request.run_id.0)
        .bind(request.step_id.as_ref().map(|step| step.0.as_str()))
        .bind(Self::approval_kind_to_str(&request.kind))
        .bind(Self::approval_status_to_str(&request.status))
        .bind(&request.title)
        .bind(serde_json::to_string(&request.payload).map_err(|err| err.to_string())?)
        .bind(request.decision_note.as_deref())
        .bind(&request.created_at)
        .bind(request.resolved_at.as_deref())
        .execute(&self.pool)
        .await
        .map_err(|err| err.to_string())?;
        Ok(())
    }

    pub async fn load_approval_request(
        &self,
        id: &ApprovalRequestId,
    ) -> Result<Option<ApprovalRequest>, String> {
        let row = sqlx::query("SELECT id, task_id, run_id, step_id, kind, status, title, payload, decision_note, created_at, resolved_at FROM approval_requests WHERE id = ?")
            .bind(&id.0)
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| err.to_string())?;
        let Some(row) = row else { return Ok(None) };
        let payload: String = row.try_get("payload").map_err(|err| err.to_string())?;
        Ok(Some(ApprovalRequest {
            id: ApprovalRequestId(row.try_get("id").map_err(|err| err.to_string())?),
            task_id: TaskId(row.try_get("task_id").map_err(|err| err.to_string())?),
            run_id: RunId(row.try_get("run_id").map_err(|err| err.to_string())?),
            step_id: row
                .try_get::<Option<String>, _>("step_id")
                .map_err(|err| err.to_string())?
                .map(StepId),
            kind: Self::approval_kind_from_str(
                &row.try_get::<String, _>("kind")
                    .map_err(|err| err.to_string())?,
            ),
            status: Self::approval_status_from_str(
                &row.try_get::<String, _>("status")
                    .map_err(|err| err.to_string())?,
            ),
            title: row.try_get("title").map_err(|err| err.to_string())?,
            payload: serde_json::from_str(&payload).map_err(|err| err.to_string())?,
            decision_note: row
                .try_get("decision_note")
                .map_err(|err| err.to_string())?,
            created_at: row.try_get("created_at").map_err(|err| err.to_string())?,
            resolved_at: row.try_get("resolved_at").map_err(|err| err.to_string())?,
        }))
    }

    pub async fn list_approval_requests(&self) -> Result<Vec<ApprovalRequest>, String> {
        let rows = sqlx::query("SELECT id FROM approval_requests ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await
            .map_err(|err| err.to_string())?;
        let mut approvals = Vec::with_capacity(rows.len());
        for row in rows {
            let id = ApprovalRequestId(
                row.try_get::<String, _>("id")
                    .map_err(|err| err.to_string())?,
            );
            if let Some(item) = self.load_approval_request(&id).await? {
                approvals.push(item);
            }
        }
        Ok(approvals)
    }

    pub async fn save_teammate(&self, teammate: Teammate) -> Result<(), String> {
        let now = Self::now_rfc3339();
        let created_at = if teammate.created_at.is_empty() {
            now.clone()
        } else {
            teammate.created_at.clone()
        };
        let updated_at = if teammate.updated_at.is_empty() {
            now.clone()
        } else {
            teammate.updated_at.clone()
        };
        sqlx::query(
            "INSERT INTO teammates (id, name, role, status, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                role = excluded.role,
                status = excluded.status,
                updated_at = excluded.updated_at",
        )
        .bind(&teammate.id.0)
        .bind(&teammate.name)
        .bind(&teammate.role)
        .bind(Self::teammate_status_to_str(&teammate.status))
        .bind(&created_at)
        .bind(&updated_at)
        .execute(&self.pool)
        .await
        .map_err(|err| err.to_string())?;
        Ok(())
    }

    pub async fn list_teammates(&self) -> Result<Vec<Teammate>, String> {
        let rows = sqlx::query("SELECT id, name, role, status, created_at, updated_at FROM teammates ORDER BY created_at ASC")
            .fetch_all(&self.pool)
            .await
            .map_err(|err| err.to_string())?;
        let mut teammates = Vec::with_capacity(rows.len());
        for row in rows {
            teammates.push(Teammate {
                id: TeammateId(row.try_get("id").map_err(|err| err.to_string())?),
                name: row.try_get("name").map_err(|err| err.to_string())?,
                role: row.try_get("role").map_err(|err| err.to_string())?,
                status: Self::teammate_status_from_str(
                    &row.try_get::<String, _>("status")
                        .map_err(|err| err.to_string())?,
                ),
                created_at: row.try_get("created_at").map_err(|err| err.to_string())?,
                updated_at: row.try_get("updated_at").map_err(|err| err.to_string())?,
            });
        }
        Ok(teammates)
    }

    pub async fn save_teammate_message(&self, message: TeammateMessage) -> Result<(), String> {
        sqlx::query(
            "INSERT INTO teammate_messages (id, teammate_id, from_name, content, status, created_at, read_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                status = excluded.status,
                read_at = excluded.read_at"
        )
        .bind(&message.id.0)
        .bind(&message.teammate_id.0)
        .bind(&message.from_name)
        .bind(&message.content)
        .bind(Self::teammate_message_status_to_str(&message.status))
        .bind(&message.created_at)
        .bind(message.read_at.as_deref())
        .execute(&self.pool)
        .await
        .map_err(|err| err.to_string())?;
        Ok(())
    }

    pub async fn list_teammate_messages(
        &self,
        teammate_id: &TeammateId,
    ) -> Result<Vec<TeammateMessage>, String> {
        let rows = sqlx::query(
            "SELECT id, teammate_id, from_name, content, status, created_at, read_at
             FROM teammate_messages WHERE teammate_id = ? ORDER BY created_at ASC",
        )
        .bind(&teammate_id.0)
        .fetch_all(&self.pool)
        .await
        .map_err(|err| err.to_string())?;
        let mut messages = Vec::with_capacity(rows.len());
        for row in rows {
            messages.push(TeammateMessage {
                id: TeammateMessageId(row.try_get("id").map_err(|err| err.to_string())?),
                teammate_id: TeammateId(row.try_get("teammate_id").map_err(|err| err.to_string())?),
                from_name: row.try_get("from_name").map_err(|err| err.to_string())?,
                content: row.try_get("content").map_err(|err| err.to_string())?,
                status: Self::teammate_message_status_from_str(
                    &row.try_get::<String, _>("status")
                        .map_err(|err| err.to_string())?,
                ),
                created_at: row.try_get("created_at").map_err(|err| err.to_string())?,
                read_at: row.try_get("read_at").map_err(|err| err.to_string())?,
            });
        }
        Ok(messages)
    }

    pub async fn mark_teammate_messages_read(
        &self,
        teammate_id: &TeammateId,
    ) -> Result<(), String> {
        sqlx::query(
            "UPDATE teammate_messages SET status = ?, read_at = ? WHERE teammate_id = ? AND status = ?"
        )
        .bind(Self::teammate_message_status_to_str(&TeammateMessageStatus::Read))
        .bind(Self::now_rfc3339())
        .bind(&teammate_id.0)
        .bind(Self::teammate_message_status_to_str(&TeammateMessageStatus::Unread))
        .execute(&self.pool)
        .await
        .map_err(|err| err.to_string())?;
        Ok(())
    }

    pub async fn append_event(
        &self,
        event: TaskEvent,
        run_id: Option<&RunId>,
    ) -> Result<(), String> {
        let created_at = if event.created_at.is_empty() {
            Self::now_rfc3339()
        } else {
            event.created_at.clone()
        };
        sqlx::query(
            "INSERT INTO run_events (id, run_id, task_id, step_id, kind, payload, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&event.id.0)
        .bind(run_id.map(|id| id.0.as_str()).unwrap_or(""))
        .bind(&event.task_id.0)
        .bind(event.step_id.as_ref().map(|id| id.0.as_str()))
        .bind(Self::event_kind_to_str(&event.kind))
        .bind(serde_json::to_string(&event.payload).map_err(|err| err.to_string())?)
        .bind(&created_at)
        .execute(&self.pool)
        .await
        .map_err(|err| err.to_string())?;
        Ok(())
    }

    pub async fn list_events(&self, task_id: &TaskId) -> Result<Vec<TaskEvent>, String> {
        let rows = sqlx::query("SELECT id, task_id, step_id, kind, payload, created_at FROM run_events WHERE task_id = ? ORDER BY created_at ASC")
            .bind(&task_id.0)
            .fetch_all(&self.pool)
            .await
            .map_err(|err| err.to_string())?;
        let mut events = Vec::with_capacity(rows.len());
        for row in rows {
            let payload: String = row.try_get("payload").map_err(|err| err.to_string())?;
            events.push(TaskEvent {
                id: EventId(row.try_get("id").map_err(|err| err.to_string())?),
                task_id: TaskId(row.try_get("task_id").map_err(|err| err.to_string())?),
                step_id: row
                    .try_get::<Option<String>, _>("step_id")
                    .map_err(|err| err.to_string())?
                    .map(StepId),
                kind: Self::event_kind_from_str(
                    &row.try_get::<String, _>("kind")
                        .map_err(|err| err.to_string())?,
                ),
                payload: serde_json::from_str(&payload).map_err(|err| err.to_string())?,
                created_at: row.try_get("created_at").map_err(|err| err.to_string())?,
            });
        }
        Ok(events)
    }
}
