# Chat Bot 桌面应用 Detail 设计文档

> 基于上一版 `desktop-agent-workbench-design.md`，继续细化：
> - 更细目录结构
> - Rust crate 划分
> - 数据模型初稿
> - API 草案
> - Tauri2 / Web 共用内核方式

---

## 1. 目标

这个项目不是简单聊天应用，而是一个 **Agent Workbench / Task Console**。

核心原则：
- UI 只是壳
- Rust application 才是共用内核
- Tauri2 和 Web 共用同一套 use case / DTO / event schema
- `task / plan / run / thread` 是第一层对象，不是 message-only app

---

## 2. 更细目录结构

```text
chat-bot/
├── Cargo.toml                      # workspace root
├── Cargo.lock
├── package.json                    # 前端 workspace root（可选 pnpm workspace）
├── pnpm-workspace.yaml
├── README.md
│
├── apps/
│   ├── desktop-tauri/
│   │   ├── src-tauri/
│   │   │   ├── Cargo.toml
│   │   │   ├── tauri.conf.json
│   │   │   └── src/
│   │   │       ├── main.rs
│   │   │       ├── lib.rs
│   │   │       ├── commands/
│   │   │       │   ├── mod.rs
│   │   │       │   ├── threads.rs
│   │   │       │   ├── tasks.rs
│   │   │       │   ├── runs.rs
│   │   │       │   ├── settings.rs
│   │   │       │   ├── data_sources.rs
│   │   │       │   └── skills.rs
│   │   │       ├── state/
│   │   │       │   └── app_state.rs
│   │   │       └── bridge/
│   │   │           ├── event_emit.rs
│   │   │           └── dto_mapper.rs
│   │   │
│   │   └── ui/
│   │       ├── index.html
│   │       ├── src/
│   │       │   ├── main.tsx
│   │       │   ├── app/
│   │       │   ├── pages/
│   │       │   ├── widgets/
│   │       │   ├── stores/
│   │       │   ├── hooks/
│   │       │   ├── api/
│   │       │   └── view-models/
│   │       └── package.json
│   │
│   └── web/
│       ├── src/
│       │   ├── main.tsx
│       │   ├── app/
│       │   ├── pages/
│       │   ├── widgets/
│       │   ├── stores/
│       │   ├── hooks/
│       │   ├── api/
│       │   └── view-models/
│       └── package.json
│
├── crates/
│   ├── api-contracts/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── dto/
│   │       ├── events/
│   │       └── enums/
│   │
│   ├── domain/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── thread/
│   │       ├── task/
│   │       ├── plan/
│   │       ├── run/
│   │       ├── artifact/
│   │       ├── approval/
│   │       ├── data_source/
│   │       ├── skill/
│   │       ├── workflow/
│   │       ├── tag/
│   │       └── common/
│   │
│   ├── application/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── app_service.rs
│   │       ├── commands/
│   │       ├── queries/
│   │       ├── handlers/
│   │       ├── ports/
│   │       ├── dto/
│   │       └── mappers/
│   │
│   ├── plan-runtime/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── runtime.rs
│   │       ├── intent_parser.rs
│   │       ├── tool_matcher.rs
│   │       ├── strategy/
│   │       ├── plan_builder.rs
│   │       ├── step_selector.rs
│   │       ├── step_executor.rs
│   │       ├── replanner.rs
│   │       ├── goal_evaluator.rs
│   │       ├── approval_gate.rs
│   │       ├── state.rs
│   │       └── events.rs
│   │
│   ├── tool-runtime/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── registry.rs
│   │       ├── capability.rs
│   │       ├── executor.rs
│   │       ├── normalizer.rs
│   │       └── adapters/
│   │           ├── fs_read.rs
│   │           ├── fs_search.rs
│   │           ├── shell.rs
│   │           ├── provider_chat.rs
│   │           ├── mcp.rs
│   │           └── browser.rs
│   │
│   ├── integrations/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── providers/
│   │       ├── mcp/
│   │       ├── filesystem/
│   │       ├── browser/
│   │       ├── automation/
│   │       └── embedding/
│   │
│   ├── persistence/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── db.rs
│   │       ├── migrations/
│   │       ├── repositories/
│   │       ├── sqlite/
│   │       └── queries/
│   │
│   ├── automation/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── scheduler.rs
│   │       ├── trigger_engine.rs
│   │       ├── workflow_runtime.rs
│   │       └── workflow_templates/
│   │
│   └── app-host/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── container.rs
│           ├── bootstrap.rs
│           └── event_bus.rs
│
├── docs/
│   ├── design/
│   ├── architecture/
│   ├── api/
│   └── product/
│
├── scripts/
│   ├── dev-desktop.sh
│   ├── dev-web.sh
│   ├── fmt.sh
│   ├── lint.sh
│   └── migrate.sh
│
└── fixtures/
    ├── mock-threads/
    ├── mock-runs/
    └── mock-plans/
```

