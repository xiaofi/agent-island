# Architecture Overview

Agent Island 采用 Vue 前端 + Tauri Rust 后台的本地架构。前端负责展示和交互，Rust 侧负责进程扫描、配置、窗口控制、adapter discovery 和后续 hook ingest。

## 系统分层

```text
Vue UI
  app windows
  island components
  settings and diagnostics
  Pinia stores
  domain helpers
        │ Tauri invoke / events
Rust Core
  commands
  adapters
  services
  aggregator
        │
Local Sources
  hook event spool
  process list
  candidate config/session paths
  app config
```

## 前端模块

- `src/app/`：窗口入口。`IslandApp.vue` 是悬浮岛，`FullWindowApp.vue` 是设置和诊断窗口。
- `src/components/island/`：压缩态、展开态、任务卡片、详情。
- `src/components/settings/`：设置和诊断面板。
- `src/domain/`：任务类型、排序、隐私处理等纯领域逻辑。
- `src/stores/`：Pinia store。
- `src/bridge/`：所有 Tauri invoke 和 event 订阅封装。

## Rust 模块

- `src-tauri/src/adapters/`：Codex、Claude Code、mock/manual adapter 和共享类型。
- `src-tauri/src/commands/`：Tauri commands，包括任务、设置、诊断、窗口。
- `src-tauri/src/services/`：进程扫描、配置读写、打开应用或目录等系统能力。
- `src-tauri/src/aggregator/`：任务合并、去重和状态推断。

## 数据流

```text
hook / discovery / mock
        │
Rust adapter or ingest service
        │ AgentTask / AgentEvent
        ▼
Tauri events and commands
        ▼
Pinia task store
        ▼
Island UI and diagnostics UI
```

## 当前真实采集策略

真实状态采集优先使用官方 hook。Hook 未启用、未 trust 或不可用时，系统降级到进程扫描和候选路径诊断。详细设计见 [hook-ingestion.md](hook-ingestion.md) 和 [hook-integration-plan.md](hook-integration-plan.md)。

完整技术方案见 [technical-plan.md](technical-plan.md)。
