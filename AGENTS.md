# AGENTS.md

## 语言

- 默认使用简体中文回答，除非用户明确要求其他语言。
- 代码、命令、文件名保持原样。

## 项目定位

Agent Island 是一个本地桌面悬浮岛，用于展示 Claude Code、Codex 等 AI 编程代理的运行状态。它只做状态发现、归一化、展示和跳转，不接管 agent 执行流程，不默认读取完整对话内容，不上传数据。

## 必读路径

按任务读取最小必要上下文：

- 项目总览：`README.md`、`docs/README.md`
- AI 上下文地图：`llms.txt`、`docs/ai/context-map.md`
- 产品行为：`docs/product/spec.md`、`docs/product/overview.md`
- 技术架构：`docs/architecture/technical-plan.md`、`docs/architecture/overview.md`
- Hook 接入：`docs/architecture/hook-integration-plan.md`、`docs/architecture/hook-ingestion.md`
- 开发运行：`docs/development/getting-started.md`
- 隐私与权限：`docs/operations/privacy-and-permissions.md`

## 当前技术栈

- Desktop shell: Tauri 2
- Frontend: Vue 3 + TypeScript + Vite
- State: Pinia
- Rust services: Tauri commands, process scan, settings, adapter discovery
- Tests: Vitest; Rust side 用 `cargo check` 和后续单测

## 目录约定

- `src/app/`：Tauri 窗口入口和应用组合。
- `src/components/island/`：悬浮岛压缩、展开、任务卡片和详情。
- `src/components/settings/`：独立设置和诊断窗口内容。
- `src/domain/`：跨 UI / Rust 语义一致的领域类型和纯函数。
- `src/stores/`：Pinia stores。
- `src/bridge/`：Tauri invoke / event 封装。
- `src-tauri/src/adapters/`：agent adapter 和诊断模型。
- `src-tauri/src/commands/`：Tauri commands。
- `src-tauri/src/services/`：本地系统能力。

## 实现约束

- 不要引入 React；前端使用 Vue。
- 悬浮岛只承载轻量状态浏览；设置、诊断等完整功能打开独立窗口。
- Hook 采集是旁路观测，不能影响 Claude Code / Codex 正常执行。
- 不静默修改用户 Claude Code / Codex 配置；hook 安装必须 dry-run、备份、用户确认、原子写入。
- 卸载 hook 只能删除 Agent Island 自己的 command，不能改动用户已有 hook。
- 不保存 prompt、回复正文、完整工具输入、完整工具输出、完整 shell command、完整 patch 或 transcript 内容。
- 修改 UI 后需要至少运行 `npm test -- --run` 和 `npm run build`；涉及 Rust 时运行 `cd src-tauri && cargo check`。

## 常用命令

```bash
npm test -- --run
npm run build
npm run tauri -- dev
cd src-tauri && cargo check
```

## 文档维护规则

- 新增功能先更新任务相关的短文档，再按需更新详细方案。
- 短文档只放稳定结论和链接；长设计放在 `docs/product/spec.md`、`docs/architecture/technical-plan.md` 或专项方案中。
- 任何架构级取舍写入 `docs/decisions/`。
- 对 AI 有用的入口变化同步更新 `llms.txt` 和 `docs/ai/context-map.md`。