---

## 3. crate 划分建议

## 3.1 `api-contracts`
职责：
- 前后端共享 DTO
- 事件 schema
- 状态枚举
- API 输入输出结构

这里不要放业务逻辑。

典型内容：
- `ThreadDto`
- `TaskDto`
- `RunDto`
- `PlanDto`
- `PlanStepDto`
- `RunEventDto`
- `TaskStatusDto`

---

## 3.2 `domain`
职责：
- 核心业务对象和规则
- 状态流转
- 领域不变量

典型聚合：
- Thread
- Task
- Run
- Plan
- ApprovalRequest
- DataSource
- Workflow

例子：
- Task 从 `draft -> planned -> running -> waiting_approval -> completed/failed/cancelled`
- Run 必须归属于 Task
- PlanStep 必须属于 Plan

---

## 3.3 `application`
职责：
- 提供统一 use case
- 是 Tauri2 与 Web 共用 API 的真正核心
- 组织 transaction / repository / runtime 调用

典型接口：
- `create_thread`
- `append_message`
- `create_task_from_thread`
- `generate_plan`
- `start_run`
- `approve_run`
- `cancel_run`
- `list_threads`
- `list_tasks`
- `list_runs`
- `save_settings`

一句话：**这层是你的 API 层。**

---

## 3.4 `plan-runtime`
职责：
- 把自然语言请求转成 plan
- 匹配工具
- 执行 step loop
- replan

它不应该直接关心 UI，也不应该直接做数据库细节。

---

## 3.5 `tool-runtime`
职责：
- 管工具注册与调用
- 统一工具能力元信息
- 处理工具执行结果标准化

---

## 3.6 `integrations`
职责：
- 对接外部 provider / MCP / filesystem / browser
- 封装第三方细节

---

## 3.7 `persistence`
职责：
- SQLite
- migration
- repository 实现
- query model

---

## 3.8 `automation`
职责：
- 定时任务
- 事件触发
- 长期 workflow

---

## 3.9 `app-host`
职责：
- 把所有 crate 装配起来
- 注入 repository / runtime / provider / event bus
- 给 Tauri 和 server mode 共用

这个 crate 很值，能避免 Tauri 壳直接变大泥球。

---

## 4. 数据模型初稿

这里先给第一版核心表，不求一步到位，但保证能支撑 UI 和 runtime。

## 4.1 workspaces

```sql
CREATE TABLE workspaces (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  slug TEXT NOT NULL UNIQUE,
  description TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
```

用途：
- 多工作区隔离
- 后续支持不同 bot profile / repo scope

---

## 4.2 threads

```sql
CREATE TABLE threads (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  title TEXT NOT NULL,
  status TEXT NOT NULL,
  pinned INTEGER NOT NULL DEFAULT 0,
  archived INTEGER NOT NULL DEFAULT 0,
  source TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (workspace_id) REFERENCES workspaces(id)
);
```

`status` 建议值：
- `active`
- `backlog`
- `todo`
- `waiting_review`
- `completed`
- `cancelled`
- `archived`

---

## 4.3 messages

```sql
CREATE TABLE messages (
  id TEXT PRIMARY KEY,
  thread_id TEXT NOT NULL,
  role TEXT NOT NULL,
  content TEXT NOT NULL,
  content_type TEXT NOT NULL DEFAULT 'markdown',
  reply_to_message_id TEXT,
  created_at TEXT NOT NULL,
  FOREIGN KEY (thread_id) REFERENCES threads(id),
  FOREIGN KEY (reply_to_message_id) REFERENCES messages(id)
);
```

