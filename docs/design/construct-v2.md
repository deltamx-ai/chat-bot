# Construct v2：模块依赖、系统装配与执行主链路

> 这版继续往下收敛，不再只讲“有哪些板块”，而是明确：
>
> 1. 模块之间谁依赖谁
> 2. 哪些依赖方向不能反过来
> 3. 系统启动时怎么装配
> 4. 一次真实任务怎么沿着系统流动

---

## 1. v2 的核心目标

v1 解决的是：
- 功能应该按什么职责组织
- 为什么要围绕 task execution chain 组织

v2 要解决的是：
- **这些模块怎么真正接起来**
- **依赖边界怎么定，才不会后面变成一团**

一句话：

> v1 是骨架图，v2 是接线图。

---

## 2. 总体依赖原则

整个系统建议遵守一个最重要的方向：

```text
UI
  -> Application
      -> Domain
      -> Plan Runtime
      -> Capability Center
      -> Persistence Ports
      -> Integration Ports
```

实现层再去补端口实现：

```text
Persistence Impl / Integration Impl
  -> satisfy Application / Runtime ports
```

### 关键规则

- UI 只能依赖 `application` 和 `api-contracts`
- `application` 可以调用 `plan-runtime`
- `plan-runtime` 可以调用 `capability-center`
- `plan-runtime` 不能反向依赖 UI
- `domain` 不依赖 Tauri / Web / SQLite / Provider SDK
- `integrations` 不能决定业务流程，只提供能力

---

## 3. 模块依赖关系图

```text
apps-tauri/ui ───────┐
apps/web ────────────────────┤
                             v
                     [application]
                      /    |     \
                     /     |      \
                    v      v       v
               [domain] [plan-runtime] [capability-center]
                            |              |
                            |              |
                            v              v
                       [tool-runtime]   [registry metadata]
                            |
                            v
                      [integrations]
                            |
                            v
                      [persistence]
```

上图里有两个重点：

### 第一
`application` 是总入口。

所有 UI 按钮、列表、表单动作，最后都应该折叠成 application use case。

### 第二
`plan-runtime` 不是直接连 UI，而是被 application 驱动。

这样以后：
- 桌面端能用
- Web 能用
- 自动化 workflow 也能用

---

## 4. 每个模块的输入输出边界

## 4.1 UI
输入：
- DTO
- event stream
- view model

输出：
- user action
- command input
- query filter

不应该知道：
- SQLite 表结构
- provider SDK
- runtime 内部状态机细节

---

## 4.2 application
输入：
- UI command/query
- workflow trigger
- system event

输出：
- DTO
- domain mutation
- run start / plan generation / approval flow
- event publish

作用：
- 像总调度台
- 把散乱动作收成稳定用例

---

## 4.3 domain
输入：
- application 层传入的命令语义

输出：
- 领域对象
- 状态迁移结果
- 领域规则校验

作用：
- 回答什么是 Thread/Task/Run/Plan
- 回答哪些状态转换合法

---

## 4.4 plan-runtime
输入：
- task goal
- current context
- capability candidates
- policy / approval rules

输出：
- plan
- step execution requests
- replan decisions
- run events

作用：
- 这是“怎么推进”的大脑

---

## 4.5 capability-center
输入：
- 当前 workspace 配置
- tool/skill/agent/data source/provider 注册信息

输出：
- candidate capabilities
- capability descriptors
- risk / availability / permission metadata

作用：
- 给 runtime 提供“可调用能力地图”

---

## 4.6 tool-runtime
输入：
- step execution request
- capability metadata

输出：
- normalized execution result

作用：
- 把“做这一步”真正跑起来

---

## 4.7 integrations
输入：
- 具体 adapter 参数

输出：
- provider response
- filesystem result
- shell result
- mcp result

作用：
- 和外部世界打交道

---

## 4.8 persistence
输入：
- repository command/query

输出：
- stored state
- query model
- event history

