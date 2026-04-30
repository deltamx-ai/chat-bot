# desktop

Tauri 桌面端项目。

## 结构

- `src/`：React + TypeScript + Vite 前端
- `src-tauri/`：Tauri Rust 壳

## 常用命令

```bash
cd apps/desktop
npm install
npm run dev
npm run tauri:build
npm run lint
npm run build
npm run cargo:check
```

说明：
- `npm run dev` / `npm run tauri:dev`：启动 Tauri 桌面开发环境
- `npm run web:dev`：只启动前端 Vite 开发服务器
- `npm run tauri:build`：构建桌面应用
- `npm run cargo:check`：只检查 Tauri Rust 壳
