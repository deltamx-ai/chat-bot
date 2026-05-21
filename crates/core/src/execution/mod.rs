//! Execution flow and task orchestration.

pub mod approval;
pub mod context;
pub mod event;
pub mod reader;
pub mod registry;
pub mod result;
pub mod router;
pub mod run;
pub mod runner;
pub mod search;
pub mod shell;
pub mod sqlite;
pub mod state;
pub mod step;
pub mod store;
pub mod task;
pub mod teammate;
pub mod tool;
pub mod validate;
pub mod writer;

pub use approval::{
    ApprovalRequest, ApprovalRequestId, ApprovalRequestKind, ApprovalRequestStatus,
};
pub use context::ExecutionContext;
pub use event::{EventId, TaskEvent, TaskEventKind};
pub use reader::ReadTool;
pub use registry::ToolRegistry;
pub use result::{ShellOutput, StepResult, TaskError, TaskResult, ToolError, ToolOutput};
pub use router::InMemoryToolRouter;
pub use run::{Run, RunId, RunStatus};
pub use runner::{SequentialTaskRunner, TaskRunner};
pub use search::SearchTool;
pub use shell::{ShellCommand, ShellTool, shell_confirmation_policy, shell_input_mode};
pub use sqlite::SqliteExecutionStore;
pub use state::{StepTransitionError, TaskTransitionError};
pub use step::{StepAction, StepId, StepStatus, TaskStep};
pub use store::{ApprovalStore, EventStore, InMemoryTaskStore, RunStore, TaskStore};
pub use task::{
    AssigneeKind, Task, TaskAssignee, TaskId, TaskKind, TaskPriority, TaskStatus, infer_task_kind,
};
pub use teammate::{
    Teammate, TeammateId, TeammateMessage, TeammateMessageId, TeammateMessageStatus, TeammateStatus,
};
pub use tool::{ConfirmationPolicy, InputMode, Tool};
pub use validate::ValidateTool;
pub use writer::WriteTool;