作用：
- 把 thread/task/run/plan 落下来

---

## 5. 不能反向依赖的地方

这里很重要，不然后面一定长歪。

### 禁止 1：domain 依赖 persistence
错误：
- domain model 里直接塞 SQL/ORM 逻辑

正确：
- domain 只定义对象和规则
- persistence 去实现 repository

### 禁止 2：plan-runtime 依赖 Tauri
错误：
- runtime 里直接 emit tauri event

正确：
- runtime 发标准 domain/app event
- app-host / desktop bridge 再转成 Tauri 事件

### 禁止 3：UI 直连 tool/integration
错误：
- 前端按钮直接调 shell / fs / provider

正确：
- 一律走 application

### 禁止 4：integrations 决定任务流
错误：
- provider adapter 自己决定重试、审批、切 plan

正确：
- 这些决策在 application / runtime

---

## 6. 系统装配方式

这里建议专门有个 `app-host` crate。

它负责：
- 初始化配置
- 建立 SQLite 连接
- 装配 repositories
- 装配 capability registries
- 装配 tool adapters
- 装配 runtime
- 装配 application service
- 装配 event bus

### 启动顺序建议

```text
load config
  -> init logger
  -> init db
  -> run migrations
  -> build repositories
  -> build integrations
  -> build capability center
  -> build tool runtime
  -> build plan runtime
  -> build application service
  -> expose to tauri / web host
```

这套顺序的意思是：
- 先有底座
- 再有能力
- 再有大脑
- 最后再有壳

---

## 7. 一次真实请求的执行主链路

假设用户在右侧输入：

> 帮我检查这个 repo 里登录按钮为什么点了没反应

### Step 1：UI 层
- 输入框提交
- UI 调 `append_message`
- 如果用户点了“执行”，再调 `create_task` / `generate_plan`

### Step 2：application 层
- 把消息写入 thread
- 根据上下文创建 task
- 请求 plan-runtime 生成 plan
- plan 保存入库
- 把 `PlanGenerated` 事件发给 UI

### Step 3：plan-runtime
- 解析 goal
- 判断任务类型是 `code_change` / `diagnostic`
- 去 capability-center 拿候选能力
- 产出 plan：搜代码 -> 读文件 -> 分析 -> 修改 -> 验证

### Step 4：用户确认执行
- UI 调 `start_run`
- application 创建 run
- runtime 开始循环

### Step 5：tool-runtime + integrations
- 执行 grep
- 执行 read
- 执行 edit
- 执行 shell test
- 每步结果标准化返回

### Step 6：状态回写
- run_steps 落库
- artifacts 落库
- task/run 状态更新
- event 推回 UI

### Step 7：收尾
- run completed / failed
- UI 更新详情页
- thread 可追加总结消息

这条链一旦清楚，整个产品就不会散。

---

## 8. construct v2 下的目录再收敛一次

```text
chat-bot/
├── apps/
│   ├── desktop-tauri/
│   └── web/
│
├── crates/
│   ├── api-contracts/        # DTO / event schema / enums
│   ├── domain/               # 业务对象与规则
│   ├── application/          # 统一用例入口
│   ├── plan-runtime/         # 任务推进大脑
│   ├── capability-center/    # 能力目录与元数据
│   ├── tool-runtime/         # step 执行层
│   ├── integrations/         # 外部系统接入
│   ├── persistence/          # sqlite / repo / migrations
│   ├── automation/           # scheduler / triggers / workflow host
│   └── app-host/             # 装配容器 / bootstrap / event bus
│
└── docs/
    └── design/
```

这里最关键的变化是：

### `capability-center` 独立出来
因为你前面问“这些功能怎么组织到一起”，答案里最容易漏掉的一层就是：

> **能力不是 runtime 现想现拿，而是需要一个中间层统一编目。**

它负责把：
- tools
- skills
- agents
- data sources
- models/providers

组织成 runtime 可消费的能力地图。

---

## 9. 推荐的内部端口设计