用途：
- 保存线程消息
- 支持用户指令、系统消息、assistant 输出

---

## 4.4 tasks

```sql
CREATE TABLE tasks (
  id TEXT PRIMARY KEY,
  thread_id TEXT NOT NULL,
  workspace_id TEXT NOT NULL,
  title TEXT NOT NULL,
  description TEXT,
  status TEXT NOT NULL,
  priority TEXT NOT NULL DEFAULT 'normal',
  source_message_id TEXT,
  current_plan_id TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (thread_id) REFERENCES threads(id),
  FOREIGN KEY (workspace_id) REFERENCES workspaces(id),
  FOREIGN KEY (source_message_id) REFERENCES messages(id)
);
```

`status` 建议值：
- `draft`
- `planned`
- `running`
- `waiting_approval`
- `waiting_review`
- `completed`
- `failed`
- `cancelled`

---

## 4.5 plans

```sql
CREATE TABLE plans (
  id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL,
  version INTEGER NOT NULL,
  summary TEXT,
  status TEXT NOT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY (task_id) REFERENCES tasks(id)
);
```

一个 task 可以有多个 plan version。

---

## 4.6 plan_steps

```sql
CREATE TABLE plan_steps (
  id TEXT PRIMARY KEY,
  plan_id TEXT NOT NULL,
  step_order INTEGER NOT NULL,
  title TEXT NOT NULL,
  kind TEXT NOT NULL,
  tool_name TEXT,
  status TEXT NOT NULL,
  reason TEXT,
  args_json TEXT,
  success_check TEXT,
  failure_policy TEXT,
  created_at TEXT NOT NULL,
  FOREIGN KEY (plan_id) REFERENCES plans(id)
);
```

`kind` 示例：
- `search`
- `inspect`
- `modify`
- `verify`
- `delegate`
- `approval`

---

## 4.7 runs

```sql
CREATE TABLE runs (
  id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL,
  plan_id TEXT,
  status TEXT NOT NULL,
  trigger_source TEXT NOT NULL,
  model TEXT,
  provider TEXT,
  started_at TEXT,
  finished_at TEXT,
  created_at TEXT NOT NULL,
  FOREIGN KEY (task_id) REFERENCES tasks(id),
  FOREIGN KEY (plan_id) REFERENCES plans(id)
);
```

`trigger_source`：
- `manual`
- `workflow`
- `retry`
- `resume`

---

## 4.8 run_steps

```sql
CREATE TABLE run_steps (
  id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL,
  plan_step_id TEXT,
  status TEXT NOT NULL,
  tool_name TEXT,
  input_json TEXT,
  output_json TEXT,
  error_text TEXT,
  started_at TEXT,
  finished_at TEXT,
  FOREIGN KEY (run_id) REFERENCES runs(id),
  FOREIGN KEY (plan_step_id) REFERENCES plan_steps(id)
);
```

这是执行日志核心表。

---

## 4.9 artifacts

```sql
CREATE TABLE artifacts (
  id TEXT PRIMARY KEY,
  run_id TEXT,
  task_id TEXT,
  type TEXT NOT NULL,
  name TEXT NOT NULL,
  uri TEXT,
  content_text TEXT,
  metadata_json TEXT,
  created_at TEXT NOT NULL,
  FOREIGN KEY (run_id) REFERENCES runs(id),
  FOREIGN KEY (task_id) REFERENCES tasks(id)
);
```

`type` 示例：
- `plan_markdown`
- `patch`
- `report`
- `file`
- `image`
- `json`

---

## 4.10 approval_requests

```sql
CREATE TABLE approval_requests (
  id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL,
  run_id TEXT,
  step_id TEXT,
  status TEXT NOT NULL,
  reason TEXT NOT NULL,
  requested_at TEXT NOT NULL,
  decided_at TEXT,
  decided_by TEXT,
  FOREIGN KEY (task_id) REFERENCES tasks(id),
  FOREIGN KEY (run_id) REFERENCES runs(id)
);
```

