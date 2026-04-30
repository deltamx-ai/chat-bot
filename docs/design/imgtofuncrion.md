# imgtofuncrion：参考图功能点到模块划分

> 这份不是讲交互细节，而是先回答一个更基础的问题：
>
> **图片上看到的每一个功能点，应该归属于哪个系统模块。**

这里的目标是先把“界面元素”映射到“模块职责”，避免后面一边画 UI 一边把逻辑塞乱。

---

## 1. 模块总表

先把模块定下来，后面所有功能点都往这里挂。

### M1. Workspace & Navigation
负责：
- 全局导航
- 入口切换
- 过滤上下文切换
- 页面/区域定位

### M2. Thread & Task Hub
负责：
- 会话管理
- 任务列表
- 状态分类
- thread 和 task 的索引与切换

### M3. Plan & Run Console
负责：
- 当前任务详情
- plan 展示
- run 控制
- step 状态
- 主输入区

### M4. Capability Center
负责：
- API/provider
- MCP
- 本地文件夹
- skills
- agent profiles
- model bindings

### M5. Automation Center
负责：
- 定时任务
- 事件触发
- 常驻智能体
- workflow host

### M6. Settings & System
负责：
- 设置
- 系统通知
- 最新动态
- 全局策略

### M7. Application Layer
负责：
- 把 UI 动作收敛成 use case
- 协调 domain / runtime / persistence / events

### M8. Plan Runtime
负责：
- 生成 plan
- 选择下一步
- 调用能力
- 处理 replan
- 发出运行事件

### M9. Persistence & Eventing
负责：
- thread/task/plan/run 落库
- event log
- read model
- artifact 持久化

---

## 2. 图片上的功能点 -> 模块映射

## 2.1 顶部/窗口区

### 1）macOS 三色按钮
归属模块：
- **M1 Workspace & Navigation**
- 桌面宿主补充：desktop shell

说明：
- 这是窗口控制，不属于业务模块
- 业务上只需要保留窗口状态感知，不要让它污染 application

### 2）左上品牌图标
归属模块：
- **M1 Workspace & Navigation**

说明：
- 用于回到主工作台
- 后续也可作为 workspace 入口

### 3）返回箭头
归属模块：
- **M1 Workspace & Navigation**

说明：
- 是导航行为，不是 thread 领域行为
- 不要把它做成 runtime 动作

### 4）顶部工作区/助手选择器
归属模块：
- **M1 Workspace & Navigation**
- 部分配置归 **M6 Settings & System**

说明：
- UI 上看是切换器
- 系统里本质是 workspace / profile context 切换

---

## 2.2 左侧导航区

### 5）新建会话
归属模块：
- UI 归 **M1 Workspace & Navigation**
- 业务归 **M2 Thread & Task Hub**
- 用例落点归 **M7 Application Layer**

对应 use case：
- `CreateThread`
- `CreateTaskFromThread`（可选）

### 6）所有会话
归属模块：
- **M2 Thread & Task Hub**

对应 use case：
- `ListThreads`
- `ListTasks`

### 7）积压
归属模块：
- **M2 Thread & Task Hub**

说明：
- 是 task intake / backlog 视图
- 本质是 task query slice，不是独立领域对象

### 8）待办
归属模块：
- **M2 Thread & Task Hub**

说明：
- 是任务状态视图
- 通常对应 `draft / todo / planned`

### 9）待审查
归属模块：
- 主要归 **M2 Thread & Task Hub**
- 审批机制归 **M7 Application Layer**
- 审批事件归 **M9 Persistence & Eventing**

说明：
- 这是“待处理任务入口”
- 真正审批逻辑不在列表层

### 10）完成
归属模块：
- **M2 Thread & Task Hub**

### 11）已取消
归属模块：
- **M2 Thread & Task Hub**

### 12）已标记
归属模块：
- **M2 Thread & Task Hub**

说明：
- 是 thread/task 的视图标签
- 属于索引组织，不属于 runtime

### 13）已归档
归属模块：
- **M2 Thread & Task Hub**
- 持久化策略归 **M9 Persistence & Eventing**