为了避免 crate 乱依赖，建议 application 和 runtime 都只看 trait / port。

### application ports
- `ThreadRepository`
- `TaskRepository`
- `RunRepository`
- `PlanRepository`
- `SettingsRepository`
- `EventPublisher`
- `PlanRuntimePort`

### runtime ports
- `CapabilityLookup`
- `ToolExecutor`
- `ApprovalPolicy`
- `RunStateStore`
- `EventSink`

### integration ports
- `ProviderClient`
- `ShellClient`
- `FilesystemClient`
- `McpClient`
- `BrowserClient`

这样真正实现可以换，但骨架不动。

---

## 10. 事件流在 v2 里的位置

v1 提过“事件化”，v2 这里把它放准位置：

```text
plan-runtime / application
   -> event bus
      -> persistence event log
      -> tauri bridge
      -> web ws/sse bridge
      -> automation listeners
```

也就是说，事件不是 UI 专属玩具，而是系统级主通道。

它同时服务：
- UI 实时刷新
- 审计
- 重放
- 自动化触发
- 调试

---

## 11. construct v2 的最终结论

如果只保留一句最关键的话：

> **这个系统应该围绕“Application 统一门面 + Plan Runtime 执行大脑 + Capability Center 能力地图 + Persistence/Integrations 底座”来组织。**

对应关系就是：
- UI 负责看和点
- Application 负责接请求和编排用例
- Plan Runtime 负责想下一步
- Capability Center 负责告诉它现在能做什么
- Tool Runtime/Integrations 负责把动作真正执行掉
- Persistence 负责把一切留痕

这样整套东西才不会越做越糊。

---

## 12. 下一步最值钱的文档

按现在这个阶段，后面最该补的是：

1. **Use Case Map**
   - 每个按钮对应哪个 application use case

2. **Event Model**
   - 所有 run / plan / approval 事件定义

3. **Dependency Rules**
   - 每个 crate 允许依赖谁，不允许依赖谁

---

## 13. Application Use Case 设计

这一层要解决的不是“页面上有什么按钮”，而是：

> **所有按钮最终都收敛成哪些稳定的系统用例。**

如果这里不先收敛，后面 UI 一改，接口就会散。

### 13.1 Thread 相关 use case

- `CreateThread`
  - 创建新会话
  - 输入：workspace_id, title?, source?
  - 输出：thread summary

- `ListThreads`
  - 查询线程列表
  - 输入：filter, tag, status, keyword, pagination
  - 输出：thread list item[]

- `GetThreadDetail`
  - 查看线程详情
  - 输入：thread_id
  - 输出：thread detail + message timeline + current task refs

- `AppendThreadMessage`
  - 在线程中追加消息
  - 输入：thread_id, role, content, attachments?
  - 输出：message dto

- `RenameThread`
- `ArchiveThread`
- `PinThread`
- `TagThread`

### 13.2 Task 相关 use case

- `CreateTaskFromThread`
  - 从当前 thread 上下文抽取任务
  - 输入：thread_id, goal, scope?, mode?
  - 输出：task detail

- `ListTasks`
- `GetTaskDetail`
- `UpdateTaskGoal`
- `CancelTask`
- `ApproveTask`
- `RejectTask`
- `ReopenTask`

### 13.3 Plan 相关 use case

- `GeneratePlan`
  - 输入：task_id
  - 输出：plan detail + candidate capabilities

- `RegeneratePlan`
  - 输入：task_id, reason, constraints?
  - 输出：new plan version

- `PatchPlan`
  - 输入：plan_id, user edits
  - 输出：patched plan

- `ApprovePlan`
- `RejectPlan`
- `GetPlanDetail`
- `ListPlanVersions`

### 13.4 Run 相关 use case

- `StartRun`
  - 输入：task_id or plan_id
  - 输出：run detail

- `PauseRun`
- `ResumeRun`
- `StopRun`
- `RetryRun`
- `RetryStep`
- `GetRunDetail`
- `ListRunSteps`
- `GetRunArtifacts`

