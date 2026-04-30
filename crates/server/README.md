# server

服务端入口 crate。

## 职责
- 启动服务端进程
- 组织路由、启动流程和入口配置
- 调用 `../core` 中的核心能力

## 常用命令
```bash
cargo check --manifest-path crates/server/Cargo.toml
cargo run --manifest-path crates/server/Cargo.toml
```
