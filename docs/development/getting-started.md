# Getting Started

## 环境

- Node.js 可运行 Vite / Vue 工具链。
- Rust toolchain 可运行 Tauri。
- macOS 需要 Xcode Command Line Tools；完整 Xcode 不是当前 MVP 的硬性前提，但打包和签名阶段可能需要。

## 安装

```bash
npm install
```

## 本地运行

Web dev server:

```bash
npm run dev
```

Tauri app:

```bash
npm run tauri -- dev
```

## 验证

前端测试：

```bash
npm test -- --run
```

前端构建：

```bash
npm run build
```

Rust 编译检查：

```bash
cd src-tauri && cargo check
```

## 已知工程细节

- `src-tauri/Cargo.toml` 当前通过 `[patch.crates-io]` vendor 了 `dispatch2`，用于规避本地 Tauri 依赖链编译问题。
- Tauri 透明窗口使用 `macos-private-api`，MVP 目标是本地 macOS 工具，不以 Mac App Store 分发为约束。
- `src-tauri/Cargo.lock` 应保留，不应加入忽略。

## 开发入口

- UI 入口：`src/app/App.vue`
- 悬浮岛：`src/app/IslandApp.vue`
- 完整窗口：`src/app/FullWindowApp.vue`
- Tauri bridge：`src/bridge/tauriApi.ts`
- Rust app：`src-tauri/src/lib.rs`
- Rust commands：`src-tauri/src/commands/`