### 13.5 Capability 相关 use case

- `ListCapabilities`
- `GetCapabilityDetail`
- `TestCapabilityAvailability`
- `EnableCapability`
- `DisableCapability`
- `ListProviders`
- `UpdateProviderBinding`

### 13.6 Workspace / Settings 相关 use case

- `GetWorkspaceOverview`
- `SwitchWorkspace`
- `GetSettings`
- `UpdateSettings`
- `ListDataSources`
- `RefreshDataSourceIndex`

### 13.7 Approval / Review 相关 use case

- `ListPendingApprovals`
- `ApproveRunStep`
- `RejectRunStep`
- `RequestHumanInput`

### 13.8 UI 按钮到 use case 的收敛规则

可以用一句话定死：

- 页面按钮不直接对应 repository
- 页面按钮不直接对应 tool 调用
- 页面按钮统一对应 application use case

也就是说：

```text
Button Click
  -> UI Action
  -> Application Use Case
  -> Domain/Runtime/Repo/Events
```

这样以后桌面端和 web 端才能共用同一套系统行为。

---

## 14. Event Model 设计

如果说 use case 是“用户从外面怎么敲门”，
那 event model 就是“系统内部怎么说话”。

我建议 event model 分 4 层：

1. domain events
2. application events
3. runtime events
4. integration events

对 UI 暴露时，再映射成统一的 stream event。

### 14.1 统一事件信封

建议所有事件先包一层 envelope：

```rust
pub struct EventEnvelope<T> {
    pub event_id: String,
    pub event_type: String,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub sequence: i64,
    pub occurred_at: String,
    pub actor: Option<String>,
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
    pub payload: T,
}
```

这层非常关键，后面做：
- UI 实时订阅
- event log
- run replay
- 调试追踪
- 自动化触发

都会轻松很多。

### 14.2 Thread 事件

- `ThreadCreated`
- `ThreadRenamed`
- `ThreadArchived`
- `ThreadTagged`
- `ThreadMessageAppended`
- `ThreadMessageEdited`

核心 payload 示例：
- thread_id
- message_id
- role
- content_summary
- attachment_refs

### 14.3 Task 事件

- `TaskCreated`
- `TaskGoalUpdated`
- `TaskStatusChanged`
- `TaskApproved`
- `TaskRejected`
- `TaskCancelled`
- `TaskReopened`

关键字段：
- task_id
- thread_id
- goal
- old_status
- new_status
- reason

### 14.4 Plan 事件

- `PlanGenerated`
- `PlanRegenerated`
- `PlanPatched`
- `PlanApproved`
- `PlanRejected`
- `PlanVersionCreated`
- `PlanStepAdded`
- `PlanStepUpdated`
- `PlanStepRemoved`

关键字段：
- plan_id
- task_id
- version
- step_count
- strategy_summary

### 14.5 Run 事件

这是最核心的一组。

- `RunCreated`
- `RunQueued`
- `RunStarted`
- `RunPaused`
- `RunResumed`
- `RunCompleted`
- `RunFailed`
- `RunCancelled`
- `RunReplanned`

关键字段：
- run_id
- plan_id
- task_id
- status
- started_at
- finished_at
- failure_reason?

### 14.6 Step 事件

- `RunStepSelected`
- `RunStepStarted`
- `RunStepProgressed`
- `RunStepCompleted`
- `RunStepFailed`
- `RunStepWaitingApproval`
- `RunStepSkipped`
- `RunStepRetried`

关键字段：
- run_id
- step_id
- step_index
- capability_id
- attempt
- input_ref
- output_ref
- error_summary?

### 14.7 Approval 事件

- `ApprovalRequested`
- `ApprovalGranted`
- `ApprovalRejected`
- `HumanInputRequested`
- `HumanInputReceived`

这个模型是后面“敏感操作先确认”的基础。

