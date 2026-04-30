# chat-bot

这是一个**包含多个项目和多种框架**的代码仓库，不按典型 monorepo 产品仓来组织。

当前主要包含两个运行入口和一个核心复用层：

- `apps`：Tauri 桌面端项目
  - 前端：React + TypeScript + Vite
  - 桌面壳：`src-tauri`
- `crates/server`：独立的 Rust 服务端入口
- `crates/cli`：独立的 Rust CLI 入口
- `crates/core`：核心能力复用区，供 `cli` 和 `server` 共用

## 目录结构
```text
chat-bot/
├─ apps/
│  └─ desktop/            # 独立 Tauri 桌面项目
│     ├─ src/             # React + TS + Vite 前端
│     └─ src-tauri/       # Tauri Rust 壳
├─ crates/
│  ├─ cli/                # CLI 入口
│  ├─ core/               # 核心领域与业务能力
│  └─ server/             # 服务端入口
├─ docs/
└─ Cargo.toml             # Rust 工程协调配置
```

## 说明
- 这不是一个前端 workspace 驱动的 monorepo 产品仓
- 根目录不负责统一调度前端脚本
- 每个项目尽量在自己的目录里完成安装、开发、构建
- Rust 侧统一采用 `core + adapter` 思路：核心能力在 `core`，运行入口放在 `cli` 和 `server`

## 常用命令

### desktop
```bash
cd apps
npm install
npm run dev
npm run tauri:build
npm run lint
npm run build
npm run cargo:check
```

### server
```bash
cargo check --manifest-path crates/server/Cargo.toml
cargo run --manifest-path crates/server/Cargo.toml
```

### cli
```bash
cargo check --manifest-path crates/cli/Cargo.toml
cargo run --manifest-path crates/cli/Cargo.toml
```

### Rust 全局检查
```bash
cargo check
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```
