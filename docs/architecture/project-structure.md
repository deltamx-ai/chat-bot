# 项目结构设计规范

## 设计目标
本项目是一个桌面优先的 agent workbench，不是单一聊天页。
因此目录结构必须同时支撑：
- 多会话管理
- 任务/执行流展示
- 技能与数据源接入
- Web 与 Tauri 双端承载
- Rust 后端 API 与领域逻辑复用

## 顶层结构
```text
chat-bot/
├─ apps/
│  ├─ api-server/              # Rust Web API 入口
│  ├─ desktop-tauri/           # Tauri 桌面壳
│  └─ web/                     # React + TS + Vite 前端
├─ crates/
│  ├─ domain/                  # 核心领域模型
│  ├─ application/             # 用例与业务编排
│  └─ api-contracts/           # 前后端共享协议
├─ docs/
│  ├─ design/                  # 界面与交互设计
│  ├─ architecture/            # 架构和依赖边界
│  └─ api/                     # 接口说明
└─ scripts/                    # 后续启动/构建脚本
```

## 模块划分原则
### apps/
只放“可运行入口”，不沉淀业务规则。
- `api-server`：HTTP / SSE / WebSocket / 配置装配 / 路由
- `web`：工作台界面
- `desktop-tauri`：桌面容器、系统桥接、窗口能力

### crates/
只放“可复用核心”。
- `domain`：会话、任务、队列、能力等领域对象
- `application`：创建会话、执行任务、更新状态等用例
- `api-contracts`：DTO、事件协议、接口 schema

## 依赖方向
- `apps/*` 可以依赖 `crates/*`
- `application` 可以依赖 `domain` 和 `api-contracts`
- `domain` 不依赖 `apps/*`
- `api-contracts` 不依赖 UI 和运行时实现

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
