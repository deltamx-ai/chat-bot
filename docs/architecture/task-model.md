# Task 模型设计

## 目标
这份文档定义 `chat-bot` 项目中 task 的核心设计，重点回答：

- task 需要哪些字段
- step 需要哪些字段
- task / step 的状态机如何设计
- 状态流转规则如何约束
- 为什么 message、task、event 要分开

目标不是一次把所有未来场景都做完，而是先定出一版足够稳的内核模型，让后续 `planning / execution / artifact / storage / UI timeline` 都有共同语义。

---

## 一、核心设计原则

### 1. message 和 task 分离
- `message` 代表对话内容
- `task` 代表执行对象

用户说一句“修复 tauri dev”只是请求入口。
真正执行时，应创建一个 task，再把执行步骤和状态变化挂在 task 上。

### 2. task 是生命周期对象
task 不是一个普通函数返回值，而是一个有状态机的实体。
它需要能表示：
- 等待执行
- 正在执行
- 失败
- 被阻塞
- 被取消
- 成功完成
- 后续归档

### 3. task 和 step 分层
- `task` 表示整体目标
- `step` 表示执行步骤

这样：
- 顶层 task 给 UI 展示任务卡片
- step 给执行面板和 timeline 展示细节

### 4. event 单独记录
状态变化、工具调用、产物生成，不要只靠最终 `status` 推断。
应该把这些执行痕迹独立记录成 event timeline。

---

## 二、Task 字段设计

## 推荐结构
```rust
pub struct Task {
    pub id: TaskId,
    pub conversation_id: ConversationId,
    pub parent_task_id: Option<TaskId>,
    pub plan_id: Option<PlanId>,
    pub kind: TaskKind,
    pub title: String,
    pub goal: String,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub assignee: Option<TaskAssignee>,
    pub input: serde_json::Value,
    pub output: Option<serde_json::Value>,
    pub error: Option<TaskError>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
}
```

---

## 字段说明

### `id`
任务唯一标识。
建议独立于 conversation id，不要复用消息 id。

### `conversation_id`
表示该 task 属于哪个会话。
后续切换会话、查看历史、筛选任务都会用到。

### `parent_task_id`
用于支持子任务。
复杂目标通常会拆成：
- 顶层 task
- 若干子 task

第一版可以先保留字段，即使暂时只用顶层 task。

### `plan_id`
表示任务是否来自某次规划产物。
如果 `planning` 层会产出 plan，再转成 task，这个字段就很有用。

### `kind`
任务类别。
建议第一版这样定义：

```rust
pub enum TaskKind {
    Bugfix,
    Feature,
    Research,
    Refactor,
    Validate,
    Write,
    Custom(String),
}
```

作用：
- 方便过滤
- 方便 UI 分类
- 方便 planner 选择执行策略

### `title`
简短任务标题，适合列表显示。
比如：
- `修复 tauri dev 启动失败`
- `设计 storage 架构`

### `goal`
更完整的目标描述。
`title` 适合短展示，`goal` 适合作为执行目标与摘要依据。

### `status`
任务状态机核心字段。
后面会单独展开。

### `priority`
任务优先级。
建议：

```rust
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Urgent,
}
```

### `assignee`
表示当前任务由谁执行。
可以是：
- 当前主 runner
- 某个 agent
- 某个 provider

例如：

```rust
pub struct TaskAssignee {
    pub kind: AssigneeKind,
    pub id: String,
}
```

### `input`
任务输入。
第一版建议直接用 `serde_json::Value`，保持灵活。

### `output`
任务最终输出。
同样先用 `Option<serde_json::Value>`。

### `error`
任务错误对象。
不要只存字符串。
建议：

```rust
pub struct TaskError {
    pub code: String,
    pub message: String,
    pub detail: Option<String>,
    pub retriable: bool,
}
```

### `retry_count` / `max_retries`
用于重试控制。
后续 runner 会根据 `error.retriable` 和重试次数决定是否重试。

