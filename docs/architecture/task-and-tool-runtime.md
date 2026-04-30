# S12 + S20 落地设计

## 目标
这份文档把 `learn-claude-code-rs` 里的 S12（task system）和 S20（tool refactor）整理成适合当前 `chat-bot` 仓库的落地方案。

目标不是照搬章节代码，而是回答两个问题：

1. 任务系统在当前项目里应该拆成哪些文件和 trait
2. 工具路由系统在当前项目里应该怎么嵌进 `planning / execution / artifact`

---

## 两章分别解决什么问题
### S12 解决的问题
S12 解决的是：

- 聊天消息不等于执行过程
- 一次复杂请求需要拆成多个步骤
- 执行过程需要状态机，而不是一次函数调用
- 后续要支持后台执行、失败恢复、任务列表、任务时间线

一句话：
**S12 负责把“工作”建模成 task。**

### S20 解决的问题
S20 解决的是：

- 工具越来越多时，不能继续手写 if/else 分发
- 工具输入 schema 和 Rust 输入结构不能分裂成两份
- tool 调用需要统一注册、统一路由、统一 context

一句话：
**S20 负责把“step 怎么执行”重构成 tool router。**

---

## 在当前项目里的角色分工
结合当前仓库，建议这样理解：

```text
conversation  -> 用户请求和会话上下文
planning      -> 把目标拆成 plan / task / step
execution     -> 执行 task / step，并通过 tool router 调工具
artifact      -> 保存 patch、日志、文件、报告等结果
storage       -> 保存 conversation / task / event / artifact
```

也就是说：

- **S12 对应 `planning + execution::task + artifact`**
- **S20 对应 `execution::tool/router/context + read/search/write/validate`**

---

# 一、S12：任务系统应该怎么拆

## 1. 任务模型原则
任务系统的核心原则：

1. `message` 和 `task` 分离
2. `task` 是生命周期对象，不是一个普通函数返回值
3. `task` 支持 step 拆分
4. `task` 的状态变化必须可记录、可恢复、可展示

---

## 2. 建议的文件结构
建议在 `crates/core/src/planning` 和 `crates/core/src/execution` 中这样拆：

```text
crates/core/src/
  planning/
    mod.rs
    plan.rs
    planner.rs
    strategy.rs
  execution/
    mod.rs
    task.rs
    step.rs
    state.rs
    result.rs
    runner.rs
    context.rs
    event.rs
```

### 各文件职责
#### `planning/plan.rs`
定义计划模型：
- `ExecutionPlan`
- `PlannedTask`
- `PlannedStep`
- `StepDependency`

#### `planning/planner.rs`
定义规划接口：
- 如何从用户意图生成 plan
- 如何把复杂目标拆成 step
- 如何决定 step 顺序和依赖

#### `planning/strategy.rs`
定义规划策略：
- 严格顺序执行
- 可并行执行
- 需要人工确认
- 优先读、后写、最后校验

#### `execution/task.rs`
定义执行期 task 实体：
- `Task`
- `TaskId`
- `TaskKind`
- `TaskStatus`

#### `execution/step.rs`
定义执行期 step：
- `TaskStep`
- `StepId`
- `StepAction`
- `StepStatus`

#### `execution/state.rs`
定义状态流转规则：
- task 状态机
- step 状态机
- 状态是否合法迁移

#### `execution/result.rs`
定义执行结果：
- `TaskResult`
- `StepResult`
- `TaskError`
- `ValidationReport`

#### `execution/runner.rs`
定义执行器：
- 拿到 task
- 按 step 顺序执行
- 调 tool router
- 更新 task / step 状态
- 写 event / artifact

#### `execution/context.rs`
定义执行上下文：
- workspace
- conversation id
- auth session
- provider selection
- storage handles
- config snapshot

#### `execution/event.rs`
定义执行事件：
- `TaskStarted`
- `StepStarted`
- `StepCompleted`
- `TaskFailed`
- `ArtifactProduced`

---

