# chat-bot

这是一个**包含多个项目和多种框架**的代码仓库，不按典型 monorepo 产品仓来组织。

当前主要包含两个独立项目：

- `apps/desktop`：Tauri 桌面端项目
  - 前端：React + TypeScript + Vite
  - 桌面壳：`src-tauri`
- `apps/api-server`：独立的 Rust API 项目

另外有少量共享 Rust 库放在 `crates/`：

- `crates/api-contracts`：接口协议/共享 DTO
- `crates/application`：应用服务层
- `crates/domain`：核心领域模型

## 目录结构
```text
chat-bot/
├─ apps/
│  ├─ api-server/         # 独立 Rust API 项目
│  └─ desktop/            # 独立 Tauri 桌面项目
│     ├─ src/             # React + TS + Vite 前端
│     └─ src-tauri/       # Tauri Rust 壳
├─ crates/                # 共享 Rust 库
├─ docs/
└─ Cargo.toml             # Rust 工程协调配置
```

## 说明
- 这不是一个前端 workspace 驱动的 monorepo 产品仓
- 根目录不负责统一调度前端脚本
- 每个项目尽量在自己的目录里完成安装、开发、构建
- `crates/` 只是给 Rust 项目复用的共享代码，不代表整个仓库按 monorepo 产品模型管理

## 常用命令

### desktop
```bash
cd apps/desktop
npm install
npm run doctor:tauri
npm run dev
npm run tauri:build
npm run lint
npm run build
npm run cargo:check
```

### api-server
```bash
cargo check --manifest-path apps/api-server/Cargo.toml
cargo run --manifest-path apps/api-server/Cargo.toml
```

### Rust 全局检查
```bash
cargo check
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```
