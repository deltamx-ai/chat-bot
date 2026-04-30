# Construct：Chat Bot 功能组织与总体架构设计

> 这份文档不先展开按钮细节，也不先铺满 API，而是先回答一个更根的问题：
>
> **这些功能到底应该怎么组织到一起，整个系统的主骨架是什么。**

---

## 1. 目标

这个项目要做的不是“一个聊天窗口 + 一堆设置页”，而是一个：

- 以 `Thread / Task / Plan / Run` 为核心对象
- 以 `Plan Runtime` 为执行编排中枢
- 以 `Tools / Skills / Agents / Data Sources` 为能力供给层
- 以 `Desktop(Tauri2) + Web` 为两个表现壳

的 **Agent Workbench**。

所以架构的重点不是“页面怎么切”，而是：

> **用户意图，如何穿过 UI、应用层、运行时、工具层、数据层，最后变成一个可执行、可观测、可恢复的任务过程。**

---

## 2. 一句话总架构

```text
UI Shell
  ↓
Application API
  ↓
Task / Thread / Run Orchestration
  ↓
Plan Runtime
  ↓
Tool / Skill / Agent Runtime
  ↓
Integrations + Persistence
```

这条链就是整个系统的主脊柱。

---

## 3. 从“功能组织”角度看，系统分 6 个板块

我建议不要按“页面”去组织功能，而要按“职责”组织。

### 板块 A：Workspace & Navigation
负责：
- 左侧导航
- 会话分类
- 标签
- 数据源入口
- 技能入口
- 自动化入口
- 设置入口

这一层本质上是：

> **系统入口与对象索引层**

它不负责执行任务，只负责让用户快速进入某个上下文。

---

### 板块 B：Thread & Task Hub
负责：
- 新建会话
- 浏览 thread 列表
- 从 thread 里抽任务
- 查看 task 状态
- 在线程里继续补充指令

这一层是：

> **用户意图承接层**

因为用户表面上是在“聊天”，但系统内部要把聊天抽成可执行 task。

---

### 板块 C：Plan & Run Console
负责：
- 展示 plan
- 用户审批/调整 plan
- 启动 run
- 展示 step 执行过程
- 展示运行状态、耗时、结果、错误

这一层是：

> **执行控制台**

这是整套产品最核心的工作面板。

---

### 板块 D：Capability Center
负责：
- Tool registry
- Skill registry
- Agent registry
- Data source registry
- Model/provider registry

这一层是：

> **能力供应层**

它不决定“什么时候调用”，但负责回答：
- 有哪些能力
- 每个能力能干嘛
- 能不能用
- 风险多高

---

### 板块 E：Execution Core
负责：
- 解析用户输入
- 识别任务意图
- 生成 plan
- 选择 step
- 执行 step
- 观察结果
- replan
- 发出事件流

这一层就是：

> **Plan Runtime / Orchestration Core**

它是系统真正的大脑。

---

### 板块 F：Persistence & Integration
负责：
- SQLite
- Repository
- Provider 接入
- MCP 接入
- 文件系统接入
- Shell / Browser / Search 接入
- 事件持久化

这一层是：

> **底座层**

没有它，上层所有“聪明”都落不了地。

---

## 4. 系统真正的主流程

你如果想看“这些功能怎么组织起来”，最应该看的是主流程，不是菜单树。

### 主流程 1：从输入到任务

```text
用户输入一段话
  ↓
Thread 追加消息
  ↓
Application 判断是否生成/更新 Task
  ↓
Task 进入 draft / planned
```

这里说明：
- `Thread` 是交流容器
- `Task` 是执行容器
- 一段聊天不一定直接执行，但可以沉淀成 task

---

### 主流程 2：从任务到计划

```text
Task 被用户点击执行 / 自动触发
  ↓
Application 调用 Plan Runtime
  ↓
Plan Runtime 预解析意图
  ↓
匹配工具 / 技能 / 代理 / 数据源
  ↓
生成 Plan
  ↓
Plan 落库并回显 UI
```

这一步就把“聊天意图”变成“结构化执行方案”。

---

### 主流程 3：从计划到运行

```text
用户确认执行
  ↓
创建 Run
  ↓
Plan Runtime 进入执行循环
  ↓
select next step
  ↓
调用 Tool / Skill / Agent
  ↓
拿到结果
  ↓
更新 step / run / task 状态
  ↓
必要时 replan
  ↓
完成或失败
```

这一步决定了右侧控制台应该怎么设计。

---

### 主流程 4：从运行到回显

```text
Run 过程中持续发 event
  ↓
Tauri / Web UI 订阅 event
  ↓
更新 plan 卡片、timeline、状态提示、结果区
```

所以 UI 不是等一坨最终文本，而是实时吃事件流。

---

## 5. 为什么要拆成 Thread / Task / Plan / Run 四层

很多产品一开始会把这些揉在一起，后面就会乱。

### Thread
表示“交流上下文”。

它解决的是：
- 我们在聊什么
- 上下文是什么
- 消息怎么串起来

