# Chat Bot 桌面应用设计文档

> 目标：基于参考图，设计一个使用 **Rust + Tauri 2** 实现的桌面端 AI Agent Workbench。要求支持未来同时服务 **Tauri 桌面端** 和 **Web 端**，因此要抽离一层可复用 API / Application 层。

---

## 1. 产品目标

这个应用不是传统聊天框，而是一个“任务执行工作台”。

核心能力：
- 管理会话与任务状态
- 展示 agent 的 plan / 执行过程 / 结果
- 统一接入工具、技能、数据源、自动化、模型
- 支持后续把同一套核心能力同时暴露给 Tauri 和 Web

所以这个产品应该被设计成：

- **前端 UI 层**：桌面端 / Web 端共用交互模型
- **应用编排层**：统一 task、plan、run、thread、tool 调度
- **领域层**：定义消息、任务、步骤、状态、数据源等业务对象
- **基础设施层**：本地存储、provider、tool adapter、event bus、Tauri bridge

---

## 2. 参考界面拆解

参考图本质上是一个三栏式控制台：

1. **左侧导航栏**：对象分类与系统入口
2. **中间会话列表栏**：任务/线程索引
3. **右侧主工作区**：当前任务详情、执行计划、交互与控制

这不是“聊天页面”，而是“任务控制台页面”。

---

## 3. 每一个主要按钮/区域的功能拆解

## 3.1 左上角窗口区

### macOS 三色按钮
功能：
- 关闭窗口
- 最小化
- 最大化/缩放

在 Tauri2 里：
- 使用原生窗口能力
- Web 端不需要一比一实现，但可以保留视觉占位

### 左上小图标 / 品牌入口
功能：
- 返回首页或主工作台
- 展示当前 app 品牌
- 可扩展为 workspace 切换入口

### 顶部返回箭头
功能：
- 在详情导航栈中返回
- 或在线程/页面层级中回退

### 顶部工作区/助手选择器（示例里像“喵呜”）
功能：
- 切换当前 workspace / bot persona / agent profile
- 后续可支持多身份、多环境、多项目上下文

---

## 3.2 左侧导航栏

### 新建会话
功能：
- 新建 thread
- 创建空白任务上下文
- 默认进入输入态

对应领域对象：
- `Thread`
- `TaskDraft`

---

### 所有会话
功能：
- 展示全部 thread / task
- 默认主列表视图

---

### 积压
功能：
- 展示尚未进入执行的 backlog item
- 适合未来做 task intake 队列

---

### 待办
功能：
- 展示已创建但未运行的任务
- 也可表示人工确认前的任务

---

### 待审查
功能：
- 展示需要人类审批的任务
- 包括高风险操作、待 review patch、待确认 plan

领域上建议关联：
- `ApprovalRequest`
- `ReviewState`

---

### 完成
功能：
- 展示已成功完成的任务
- 可查看结果、产物、日志

---

### 已取消
功能：
- 展示被用户中断或系统放弃的任务

---

### 已标记
功能：
- 收藏/重点关注任务
- 方便快速回访

---

### 已归档
功能：
- 存历史，不参与活跃工作流

---

### 标签
功能：
- 管理标签体系
- 允许 thread/task 多维归类

建议支持：
- 手动标签
- 自动标签
- 来源标签（repo / provider / workflow）

---

## 3.3 数据源分组

### API
功能：
- 管理 provider/API 接入
- 查看连接状态、模型清单、鉴权配置

建议能力：
- provider 列表
- 测试连通性
- 模型发现
- 默认模型策略

### MCP
功能：
- 管理 MCP server
- 浏览可用 tools / resources / prompts

建议能力：
- server 状态
- 工具能力清单
- 权限边界
- 调试入口

### 本地文件夹
功能：
- 挂载本地目录
- 作为 agent 可访问数据源

建议能力：
- 目录索引
- 权限范围
- 最近访问
- 排除规则

---

## 3.4 技能

功能：
- 管理已安装 skill
- 搜索 skill
- 查看 skill 元信息
- 启用/禁用/升级

这是复合能力层，不是原子工具层。

---

## 3.5 自动化

### 定时任务
功能：
- cron 类任务
- 周期性运行 workflow

### 事件触发
功能：
- 文件变化触发
- webhook 触发
- 消息到达触发

### 智能体
功能：
- 管理常驻 agent / worker
- 配置 agent profile、权限、策略

---

