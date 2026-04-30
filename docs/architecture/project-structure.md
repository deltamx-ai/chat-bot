# 项目结构设计规范

## 设计目标
本项目是一个桌面优先的 agent workbench，不是单一聊天页。
因此目录结构必须同时支撑：
- 多会话管理
- 任务/执行流展示
- 技能与数据源接入
- Web 与 Tauri 双端承载
- Rust 服务端与核心逻辑复用

## 顶层结构
```text
chat-bot/
├─ apps/
│  └─ desktop/                 # Tauri 桌面端前端与壳
├─ crates/
│  ├─ cli/                     # CLI 入口
│  ├─ core/                    # 核心领域、共享类型、业务能力
│  └─ server/                  # 服务端入口
├─ docs/
│  ├─ design/                  # 界面与交互设计
│  ├─ architecture/            # 架构和依赖边界
│  └─ api/                     # 接口说明
└─ scripts/                    # 后续启动/构建脚本
```

## 模块划分原则
### apps/
只放“可运行前端入口”，不沉淀业务规则。
- `desktop`：工作台界面 + Tauri 容器

### crates/
只放 Rust 侧能力与入口。
- `core`：核心模型、共享 DTO、业务流程、服务抽象
- `cli`：CLI 模式入口、命令解析、终端输出
- `server`：服务端入口、路由、启动流程、对外服务提供

## 依赖方向
- `cli` 可以依赖 `core`
- `server` 可以依赖 `core`
- `core` 不依赖 `cli` 和 `server`
- `apps/desktop/src-tauri` 可以按需依赖 `core`

## 当前推荐业务模块
- `conversation`
- `task`
- `queue`
- `capability`
- `settings`

## 目录演进原则
当前先维持最小闭环，不提前拆出以下空壳目录：
- `runtime`
- `host`
- `engine`
- `adapter`
- `common`

只有在真实复杂度出现后，再新增：
- `infrastructure`
- `persistence`
- `integrations`
- `agent-runtime`