### `tags`
用于简单过滤和搜索。
例如：
- `tauri`
- `rust`
- `auth`
- `storage`

### 时间字段
- `created_at`：创建时间
- `updated_at`：最后更新时间
- `started_at`：开始执行时间
- `finished_at`：结束时间

这几个字段对 timeline、持续时间统计、恢复执行都很有用。

---

## 三、TaskStep 字段设计

## 推荐结构
```rust
pub struct TaskStep {
    pub id: StepId,
    pub task_id: TaskId,
    pub index: u32,
    pub title: String,
    pub action: StepAction,
    pub tool_name: String,
    pub status: StepStatus,
    pub input: serde_json::Value,
    pub output: Option<serde_json::Value>,
    pub error: Option<TaskError>,
    pub depends_on: Vec<StepId>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
}
```

---

## 字段说明

### `task_id`
step 必须明确归属于某个 task。

### `index`
用于顺序展示和默认执行顺序。
即使有 `depends_on`，`index` 依然适合 UI 展示。

### `title`
步骤标题。
例如：
- `读取 apps/package.json`
- `检查 src-tauri Cargo.toml`
- `运行 cargo check`

### `action`
描述步骤的语义动作。
建议：

```rust
pub enum StepAction {
    Read,
    Search,
    Write,
    Validate,
    Plan,
    Custom(String),
}
```

### `tool_name`
路由到哪个 tool。
比如：
- `read_file`
- `search_code`
- `write_file`
- `validate_workspace`

### `status`
step 的执行状态。
比 task 稍微更细。

### `depends_on`
依赖的 step id。
这样 future 可以支持：
- 严格顺序执行
- 简单 DAG 执行
- 并行准备后汇总

第一版即使先顺序执行，也建议字段先留着。

---

## 四、Task 状态机设计

## 推荐状态
```rust
pub enum TaskStatus {
    Draft,
    Pending,
    Running,
    Blocked,
    Succeeded,
    Failed,
    Cancelled,
    Archived,
}
```

---

## 各状态语义

### `Draft`
任务已创建，但还没正式进入调度。
适合：
- planner 初次生成任务
- 用户确认前的任务草稿

### `Pending`
任务已准备好，等待执行。

### `Running`
任务正在执行。
至少有一个 step 正在处理，或者 runner 已接管。

### `Blocked`
任务被阻塞。
例如：
- 缺权限
- 缺依赖
- 缺用户确认
- 等待外部条件

### `Succeeded`
任务成功完成。

### `Failed`
任务执行失败。
表示当前执行已经中断且未成功恢复。

### `Cancelled`
任务被主动取消。

### `Archived`
任务结束后进入归档态。
用于从 active 列表移出，但保留记录。

---

## 五、Step 状态机设计

## 推荐状态
```rust
pub enum StepStatus {
    Pending,
    Ready,
    Running,
    Succeeded,
    Failed,
    Skipped,
    Cancelled,
}
```

---

## 各状态语义

### `Pending`
step 已创建，但依赖尚未满足。

### `Ready`
step 依赖满足，可以开始执行。

### `Running`
step 当前正在执行。

### `Succeeded`
step 成功完成。

### `Failed`
step 执行失败。

### `Skipped`
该 step 被跳过。
常见原因：
- 上游结果已满足目标
- 分支策略不再需要执行此步骤

### `Cancelled`
该 step 因任务取消或人工中断而停止。

---

## 六、状态流转规则

## Task 状态流转
推荐：

```text
Draft -> Pending
Draft -> Cancelled

Pending -> Running
Pending -> Cancelled

Running -> Succeeded
Running -> Failed
Running -> Blocked
Running -> Cancelled

Blocked -> Pending
Blocked -> Cancelled

Failed -> Pending      (retry)

Succeeded -> Archived
Failed -> Archived
Cancelled -> Archived
```

---

## Step 状态流转
推荐：

