# Storage 设计

## 目标
当前项目还没有进入高并发、多终端同步、复杂查询阶段。
因此存储方案的第一优先级不是“最强”，而是：
- 容易落地
- 容易调试
- 容易迁移
- 能支撑多会话与任务执行流

本阶段推荐采用：

- **主存储：文件系统目录 + JSON / JSONL**
- **后续演进：SQLite**
- **二进制附件：始终保留为文件系统存储**

也就是说，第一版不急着上数据库，但会在结构上为后续迁移到 SQLite 预留边界。

---

## 为什么第一版不用数据库
### 适合当前阶段的原因
1. 数据模型还在快速变化
2. 当前最需要的是可读、可改、可观察
3. 会话、消息、任务天然适合追加写入
4. 文件存储更方便排查问题与导出备份
5. 后续迁移到 SQLite 的成本可控

### 当前不急着上 DB 的前提
下面这些需求还没有成为核心矛盾：
- 大规模全文搜索
- 多维筛选和聚合统计
- 高并发写入
- 强事务需求
- 多端同时写同一份数据

只要还没到这一步，先上 DB 反而会增加实现和演进成本。

---

## 第一版推荐结构
```text
storage/
├─ conversations/
│  └─ <conversationId>/
│     ├─ meta.json
│     ├─ messages.jsonl
│     ├─ events.jsonl
│     ├─ summary.md
│     └─ artifacts/
├─ indexes/
│  ├─ recent.json
│  ├─ pinned.json
│  └─ tags.json
└─ snapshots/
   └─ <conversationId>-<timestamp>.json
```

### 目录说明
#### `conversations/<conversationId>/meta.json`
保存会话元信息：
- 标题
- 创建时间
- 更新时间
- 会话状态
- 工作区路径
- 标签
- 当前摘要
- 关联配置（模型、provider、模式等）

#### `messages.jsonl`
一行一条消息。
适合：
- 追加写
- 按时间回放
- 崩溃后恢复
- 调试查看

#### `events.jsonl`
保存非聊天类事件：
- 工具调用
- 子任务开始/结束
- 状态变更
- 文件产物生成
- 错误与重试

#### `summary.md`
会话摘要，不追求完整原始记录，而是给：
- UI 快速展示
- 历史回顾
- 上下文压缩

#### `artifacts/`
保存文件产物：
- patch
- screenshot
- exported json
- log
- generated file

---

## 为什么消息和事件要分开
不要把“聊天内容”和“执行轨迹”混成一个列表。

因为它们是两种完全不同的数据：

### 消息（messages）
代表对话本身：
- user message
- assistant message
- system note

### 事件（events）
代表执行过程：
- tool_started
- tool_finished
- task_created
- task_completed
- artifact_written
- retry
- warning
- error

分开之后：
- UI 更容易做成“三栏”
- 摘要压缩更清晰
- 搜索和过滤更自然
- 以后迁 DB 时表结构也更干净

---

## 会话如何切分
会话不建议按“时间”切，也不建议所有内容共用一个大对话。

推荐规则：

### 一个会话 = 一个明确目标 + 一个连续上下文
例如：
- 修复 Tauri dev 报错
- 重构 crates 目录
- 设计 storage 架构
- 为 server 增加配置加载

### 复用同一个会话的条件
满足这些条件就继续复用：
- 工作目标没变
- 仓库没变
- 问题链路没断
- 仍然需要上一轮上下文

### 新建会话的条件
出现这些情况就新建：
- 工作目标已经变化
- 仓库或项目切换
- 从“修 bug”切到“产品设计”
- 当前上下文太长、太脏
- 希望归档独立结果

一句话：
**按“目标/上下文”切，不按“今天聊了几句”切。**

---

## 会话元数据建议
`meta.json` 推荐至少包含：

```json
{
  "id": "conv_20260430_001",
  "title": "修复 Tauri dev 启动失败",
  "kind": "bugfix",
  "status": "active",
  "workspace": "~/workspace/ai/chat-bot",
  "scope": ["apps", "src-tauri"],
  "tags": ["tauri", "desktop"],
  "createdAt": "2026-04-30T15:30:00+08:00",
  "updatedAt": "2026-04-30T15:42:00+08:00",
  "summary": "正在修复 Tauri dev 启动链路并整理项目结构"
}
```

### 字段说明
- `kind`：如 `bugfix` / `feature` / `research` / `design` / `ops`
- `status`：如 `active` / `paused` / `archived`
- `workspace`：当前工作区或仓库路径
- `scope`：本会话主要影响范围
- `summary`：给列表和恢复上下文用

