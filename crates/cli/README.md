# cli

命令行入口 crate。

## 职责
- 提供 CLI 模式入口
- 解析参数、组织命令编排
- 调用 `../core` 中的核心能力

## 常用命令
```bash
cargo check --manifest-path crates/cli/Cargo.toml
cargo run --manifest-path crates/cli/Cargo.toml
```