## 3.6 底部系统区

### 设置
功能：
- 全局配置
- 模型配置
- provider 配置
- 外观主题
- 存储路径
- 安全与审批策略

### 最新动态
功能：
- 更新日志
- 任务事件流
- 系统通知
- 重要报错提醒

---

## 3.7 中间栏：会话列表区

### 顶部标题（所有会话）
功能：
- 当前筛选上下文显示

### 右上筛选/排序按钮
功能：
- 按状态过滤
- 按更新时间排序
- 按标签筛选
- 按 workspace / provider / repo 筛选

### 会话列表项
每一项建议展示：
- 标题
- 状态图标
- 当前阶段
- 最近更新时间或耗时
- 是否 pinned / waiting approval / running

点击行为：
- 切换右侧详情
- 可支持快捷键上下导航

---

## 3.8 右侧主工作区

### 顶部标题（当前任务名）
功能：
- 展示当前 thread 或 task 的标题
- 支持重命名

### 右上角分享/关闭/展开类按钮
建议定义为：
- 分享：导出链接 / 导出 markdown / 导出 JSON
- 全屏：放大详情视图
- 关闭：关闭当前详情或返回列表

---

### 执行计划卡片（Plan）
功能：
- 展示规划出来的步骤
- 告诉用户系统打算怎么做
- 某些情况下允许审批后执行

建议支持：
- Markdown 渲染
- step 展开/折叠
- 状态标记（pending/running/done/failed）
- 复制/导出

---

### 计划卡片底部操作（复制 / Markdown）
功能：
- 复制 plan 文本
- 切换 markdown/raw 视图
- 导出给外部模型/团队成员

---

### 状态提示（思考中... 2s）
功能：
- 展示当前执行态
- 呈现 elapsed time
- 呈现是否在调用工具 / 等待审批 / 重规划

建议状态模型：
- idle
- planning
- running
- waiting_approval
- blocked
- completed
- failed

---

### 执行按钮
功能：
- 对 plan 进行确认执行
- 或继续执行被暂停的任务

对应动作：
- `startTaskRun`
- `resumeTaskRun`

---

### 待办下拉按钮
功能：
- 改变任务状态
- 标记为待办 / 审批 / 归档 / 取消
- 也可切换 run policy

---

### 输入框
功能：
- 继续给当前线程补充指令
- 插入修正意见
- 追问
- 调整 plan

这个输入框是 **thread continuation input**，不只是聊天输入框。

---

### 左下角附件/工具按钮
建议定义为：
- 上传附件
- 插入文件引用
- 打开工具选择器
- 选择是否允许访问目录/图片

---

### 右下角模型选择器
功能：
- 切换当前 run 的默认模型
- 支持 per-thread / per-run 配置

建议支持：
- 最近使用
- provider 分组
- 默认模型策略

---

### 信息按钮
功能：
- 打开任务元信息面板
- 展示 thread id、task id、run id、workspace、repo、审批记录、日志摘要

---

## 4. 推荐产品对象模型

最少应该有这些核心对象：

- `Workspace`
- `Thread`
- `Message`
- `Task`
- `Plan`
- `PlanStep`
- `Run`
- `Artifact`
- `ApprovalRequest`
- `DataSource`
- `Skill`
- `Tool`
- `AgentProfile`
- `Workflow`
- `Tag`

其中关系建议：
- 一个 `Thread` 可关联多个 `Task`
- 一个 `Task` 可有多个 `Run`
- 一个 `Run` 对应一次实际执行过程
- 一个 `Run` 可生成多个 `Artifact`
- 一个 `Plan` 属于某个 `Task` 或某次 `Run`

---

## 5. 架构设计建议

你已经定了：
- **Rust**
- **Tauri 2**
- 抽离一个 **API 层** 给 Tauri2 和 Web 共用

我建议不要把它做成“前端直接撞 Rust command 一把梭”，而是拆成 5 层。

## 5.1 推荐分层

```text
apps/
├── desktop-tauri/         # Tauri2 桌面壳
├── web/                   # Web 前端
└── shared-ui/             # 可复用前端组件/状态模型（可选）

crates/
├── domain/                # 领域模型与规则
├── application/           # 用例层 / API 层 / orchestration facade
├── plan-runtime/          # plan / act / observe / replan 执行内核
├── tool-runtime/          # tool registry / adapters / execution
├── integrations/          # provider / mcp / fs / browser / web search 等接入
├── persistence/           # sqlite / repository / migrations
├── automation/            # workflow / scheduler / triggers
└── api-contracts          # 前后端共享 DTO / event schema
```

