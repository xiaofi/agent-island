# AI Context Map

这个文件用于帮助 AI agent 在有限上下文内快速选择正确文档。不要把所有设计细节塞进这里；这里只做任务路由。

## 先读

所有任务先读：

1. [../../AGENTS.md](../../AGENTS.md)
2. [../../README.md](../../README.md)
3. 本文件

## 按任务选择上下文

| 任务 | 必读 | 按需读取 |
| --- | --- | --- |
| 修改悬浮岛 UI | [../product/overview.md](../product/overview.md), [../architecture/overview.md](../architecture/overview.md) | [../product/spec.md](../product/spec.md), `src/app/IslandApp.vue`, `src/components/island/` |
| 修改设置/诊断窗口 | [../product/overview.md](../product/overview.md), [../operations/privacy-and-permissions.md](../operations/privacy-and-permissions.md) | `src/app/FullWindowApp.vue`, `src/components/settings/` |
| 修改状态模型 | [../architecture/overview.md](../architecture/overview.md) | [../architecture/technical-plan.md](../architecture/technical-plan.md), `src/domain/taskTypes.ts`, `src-tauri/src/adapters/types.rs` |
| 做 Codex / Claude hook | [../architecture/hook-ingestion.md](../architecture/hook-ingestion.md), [../operations/privacy-and-permissions.md](../operations/privacy-and-permissions.md) | [../architecture/hook-integration-plan.md](../architecture/hook-integration-plan.md) |
| 做 discovery 或 adapter | [../architecture/overview.md](../architecture/overview.md) | `src-tauri/src/adapters/`, `src-tauri/src/services/process_scan.rs` |
| 改构建或运行方式 | [../development/getting-started.md](../development/getting-started.md) | `package.json`, `src-tauri/Cargo.toml`, `vite.config.ts` |
| 改文档结构 | [../research/ai-friendly-documentation.md](../research/ai-friendly-documentation.md), [../decisions/0001-ai-friendly-documentation.md](../decisions/0001-ai-friendly-documentation.md) | [../README.md](../README.md), [../../llms.txt](../../llms.txt) |

## 项目不变量

- 前端使用 Vue 3，不使用 React。
- 悬浮岛和完整窗口分离：岛内只做轻量浏览，设置和诊断打开独立窗口。
- Hook 是状态采集主路径，但只能旁路观测，不能影响 agent 执行。
- Discovery 是诊断和降级路径，不依赖私有本地数据结构作为唯一稳定接口。
- 隐私默认收敛：不采集完整对话、完整工具输入输出或 transcript 内容。

## 常用验证

```bash
npm test -- --run
npm run build
cd src-tauri && cargo check
```