## 3. S12 推荐 trait
### `Planner`
```rust
pub trait Planner {
    fn create_plan(&self, request: PlanRequest) -> Result<ExecutionPlan, PlanError>;
}
```

职责：
- 接收用户目标
- 生成 task / step 结构
- 指定依赖和执行策略

### `TaskRunner`
```rust
pub trait TaskRunner {
    fn run(&self, task: Task, ctx: ExecutionContext) -> Result<TaskResult, TaskError>;
}
```

职责：
- 驱动 task 生命周期
- 顺序或并行执行 step
- 调用工具
- 更新状态

### `TaskStore`
```rust
pub trait TaskStore {
    fn save_task(&self, task: &Task) -> Result<(), StorageError>;
    fn load_task(&self, id: &TaskId) -> Result<Task, StorageError>;
    fn append_event(&self, id: &TaskId, event: TaskEvent) -> Result<(), StorageError>;
}
```

职责：
- 持久化 task
- 恢复 task
- 写 event timeline

---

## 4. S12 推荐核心类型
### `Task`
```rust
pub struct Task {
    pub id: TaskId,
    pub kind: TaskKind,
    pub title: String,
    pub status: TaskStatus,
    pub steps: Vec<TaskStep>,
}
```

### `TaskStatus`
```rust
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}
```

### `TaskStep`
```rust
pub struct TaskStep {
    pub id: StepId,
    pub title: String,
    pub action: StepAction,
    pub status: StepStatus,
}
```

### `StepAction`
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

---

## 5. 状态机建议
### Task 状态流转
```text
Pending -> Running -> Completed
Pending -> Running -> Failed
Pending -> Cancelled
Failed  -> Running   (retry)
```

### Step 状态流转
```text
Pending -> Running -> Completed
Pending -> Running -> Failed
Pending -> Skipped
```

注意：
- 不要允许 `Completed -> Running`
- 不要允许 `Failed -> Completed` 直接跳过 retry
- 状态流转最好集中在 `state.rs`，不要散在各处 if/else

---

# 二、S20：工具系统应该怎么拆

## 1. 工具系统原则
工具系统的目标是：

1. 所有工具统一注册
2. 所有工具统一按名称路由
3. 输入 schema 和 Rust 结构体使用同一份定义
4. 工具 handler 和执行上下文解耦
5. 新增工具时尽量少改框架代码

---

## 2. 建议的文件结构
在 `crates/core/src/execution` 下继续补这一组：

```text
crates/core/src/execution/
  mod.rs
  tool.rs
  router.rs
  context.rs
  registry.rs
  reader.rs
  search.rs
  writer.rs
  validate.rs
```

### 各文件职责
#### `tool.rs`
定义统一工具 trait。

#### `router.rs`
维护工具注册表并负责按名字分发调用。

#### `registry.rs`
负责批量注册默认工具、按能力组织工具集。

#### `context.rs`
定义 `ToolContext`，把依赖注入工具而不是注入 router。

#### `reader.rs / search.rs / writer.rs / validate.rs`
先实现当前最核心的四类工具。

---

## 3. S20 推荐 trait
### `Tool`
```rust
pub trait Tool {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> serde_json::Value;
    fn call(&self, ctx: ToolContext, input: serde_json::Value) -> Result<ToolOutput, ToolError>;
}
```

职责：
- 暴露工具元数据
- 返回输入 schema
- 接收 JSON input
- 产出统一 `ToolOutput`

### `ToolRouter`
```rust
pub trait ToolRouter {
    fn register(&mut self, tool: Box<dyn Tool>);
    fn call(&self, ctx: ToolContext, name: &str, input: serde_json::Value) -> Result<ToolOutput, ToolError>;
}
```

职责：
- 注册工具
- 名称分发
- 调用对应 handler

### `ToolRegistry`
```rust
pub trait ToolRegistry {
    fn default_tools() -> Vec<Box<dyn Tool>>;
}
```

职责：
- 管理默认工具集
- 后续可按 provider / mode / permission 组合不同工具列表

---