### 14）标签
归属模块：
- UI 入口归 **M1 Workspace & Navigation**
- 标签管理归 **M2 Thread & Task Hub**
- 标签数据落库归 **M9 Persistence & Eventing**

---

## 2.3 数据源区

### 15）API
归属模块：
- **M4 Capability Center**
- 配置管理归 **M6 Settings & System**

说明：
- 这里是 provider registry / model binding 的入口
- 不是 thread/task 模块内容

### 16）MCP
归属模块：
- **M4 Capability Center**

说明：
- 负责 server、tool、resource、prompt 编目

### 17）本地文件夹
归属模块：
- **M4 Capability Center**
- 底层 adapter 会落到 integrations/persistence

说明：
- UI 上是数据源
- runtime 看见的是 capability / datasource descriptor

---

## 2.4 技能区

### 18）技能
归属模块：
- **M4 Capability Center**

说明：
- skill 是能力资产，不是一次 run 的临时状态
- 安装/启停/升级都应该从 capability 视角管理

---

## 2.5 自动化区

### 19）定时任务
归属模块：
- **M5 Automation Center**

### 20）事件触发
归属模块：
- **M5 Automation Center**
- 底层事件依赖 **M9 Persistence & Eventing**

### 21）智能体
归属模块：
- UI 入口可放 **M5 Automation Center**
- 能力注册归 **M4 Capability Center**

说明：
- 如果它是“可长期运行的 agent/worker”，偏自动化
- 如果它是“可调用的 agent profile”，偏 capability
- 所以这里是一个跨模块点，但入口建议放自动化

---

## 2.6 底部系统区

### 22）设置
归属模块：
- **M6 Settings & System**

### 23）最新动态
归属模块：
- **M6 Settings & System**
- 数据来源依赖 **M9 Persistence & Eventing**

说明：
- 它本质是事件流投影/通知中心
- 不是单独的业务对象模块

---

## 2.7 中间会话列表区

### 24）列表标题（如“所有会话”）
归属模块：
- **M2 Thread & Task Hub**

### 25）筛选/排序按钮
归属模块：
- **M2 Thread & Task Hub**
- 查询编排归 **M7 Application Layer**

### 26）会话列表项
归属模块：
- **M2 Thread & Task Hub**

列表项内字段来源：
- title/status/current phase/pinned 来自 read model
- 最近更新时间来自 persistence query model
- waiting approval/running 来自 run/task projection

### 27）点击会话切换右侧详情
归属模块：
- UI 导航归 **M1 Workspace & Navigation**
- 数据承接归 **M2 Thread & Task Hub**
- 查询 use case 归 **M7 Application Layer**

---

## 2.8 右侧主工作区

### 28）当前任务标题
归属模块：
- **M3 Plan & Run Console**
- 标题编辑提交给 **M7 Application Layer**

### 29）右上分享按钮
归属模块：
- **M3 Plan & Run Console**
- 导出能力可由 **M7 Application Layer** 协调

### 30）右上全屏/展开按钮
归属模块：
- **M1 Workspace & Navigation**

说明：
- 这是视图状态，不属于业务

### 31）右上关闭按钮
归属模块：
- **M1 Workspace & Navigation**

### 32）执行计划卡片（Plan）
归属模块：
- **M3 Plan & Run Console**
- plan 数据由 **M7 Application Layer** 提供
- plan 生成能力来自 **M8 Plan Runtime**

说明：
- UI 只是展示当前 plan
- 真正 plan 的生成、版本化、变更都不在 UI

### 33）计划卡片底部复制按钮
归属模块：
- **M3 Plan & Run Console**

### 34）计划卡片底部 Markdown/raw 切换
归属模块：
- **M3 Plan & Run Console**

### 35）状态提示（思考中 / 耗时）
归属模块：
- **M3 Plan & Run Console**
- 状态源头来自 **M8 Plan Runtime**
- 状态投影和订阅依赖 **M9 Persistence & Eventing**

### 36）执行按钮
归属模块：
- UI 归 **M3 Plan & Run Console**
- 用例归 **M7 Application Layer**
- 执行核心归 **M8 Plan Runtime**