---

## JSONL 记录格式建议
### messages.jsonl
示例：
```json
{"id":"msg_1","role":"user","content":"desktop运行命令没有配置","createdAt":"2026-04-30T13:40:00+08:00"}
{"id":"msg_2","role":"assistant","content":"我来补 Tauri 命令和依赖","createdAt":"2026-04-30T13:40:03+08:00"}
```

### events.jsonl
示例：
```json
{"id":"evt_1","type":"task_started","name":"configure-tauri","createdAt":"2026-04-30T13:40:04+08:00"}
{"id":"evt_2","type":"tool_finished","tool":"git push","status":"success","createdAt":"2026-04-30T13:44:18+08:00"}
```

---

## ID 设计建议
### conversation id
不要只用随机串，也不要只用时间戳。
推荐：

```text
conv_<date>_<shortid>
```

例如：
```text
conv_20260430_a1b2c3
```

优点：
- 人眼可读
- 基本唯一
- 文件夹排序也更顺

### message / event / artifact id
同理：
- `msg_<shortid>`
- `evt_<shortid>`
- `art_<shortid>`

---

## 写入策略
### 基本原则
- `meta.json`：覆盖写
- `messages.jsonl`：追加写
- `events.jsonl`：追加写
- `artifacts/`：文件落盘后记录事件

### 建议增加的保护
- 先写临时文件，再 rename
- 每次写 JSON 保持完整对象
- 追加写失败时保留 error event
- 定期生成 snapshot

这样可以尽量避免崩溃导致半写坏文件。

---

## 索引设计
第一版不做复杂数据库索引，只做轻量级索引文件。

### recent.json
记录最近访问的会话 id 和时间

### pinned.json
记录置顶会话 id 列表

### tags.json
记录 tag 到会话 id 的简单映射

作用：
- 首页列表加载快
- 不必扫描所有目录才能展示基础列表
- 后续可平滑替换成 SQLite 查询

---

## 搜索策略
### 第一版
- 标题搜索：扫 `meta.json`
- 内容搜索：扫 `messages.jsonl`
- 事件搜索：扫 `events.jsonl`

可接受，因为当前规模小。

### 第二版
迁到 SQLite 后：
- conversation 表
- message 表
- event 表
- artifact 表
- tag 映射表

如果需要全文搜索，再考虑 SQLite FTS。

---

## 长期记忆不要混进会话目录
会话数据只保存“这个任务相关的上下文”。

长期信息建议分层：

```text
storage/
  conversations/
  memory/
    projects/
    people/
    global/
```

### conversations
当前任务过程数据

### projects
项目长期知识
- 仓库结构
- 构建坑点
- 约定

### people
人的偏好与身份信息

### global
通用偏好、全局配置、策略

这样不会让单个 conversation 无限膨胀。

---

## 迁移到 SQLite 的时机
出现以下情况时就该切：
- 会话数量明显增多
- 搜索速度开始变差
- 过滤维度越来越多
- 需要更稳定的分页、排序、统计
- 需要跨对象关联查询

### 迁移原则
迁移时不改变领域模型，只替换存储实现。

也就是说在 `core` 中先定义统一接口：
- `ConversationRepository`
- `MessageRepository`
- `EventRepository`
- `ArtifactRepository`

第一版由文件实现。
第二版新增 SQLite 实现。

这样 UI 和业务层不用跟着重写。

---

## core 层建议抽象
建议后续在 `crates/core` 里定义这些概念：

### 实体
- `Conversation`
- `ConversationMeta`
- `Message`
- `Event`
- `Artifact`

### 枚举
- `ConversationKind`
- `ConversationStatus`
- `MessageRole`
- `EventType`

### 仓储接口
- `ConversationStore`
- `MessageStore`
- `EventStore`
- `ArtifactStore`

### 服务
- `ConversationService`
  - 创建会话
  - 切换会话
  - 归档会话
  - 生成摘要

---

## 推荐结论
当前阶段采用：

1. **文件系统目录 + JSON / JSONL** 作为第一版主存储
2. **消息、事件、产物分开存**
3. **会话按目标/上下文切分**，不是按时间硬切
4. **长期记忆独立于会话目录**
5. **在 core 层先抽象存储接口**，为后续 SQLite 演进留口子

这是当前阶段实现成本最低、可读性最好、未来迁移成本也最低的一条路。