## 4. ToolContext 为什么要单独拆
S20 里一个关键点就是：
**router 不持有 context，调用时再传 `ToolContext`。**

原因：
- router 只做注册表，职责单一
- context 生命周期更清晰
- tool 更容易单测
- 避免 router 变成巨型共享状态对象

建议的 `ToolContext`：

```rust
pub struct ToolContext {
    pub workspace: WorkspaceId,
    pub conversation_id: Option<String>,
    pub auth_session: Option<AuthSession>,
    pub provider_id: Option<String>,
    pub config_snapshot: Option<String>,
}
```

后续可以再补：
- storage handle
- artifact writer
- permission scope
- cancellation token

---

## 5. 输入结构和 schema 一体化
S20 最值得学的一点：
**工具输入不要维护两份定义。**

也就是说，最好使用：
- `serde` 做输入反序列化
- `schemars` 生成 JSON Schema

例如：

```rust
#[derive(Deserialize, JsonSchema)]
pub struct ReadInput {
    pub path: String,
    pub limit: Option<u64>,
}
```

好处：
- schema 不会和真实输入类型脱节
- 工具定义更集中
- 新增参数时改一处就够

---

## 6. 当前项目最适合先做的工具
建议第一批只实现四类：

### `ReadTool`
职责：
- 读文件
- 读目录摘要
- 读指定文本范围

### `SearchTool`
职责：
- 关键字搜索
- 文件名搜索
- 简单结构搜索

### `WriteTool`
职责：
- 写文件
- 覆盖文件
- 生成 patch / 产物

### `ValidateTool`
职责：
- 调 cargo check
- 调 cargo fmt --check
- 调 npm lint
- 产出校验报告

这四个够支撑最小闭环。

---

# 三、S12 + S20 在当前项目里的组合方式

## 1. 组合后的调用链
```text
conversation request
  -> planner.create_plan()
  -> ExecutionPlan
  -> Task + Steps
  -> runner.run(task, ctx)
  -> router.call(step.action, step.input)
  -> tool output
  -> artifact/event storage
  -> task state update
```

也就是：
- S12 负责把目标拆成 task + step
- S20 负责让 step 调到真正的工具

---

## 2. 建议的落地文件总览
```text
crates/core/src/
  planning/
    mod.rs
    plan.rs
    planner.rs
    strategy.rs
  execution/
    mod.rs
    context.rs
    event.rs
    result.rs
    router.rs
    runner.rs
    state.rs
    step.rs
    task.rs
    tool.rs
    registry.rs
    reader.rs
    search.rs
    writer.rs
    validate.rs
  artifact/
    mod.rs
```

---

## 3. 第一批最值得先写的 trait
建议优先顺序：

1. `Planner`
2. `TaskRunner`
3. `Tool`
4. `ToolRouter`
5. `TaskStore`

原因：
- 先把执行主链打通
- provider/auth/storage 先挂接口，不急着做复杂实现

---

## 4. 第一批最值得先写的实体
建议优先写：

- `Task`
- `TaskStatus`
- `TaskStep`
- `StepAction`
- `ExecutionPlan`
- `ToolContext`
- `ToolOutput`
- `TaskEvent`

这些一有，整个 runtime 就有骨架了。

---

## 5. 不建议现在就做的东西
当前阶段先别一口气做太多：

- 不先做复杂 provider routing
- 不先做宏系统 `#[tool]`
- 不先做并行调度器
- 不先做完整权限系统
- 不先做数据库实现

先把最小闭环跑通：

```text
plan -> task -> step -> tool router -> result -> artifact/event
```

这个闭环比外围花活重要得多。

---

## 推荐结论
对于当前 `chat-bot` 仓库：

- **S12 应该落成任务系统骨架**：`plan / task / step / state / runner / event`
- **S20 应该落成工具路由骨架**：`tool / router / registry / context / read/search/write/validate`
- **最小执行链路要优先于 provider 细节**
- **后续 background task、autonomous loop、MCP/plugin 都建立在这层之上**

这会是当前项目最合理、最容易演进的一版内核结构。