---

## 5.2 每层职责

### `domain`
负责：
- Task、Thread、Run、Plan、Approval 等核心模型
- 状态流转规则
- 领域约束

不负责：
- 数据库存取
- UI
- Tauri command

### `application`
负责：
- 暴露统一 use case
- 给 Tauri 和 Web 提供一致 API
- 组织 transaction、service、query

这层就是你说的“可抽离 API 层”的最佳位置。

比如：
- `create_thread`
- `list_threads`
- `start_task_run`
- `approve_run`
- `list_data_sources`
- `install_skill`

### `plan-runtime`
负责：
- 解析任务
- 工具匹配
- 生成 plan
- 执行循环
- replan

这是整个 agent workbench 的编排中枢。

### `tool-runtime`
负责：
- 统一工具注册
- tool capability
- adapter 执行
- tool result normalize

### `integrations`
负责：
- OpenAI / Anthropic / Gemini / Ollama 等 provider
- MCP server
- 本地文件系统
- 浏览器自动化
- web search
- shell

### `persistence`
负责：
- SQLite
- migrations
- repository
- 索引和查询

### `automation`
负责：
- 定时任务
- 事件触发
- 常驻 workflow

### `api-contracts`
负责：
- 给 Rust 和 Web/Tauri 共用的数据结构
- 请求响应 DTO
- event payload schema

---

## 6. Tauri2 与 Web 共用 API 层的设计

这是重点。

不要把“API 层”理解成一定是 HTTP server。更合理的是：

- **Application Layer = 统一用例接口**
- 桌面端通过 Tauri command 调它
- Web 端通过 HTTP / WebSocket / 本地 dev server 调它

也就是说，真正共用的是：
- use case
- DTO
- domain model
- event schema

而不是只共用一层 controller。

### 两种实现方式

#### 方案 A：统一 Rust 内核
- Tauri 调 Rust application service
- Web 端单独起一个 Rust server 复用同样 application service

优点：
- 逻辑完全一致
- 最不容易漂移

#### 方案 B：Rust 内核 + TS BFF
- Tauri 直接调 Rust
- Web 调 TS API，但 TS API 再桥接 Rust/服务

优点：
- Web 生态更灵活
- 前端同学更熟

我更建议先走 **方案 A**。

---

## 7. 推荐目录结构

```text
chat-bot/
├── apps/
│   ├── desktop-tauri/
│   │   ├── src-tauri/
│   │   └── ui/
│   └── web/
│       └── src/
│
├── crates/
│   ├── domain/
│   ├── application/
│   ├── plan-runtime/
│   ├── tool-runtime/
│   ├── integrations/
│   ├── persistence/
│   ├── automation/
│   └── api-contracts/
│
├── docs/
│   ├── design/
│   ├── architecture/
│   └── product/
│
├── scripts/
└── Cargo.toml
```

---

## 8. UI 到后端的交互模型建议

右侧工作区不要做成“发消息 -> 回文本”那么简单。

应该做成事件流：

```text
UI action
  -> application use case
  -> plan runtime / tool runtime
  -> event stream updates
  -> UI incremental render
```

所以建议从一开始就支持：
- `RunCreated`
- `PlanGenerated`
- `StepStarted`
- `StepCompleted`
- `StepFailed`
- `ApprovalRequested`
- `RunCompleted`

这样 UI 才能实时展示“思考中、执行中、重规划中”。

---

## 9. 第一版 MVP 建议

先不要全做满，第一版只做：

### 功能范围
- 新建会话
- 会话列表
- 右侧任务详情
- plan 展示
- 执行按钮
- 输入续聊
- 本地 provider/API 配置入口
- 本地文件夹数据源

### 后端范围
- thread/task/run 基础模型
- application 层基础用例
- plan-runtime MVP
- tool-runtime 只接 4 个能力：
  - file read
  - file search
  - shell
  - provider chat

### UI 范围
- 三栏界面
- 状态筛选
- 详情页
- step timeline
- 模型选择器

---

## 10. 一句话结论

这个项目最合理的方向不是“做一个聊天窗口”，而是：

> **做一个以 task / plan / run 为核心的 Agent Workbench，并把 Rust application 层抽成 Tauri2 与 Web 共用的统一内核。**