```text
Pending -> Ready
Ready -> Running
Ready -> Skipped
Ready -> Cancelled

Running -> Succeeded
Running -> Failed
Running -> Cancelled

Failed -> Ready        (retry)
```

---

## 不允许的流转
要显式禁止一些跳跃：

### Task
- `Succeeded -> Running`
- `Cancelled -> Running`
- `Archived -> Running`
- `Failed -> Succeeded`（不能直接跳）

### Step
- `Succeeded -> Running`
- `Skipped -> Running`
- `Cancelled -> Running`

这些规则建议集中在 `execution/state.rs` 中统一维护，不要散落在业务里随便赋值。

---

## 七、为什么要集中做 transition
不要直接这样改：

```rust
task.status = TaskStatus::Running;
```

更推荐：

```rust
task.transition_to(TaskStatus::Running)?;
```

好处：
- 统一校验合法流转
- 自动更新 `updated_at`
- 可以顺手写 event
- 以后加审计逻辑更方便

例如：

```rust
impl Task {
    pub fn transition_to(&mut self, next: TaskStatus) -> Result<(), TaskTransitionError> {
        // 校验当前状态是否允许迁移到 next
        // 更新时间字段
        // 按需设置 started_at / finished_at
        Ok(())
    }
}
```

---

## 八、TaskEvent 建议
状态机之外，还建议保留 event 流。

## 推荐结构
```rust
pub struct TaskEvent {
    pub id: EventId,
    pub task_id: TaskId,
    pub step_id: Option<StepId>,
    pub kind: TaskEventKind,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
}
```

### 推荐事件类型
```rust
pub enum TaskEventKind {
    TaskCreated,
    TaskStarted,
    TaskBlocked,
    TaskSucceeded,
    TaskFailed,
    TaskCancelled,
    StepReady,
    StepStarted,
    StepSucceeded,
    StepFailed,
    StepSkipped,
    ArtifactProduced,
    RetryScheduled,
}
```

作用：
- 前端 timeline
- 排障
- 恢复执行
- 历史回放

---

## 九、和 storage 的关系
task 本身不负责存储，但字段设计要适合文件存储和未来 SQLite。

第一版建议：
- `Task` → `meta.json` 或 `task.json`
- `TaskStep` → 跟 task 一起存，或单独 `steps.json`
- `TaskEvent` → `events.jsonl`
- `Task output / artifacts` → artifact 目录 + event 引用

因为前面 storage 方案已经定成 JSON/JSONL 优先，所以 `input/output/payload` 用 `serde_json::Value` 很合适。

---

## 十、第一版最小可行模型
如果当前要先落最小实现，我建议先做：

### Task
- `id`
- `conversation_id`
- `parent_task_id`
- `kind`
- `title`
- `goal`
- `status`
- `input`
- `output`
- `error`
- `retry_count`
- `max_retries`
- `created_at`
- `updated_at`
- `started_at`
- `finished_at`

### TaskStep
- `id`
- `task_id`
- `index`
- `title`
- `action`
- `tool_name`
- `status`
- `input`
- `output`
- `error`
- `depends_on`

### TaskEvent
- `id`
- `task_id`
- `step_id`
- `kind`
- `payload`
- `created_at`

---

## 十一、推荐结论
当前 `chat-bot` 项目的 task 模型建议采用：

1. **task 表示整体目标**
2. **step 表示执行步骤**
3. **event 表示执行痕迹**
4. **message / task / event 三者分开**
5. **状态机集中定义，流转统一校验**
6. **输入输出先用 `serde_json::Value` 保持灵活**
7. **保留 parent_task_id、depends_on、retry 字段，为后续复杂执行链预留空间**

这版模型足够支撑：
- planner 产出 task/step
- runner 驱动执行
- tool router 执行 step
- timeline 展示任务过程
- storage 落 JSON/JSONL
- 后续再演进到 background tasks / autonomous loop / SQLite
