# chat-bot

按大项目规范整理后的 monorepo 骨架。

## 技术栈
- Rust workspace：承载领域模型、应用服务、API 契约
- Rust Web API：`apps/api-server`
- React + TypeScript + Vite：`apps/web`
- Tauri 桌面壳：`apps/desktop-tauri`

## 目录结构
```text
chat-bot/
├─ apps/
│  ├─ api-server/         # Rust API 入口
│  ├─ desktop-tauri/      # Tauri 桌面端壳
│  └─ web/                # React + TS + Vite 前端
├─ crates/
│  ├─ api-contracts/      # 前后端共享协议
│  ├─ application/        # 应用服务层
│  └─ domain/             # 领域模型与核心类型
├─ docs/
│  └─ design/             # 设计/架构文档
├─ Cargo.toml             # Rust workspace 根配置
└─ package.json           # 前端 workspace 根配置
```

## 当前原则
- 先保留“真正有职责”的最小骨架
- 不提前拆太多 runtime / integrations / persistence / automation 空壳包
- 业务长出来以后，再按边界继续拆分

## 命令
- `cargo check`
- `cargo run -p api-server`
- `npm --prefix apps/web run dev`
- `npm --prefix apps/web run build`

## 后续建议
- 先把 API 路由、配置管理、日志、前端页面骨架补起来
- 等出现真实存储/工具执行/第三方接入需求后，再新增 `persistence`、`integrations` 等 crate