---

## 4.11 data_sources

```sql
CREATE TABLE data_sources (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  type TEXT NOT NULL,
  name TEXT NOT NULL,
  config_json TEXT NOT NULL,
  enabled INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (workspace_id) REFERENCES workspaces(id)
);
```

`type`：
- `api`
- `mcp`
- `local_folder`

---

## 4.12 skills

```sql
CREATE TABLE skills (
  id TEXT PRIMARY KEY,
  workspace_id TEXT,
  name TEXT NOT NULL,
  source TEXT,
  version TEXT,
  enabled INTEGER NOT NULL DEFAULT 1,
  metadata_json TEXT,
  created_at TEXT NOT NULL,
  FOREIGN KEY (workspace_id) REFERENCES workspaces(id)
);
```

---

## 4.13 tags / 映射表

```sql
CREATE TABLE tags (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL,
  name TEXT NOT NULL,
  color TEXT,
  created_at TEXT NOT NULL,
  FOREIGN KEY (workspace_id) REFERENCES workspaces(id)
);

CREATE TABLE thread_tags (
  thread_id TEXT NOT NULL,
  tag_id TEXT NOT NULL,
  PRIMARY KEY (thread_id, tag_id),
  FOREIGN KEY (thread_id) REFERENCES threads(id),
  FOREIGN KEY (tag_id) REFERENCES tags(id)
);

CREATE TABLE task_tags (
  task_id TEXT NOT NULL,
  tag_id TEXT NOT NULL,
  PRIMARY KEY (task_id, tag_id),
  FOREIGN KEY (task_id) REFERENCES tasks(id),
  FOREIGN KEY (tag_id) REFERENCES tags(id)
);
```

---

## 5. 对象关系建议

```text
Workspace
  ├─ Thread
  │   ├─ Message
  │   └─ Task
  │       ├─ Plan
  │       │   └─ PlanStep
  │       ├─ Run
  │       │   ├─ RunStep
  │       │   └─ Artifact
  │       └─ ApprovalRequest
  ├─ DataSource
  ├─ Skill
  └─ Tag
```

重点：
- `Thread` 是对话壳
- `Task` 是可执行对象
- `Run` 是一次执行实例
- `Plan` 是执行前/执行中的结构化方案

---

## 6. API 草案

我建议分成两种接口：
- **命令型 API**：创建、执行、审批、取消
- **查询型 API**：列表、详情、时间线、配置
- **事件流 API**：run 过程实时推送

### 6.1 Application Service 接口

```rust
pub trait AppService {
    fn create_thread(&self, input: CreateThreadInput) -> AppResult<ThreadDto>;
    fn list_threads(&self, input: ListThreadsInput) -> AppResult<Vec<ThreadListItemDto>>;
    fn get_thread_detail(&self, thread_id: String) -> AppResult<ThreadDetailDto>;

    fn append_message(&self, input: AppendMessageInput) -> AppResult<MessageDto>;

    fn create_task(&self, input: CreateTaskInput) -> AppResult<TaskDto>;
    fn generate_plan(&self, input: GeneratePlanInput) -> AppResult<PlanDto>;
    fn start_run(&self, input: StartRunInput) -> AppResult<RunDto>;
    fn resume_run(&self, input: ResumeRunInput) -> AppResult<RunDto>;
    fn cancel_run(&self, input: CancelRunInput) -> AppResult<()>;
    fn approve_request(&self, input: ApproveRequestInput) -> AppResult<ApprovalRequestDto>;

    fn list_runs(&self, task_id: String) -> AppResult<Vec<RunDto>>;
    fn get_run_detail(&self, run_id: String) -> AppResult<RunDetailDto>;

    fn list_data_sources(&self, workspace_id: String) -> AppResult<Vec<DataSourceDto>>;
    fn save_data_source(&self, input: SaveDataSourceInput) -> AppResult<DataSourceDto>;

    fn list_skills(&self, workspace_id: String) -> AppResult<Vec<SkillDto>>;
    fn install_skill(&self, input: InstallSkillInput) -> AppResult<SkillDto>;

    fn get_settings(&self) -> AppResult<SettingsDto>;
    fn save_settings(&self, input: SaveSettingsInput) -> AppResult<SettingsDto>;
}
```

