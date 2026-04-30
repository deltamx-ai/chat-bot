# api-server

独立的 Rust API 项目入口。

## 职责
- 启动 HTTP API 服务
- 组织路由、启动流程和入口配置
- 依赖 `../../crates` 下的共享 Rust 库

## 常用命令
```bash
cargo check --manifest-path apps/api-server/Cargo.toml
cargo run --manifest-path apps/api-server/Cargo.toml
```
