# chat-bot

按大项目规范整理后的 monorepo 骨架。

## 技术栈
- Rust workspace：承载领域模型、应用服务、API 契约
- Rust Web API：`apps/api-server`
- Tauri Desktop：`apps/desktop`
  - React + TypeScript + Vite 前端在 `apps/desktop`
  - Tauri Rust 壳在 `apps/desktop/src-tauri`

## 目录结构
```text
chat-bot/
├─ apps/
│  ├─ api-server/         # Rust API 入口
│  └─ desktop/            # Tauri 桌面应用（前端 + src-tauri）
├─ crates/
│  ├─ api-contracts/      # 前后端共享协议
│  ├─ application/        # 应用服务层
│  └─ domain/             # 领域模型与核心类型
├─ docs/
│  └─ design/             # 设计/架构文档
├─ Cargo.toml             # Rust workspace 根配置
└─ package.json           # 前端 workspace 根配置
```

## 为什么这样更合理
- Tauri 官方常见组织方式，就是一个 app 目录里同时放前端和 `src-tauri`
- `desktop` 作为一个完整交付单元，比 `web` 和 `desktop-tauri` 并排更自然
- 前端如果主要服务桌面端，这样能减少跨目录引用、脚本维护和心智负担
- 后面如果真要拆纯 Web 发布版，再单独新增 `apps/web` 也不晚

## 当前原则
- 先保留“真正有职责”的最小骨架
- 桌面端按 Tauri 应用单元组织，而不是人为拆成两个 app
- 业务长出来以后，再按边界继续拆分

## 命令
- `cargo check`
- `cargo run -p api-server`
- `npm --prefix apps/desktop run dev`
- `npm --prefix apps/desktop run build`

## 后续建议
- 先把 API 路由、配置管理、日志、前端页面骨架补起来
- 后面如需浏览器独立发布，再从 `apps/desktop` 前端抽离 `apps/web`
- 等出现真实存储/工具执行/第三方接入需求后，再新增 `persistence`、`integrations` 等 crate