### Task
表示“要解决的问题”。

它解决的是：
- 这次要完成什么目标
- 现在处于什么状态
- 是否等待审批/审核/取消

### Plan
表示“准备怎么做”。

它解决的是：
- 这件事打算分几步做
- 每步为什么这样做
- 每步要用什么能力

### Run
表示“一次实际执行过程”。

它解决的是：
- 这次真的跑了什么
- 哪一步成功/失败
- 花了多久
- 产生了哪些 artifact

一句话：

> **Thread 是对话，Task 是目标，Plan 是方案，Run 是执行实例。**

这四层一旦清楚，整个产品就容易稳住。

---

## 6. 功能组织的核心原则

### 原则 1：UI 不直接组织业务，UI 只组织视图

不要让 UI 自己决定：
- 什么时候创建 task
- 什么时候进入 waiting approval
- 什么时候 replan

这些都应该在 application + runtime 层。

---

### 原则 2：Application 层是统一门面

无论 Tauri2 还是 Web，都不要直接碰：
- repository
- provider
- runtime 内部细节

都只调用 application use case。

所以 application 层负责把分散功能组织成“可以被调用的能力入口”。

---

### 原则 3：Plan Runtime 只管推进，不管具体集成细节

它负责：
- 选下一步
- 维护状态机
- 决定是否 replan

它不应该知道：
- SQLite 怎么写
- OpenAI SDK 怎么调
- MCP transport 细节

这些都下沉到 runtime 下面的能力层和 integration 层。

---

### 原则 4：Capability Center 要做统一注册，而不是散接

Tool、Skill、Agent、Data Source、Model 都不能各自乱飞。

应该统一有：
- registry
- metadata
- capability descriptor
- availability / health status
- permission boundary

否则 planner 选能力时会失控。

---

### 原则 5：所有执行都要事件化

只要 run 在跑，就应该持续发：
- step started
- step completed
- step failed
- approval requested
- run completed

这样 UI、日志、审计、重放才会统一。

---

## 7. 推荐总体模块图

```text
[Desktop UI]        [Web UI]
      \               /
       \             /
        \           /
       [Application API Layer]
                 |
        ---------------------
        |         |         |
   [Thread/Task] [Plan Runtime] [Capability Center]
        |         |         |
        |         |    -------------------------
        |         |    |     |      |      |
        |         | [Tools] [Skills] [Agents] [Data Sources]
        |         |                 |
        -----------------------------
                      |
          [Persistence + Integrations]
```

这里最关键的是：

- UI 只接 `Application API Layer`
- `Plan Runtime` 不直接暴露给 UI
- `Capability Center` 给 runtime 提供可选能力目录
- `Persistence + Integrations` 是所有能力落地的底座

---

## 8. 每一层在产品里的角色

### UI Shell
用户看到的壳。
负责：
- 导航
- 列表
- 详情
- 输入
- 事件展示

### Application API Layer
统一门面。
负责：
- create/list/get/start/approve/cancel 这些用例
- 聚合查询
- 事务边界

### Thread / Task 子系统
业务对象入口。
负责：
- 会话承接
- 任务状态
- 任务归类

### Plan Runtime
执行编排大脑。
负责：
- plan / act / observe / replan

### Capability Center
能力目录。
负责：
- 有什么工具
- 有什么技能
- 有什么 agent
- 权限边界是什么

### Persistence + Integrations
底座。
负责：
- 数据存储
- 外部系统对接
- 原子能力执行

---

## 9. 目录组织建议（按这个骨架来）

```text
chat-bot/
├── apps/
│   ├── desktop-tauri/
│   └── web/
│
├── crates/
│   ├── api-contracts/
│   ├── domain/
│   ├── application/
│   ├── plan-runtime/
│   ├── capability-center/
│   ├── tool-runtime/
│   ├── integrations/
│   ├── persistence/
│   ├── automation/
│   └── app-host/
│
└── docs/
    └── design/
```

这里我建议把之前的能力供给再明确成一个：

### `capability-center`
它专门负责组织：
- tool metadata
- skill metadata
- agent metadata
- data source metadata
- provider metadata

因为从“功能怎么组织到一起”的角度看，**能力编目层**值得单独存在。

---

## 10. construct 的真正结论

如果只给一句最重要的话，那就是：

> **这个系统要围绕“任务执行链”组织，而不是围绕“页面菜单”组织。**

也就是：

- 左边是入口和索引
- 中间是 thread/task 列表
- 右边是 plan/run 控制台
- 中间真正把它们串起来的是 application + plan runtime
- 底下真正支撑它们的是 capability center + persistence + integrations

---

## 11. 下一步最值得补什么

如果按工程推进顺序，下一步最值的是三份：

1. **Construct v2：模块依赖关系图**
   - 谁依赖谁
   - 哪些方向不能反过来

2. **Application Use Case Map**
   - 每个按钮背后到底调用哪个 use case

3. **Event Model**
   - run 过程中到底发哪些事件
   - UI 怎么订阅和渲染

