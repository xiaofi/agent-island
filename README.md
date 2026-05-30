# Agent Island

Agent Island 是一个 macOS 优先的本地桌面悬浮岛，用于展示 Claude Code、Codex 等本地 AI 编程代理正在执行的会话任务和当前状态。

它只做状态发现、归一化、展示和跳转；不接管 agent 执行流程，不默认读取完整对话内容，也不向云端上传数据。

## 当前重点

- 桌面壳：Tauri 2。
- 前端：Vue 3 + TypeScript + Vite + Pinia。
- 状态采集：Claude Code / Codex 官方 hook 为主路径，配置文件 discovery 为接入判断和诊断路径。
- UI：悬浮岛承载轻量状态浏览；设置、诊断等复杂流程打开独立窗口。

## 快速开始

```bash
npm install
npm test
npm run build
npm run tauri -- dev
```

常用检查：

```bash
npm test -- --run
cd src-tauri && cargo check
```

## 文档入口

- [docs/README.md](docs/README.md)：文档总目录和阅读路径。
- [docs/ai/context-map.md](docs/ai/context-map.md)：AI agent 任务导向上下文地图。
- [docs/product/spec.md](docs/product/spec.md)：产品 spec 详细版本。
- [docs/architecture/technical-plan.md](docs/architecture/technical-plan.md)：技术方案详细版本。
- [docs/architecture/hook-integration-plan.md](docs/architecture/hook-integration-plan.md)：Claude Code / Codex hook 接入详细方案。

## AI Agent 入口

AI agent 应优先读取：

1. [AGENTS.md](AGENTS.md)
2. [llms.txt](llms.txt)
3. [docs/ai/context-map.md](docs/ai/context-map.md)

这三个文件只提供高信号上下文和跳转，不替代详细设计文档。