对应 use case：
- `StartRun`
- `ResumeRun`

### 37）待办下拉按钮
归属模块：
- UI 归 **M3 Plan & Run Console**
- 任务状态流转归 **M7 Application Layer**
- 状态持久化归 **M9 Persistence & Eventing**

### 38）主输入框
归属模块：
- **M3 Plan & Run Console**
- 输入后的消息/指令承接归 **M7 Application Layer**

说明：
- 这不是纯聊天输入框
- 它属于 thread continuation + task refinement 的主入口

### 39）左下附件按钮
归属模块：
- UI 归 **M3 Plan & Run Console**
- 文件引用/上传编排归 **M7 Application Layer**
- 实际存储归 **M9 Persistence & Eventing**

### 40）工具按钮
归属模块：
- UI 归 **M3 Plan & Run Console**
- 工具清单来源归 **M4 Capability Center**
- 真正执行时归 **M8 Plan Runtime**

### 41）右下模型选择器
归属模块：
- UI 归 **M3 Plan & Run Console**
- 模型/provider 管理归 **M4 Capability Center**
- 默认策略归 **M6 Settings & System**

### 42）信息按钮
归属模块：
- **M3 Plan & Run Console**
- 数据聚合由 **M7 Application Layer** 完成

说明：
- 点开后看到的是 task/thread/run 的聚合信息，不是单表直查

---

## 3. 一张更清楚的归属图

```text
界面层
├── 顶部区 / 导航行为 --------------------> M1 Workspace & Navigation
├── 左侧任务分类 / 中间列表 --------------> M2 Thread & Task Hub
├── 右侧详情 / plan / run / 输入 ----------> M3 Plan & Run Console
├── 数据源 / 技能 / 模型 -----------------> M4 Capability Center
├── 定时任务 / 触发器 / 常驻智能体 --------> M5 Automation Center
└── 设置 / 最新动态 ----------------------> M6 Settings & System

系统内核层
├── 所有按钮动作最终收敛 -----------------> M7 Application Layer
├── plan/run 推进与执行 ------------------> M8 Plan Runtime
└── 状态留痕 / 事件流 / 查询投影 ----------> M9 Persistence & Eventing
```

---

## 4. 最容易混掉的几个点

### 点 1：新建会话不等于只是 UI 动作
它表面在左侧，
但实际上会进入：
- thread 创建
- 可能的 task 草稿创建
- 列表 read model 更新
- event 发出

所以它是：
- 入口在 **M1**
- 核心归 **M2**
- 落地走 **M7 + M9**

### 点 2：执行按钮不属于“按钮模块”
它表面是一个按钮，
实际上是整套运行链路的起点：
- start run
- runtime loop
- step events
- artifacts
- final status

所以它要归：
- UI 在 **M3**
- 核心在 **M7 + M8 + M9**

### 点 3：API / MCP / 文件夹不是设置页附属品
它们本质上是系统能力地图的一部分，
所以更适合收进 **M4 Capability Center**，
不要散落在设置页里。

### 点 4：最新动态不是简单通知栏
它真正应该是：
- event stream projection
- run/task/system activity feed

所以它的数据根在 **M9 Persistence & Eventing**。

---

## 5. 建议后续怎么继续

这份 `imgtofuncrion` 的作用，是先把图片元素和模块挂钩。

下一步最自然的是补两份：

1. **imgtofuncrion-v2：功能点 -> application use case 映射表**
   - 每个按钮触发哪个 use case
   - 输入输出参数是什么

2. **imgtofuncrion-v3：功能点 -> crate / 文件目录落点**
   - 每个模块放到哪个 crate
   - Tauri / Web / shared contracts 怎么接

---

## 6. 当前结论

如果先只保留一句话：

> **这张图上的功能点，不应该按“页面区域”直接长代码，而应该先归并到 Navigation、Thread/Task Hub、Plan/Run Console、Capability Center、Automation Center、Settings/System 这 6 个前台模块，再由 Application、Plan Runtime、Persistence/Eventing 3 个内核模块承接。**

这样后面你继续往 crate、use case、event schema 推进时，就不会乱。