---

## 6.2 Tauri command 草案

```rust
#[tauri::command]
async fn create_thread(input: CreateThreadInput, state: State<'_, AppState>) -> Result<ThreadDto, String>;

#[tauri::command]
async fn list_threads(input: ListThreadsInput, state: State<'_, AppState>) -> Result<Vec<ThreadListItemDto>, String>;

#[tauri::command]
async fn get_thread_detail(thread_id: String, state: State<'_, AppState>) -> Result<ThreadDetailDto, String>;

#[tauri::command]
async fn append_message(input: AppendMessageInput, state: State<'_, AppState>) -> Result<MessageDto, String>;

#[tauri::command]
async fn generate_plan(input: GeneratePlanInput, state: State<'_, AppState>) -> Result<PlanDto, String>;

#[tauri::command]
async fn start_run(input: StartRunInput, state: State<'_, AppState>) -> Result<RunDto, String>;

#[tauri::command]
async fn approve_request(input: ApproveRequestInput, state: State<'_, AppState>) -> Result<ApprovalRequestDto, String>;
```

这些 command 不直接写业务，只转调 `application`。

---

## 6.3 Web API 草案

### Threads

```http
POST   /api/threads
GET    /api/threads
GET    /api/threads/:threadId
POST   /api/threads/:threadId/messages
```

### Tasks / Plans / Runs

```http
POST   /api/tasks
POST   /api/tasks/:taskId/plan
POST   /api/tasks/:taskId/runs
POST   /api/runs/:runId/resume
POST   /api/runs/:runId/cancel
GET    /api/tasks/:taskId/runs
GET    /api/runs/:runId
POST   /api/approval-requests/:id/approve
POST   /api/approval-requests/:id/reject
```

### Data Sources / Skills / Settings

```http
GET    /api/data-sources
POST   /api/data-sources
GET    /api/skills
POST   /api/skills/install
GET    /api/settings
PUT    /api/settings
```

---

## 6.4 Event Stream 草案

用于右侧详情实时刷新。

### 事件类型

- `thread.updated`
- `task.updated`
- `plan.generated`
- `run.created`
- `run.started`
- `run.step.started`
- `run.step.completed`
- `run.step.failed`
- `run.waiting_approval`
- `run.completed`
- `run.failed`

### 事件 DTO 示例

```json
{
  "event": "run.step.completed",
  "runId": "run_123",
  "taskId": "task_456",
  "stepId": "step_002",
  "timestamp": "2026-04-30T00:00:00Z",
  "payload": {
    "title": "搜索登录组件",
    "toolName": "fs_search",
    "status": "completed",
    "summary": "找到 3 个候选文件"
  }
}
```

---

## 7. Tauri2 与 Web 共用 API 层方式

最稳的路线：

### 方案
- `application` crate 暴露统一 use case
- `desktop-tauri/src-tauri` 只做 Tauri bridge
- 后续 `server` 或 `web-host` 也复用 `application`
- DTO 与事件 schema 从 `api-contracts` 统一拿

### 好处
- 桌面和 Web 逻辑一致
- 测试集中在 Rust 内核
- UI 可替换，但核心不散

---

## 8. MVP 切分建议

### Milestone 1：能看
- 三栏 UI
- 线程列表
- 详情页
- mock 数据驱动

### Milestone 2：能存
- SQLite
- thread/message/task/run 基础表
- create/list/detail API

### Milestone 3：能规划
- `generate_plan`
- plan / plan_steps 落库
- plan card UI

### Milestone 4：能执行
- `start_run`
- 4 个最小工具
- run event stream

### Milestone 5：能审批
- approval request
- waiting review UI
- approve/reject

---

## 9. 一句话结论

如果要兼顾 Tauri2 和 Web，最关键的不是先写哪个前端，而是先把：

> **domain + application + plan-runtime + api-contracts**

这四块立稳。这样 UI 只是换皮，核心不会散。