### 14.8 Capability / Integration 事件

- `CapabilityRegistered`
- `CapabilityAvailabilityChanged`
- `ProviderBound`
- `ProviderUnbound`
- `IntegrationCallStarted`
- `IntegrationCallCompleted`
- `IntegrationCallFailed`

注意：
这类事件不直接代表业务成功，
它只是说明某个外部能力调用发生了什么。

### 14.9 面向 UI 的 stream event

UI 不一定要吃所有底层事件，建议做一层投影：

- `thread.updated`
- `task.updated`
- `plan.updated`
- `run.updated`
- `run.step.updated`
- `approval.pending`
- `artifact.created`
- `toast.info|warn|error`

这样 UI 协议会稳很多。

### 14.10 事件持久化建议

至少保留两类存储：

1. `event_log`
   - 做审计、重放、调试
2. `read_model tables`
   - 给 UI 查当前状态

不要让 UI 直接扫原始 event_log 来拼状态，不然会很重。

---

## 15. Dependency Rules 细化

下面这部分建议以后直接当工程守则。

### 15.1 crate 依赖白名单

#### `api-contracts`
可依赖：
- serde
- shared small utils

不可依赖：
- tauri
- sqlx
- provider sdk
- runtime impl

#### `domain`
可依赖：
- `api-contracts`（可选，最好只共享 enum/value objects）
- chrono / uuid / thiserror 这类基础库

不可依赖：
- `application`
- `persistence`
- `integrations`
- `tauri`
- `axum`

#### `application`
可依赖：
- `domain`
- `api-contracts`
- port traits

不可依赖：
- sqlx 具体实现
- tauri 具体桥接
- provider sdk

#### `plan-runtime`
可依赖：
- `domain`
- `api-contracts`
- capability lookup / executor ports

不可依赖：
- `desktop-tauri`
- `web`
- sqlite 具体实现

#### `capability-center`
可依赖：
- `api-contracts`
- `domain`（少量 capability value objects）

不可依赖：
- UI crate
- runtime UI bridge

#### `tool-runtime`
可依赖：
- `api-contracts`
- execution ports

不可依赖：
- tauri/web UI
- application use case impl

#### `integrations`
可依赖：
- 外部 SDK
- port traits

不可依赖：
- UI crate
- domain state machine
- runtime replan logic

#### `persistence`
可依赖：
- sqlx / sqlite
- `domain`
- repository ports

不可依赖：
- tauri/web
- provider sdk（除非是专门的 persistence adapter）

#### `app-host`
可依赖：
- 上述所有 crate

作用：
- 唯一允许“看见全局”的装配层

### 15.2 app 层依赖规则

#### `apps-tauri`
只应该依赖：
- `application`
- `api-contracts`
- `app-host`
- tauri bridge code

#### `apps/web`
只应该依赖：
- `api-contracts`
- web adapter / http client / ws client

不要让 web 端直接知道 Rust runtime 内部结构。

### 15.3 典型错误依赖示例

错误例子 1：
- React 页面直接知道 `run_steps` 表结构

错误例子 2：
- Tauri command 直接拼业务逻辑，绕过 application

错误例子 3：
- runtime 里直接 import sqlite repository concrete type

错误例子 4：
- provider adapter 里顺手改 task 状态

这些都要禁止。

### 15.4 推荐通过 CI 做依赖守卫

后面可以加：
- cargo deny / cargo hakari / custom lint
- workspace dependency graph check
- crate-level forbidden dependency test

也就是把“架构约束”从文档变成自动检查，不然文档很快会失效。

---

## 16. 这版 v2 的收敛结论

如果继续往下做实现，我建议就按这条线推进：

1. 先定 `application use case` 清单
2. 再定 `event model`
3. 再把 `crate dependency rules` 固化
4. 然后才开始落 API / command / repo / runtime skeleton

顺序别反。

不然很容易先把接口写热闹了，最后发现系统边界全糊在一起。

