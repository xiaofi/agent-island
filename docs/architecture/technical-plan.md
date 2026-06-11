# Agent Island Technical Plan

## 1. 目标与边界

Agent Island 首版目标是实现一个 macOS 优先的本地桌面悬浮状态层，用于展示 Claude Code、Codex 等本地 agent 会话的运行状态。它只做发现、归一化、展示和跳转，不接管 agent 的执行流程，不默认展示完整对话内容，也不向云端上传数据。

MVP 需要证明三件事：

- 桌面悬浮岛体验成立：透明、置顶、可拖拽、可展开、可收起。
- 状态模型能稳定承载多来源任务：Codex、Claude Code、mock/manual adapter。
- 真实状态采集优先通过官方 hook 旁路上报跑通；discovery 作为诊断和降级来源。

## 2. 技术栈

- 桌面壳：Tauri 2。
- 前端：Vue 3 + TypeScript + Vite。
- 样式：CSS variables + 原生 CSS 类，首版避免引入重型 UI 框架。
- 状态管理：Pinia。
- Rust 后台能力：Tauri commands、事件推送、文件监听、进程检测。
- 文件监听：`notify`。
- 进程检测：`sysinfo`。
- 配置持久化：Tauri app config 目录下的 JSON 文件。
- 测试：Vitest + Vue Test Utils；Rust 单元测试覆盖 adapter parser。

### 2.1 桌面壳选型调研

结论：MVP 继续推荐 Tauri 2，但需要明确它不是唯一选择。如果后续目标变成 App Store 分发或极致 macOS 原生窗口体验，应重新评估 Swift/AppKit；如果透明置顶、鼠标穿透在 Tauri 上遇到不可接受的问题，再切到 Electron 做兜底验证。

| 方案 | 优势 | 风险 / 代价 | 结论 |
| --- | --- | --- | --- |
| Tauri 2 | 体积小；Rust 侧适合做进程检测、文件监听、系统集成；支持无边框、置顶、透明、窗口效果等配置；与 Vue/Vite 集成直接。 | macOS 透明窗口需要 `macos-private-api`，不能上 Mac App Store；部分高级窗口行为仍需平台实测。 | 首选。适合本项目本地运行、macOS first、需要 Rust adapter 的定位。 |
| Electron | 窗口能力成熟；透明、无边框、置顶、鼠标穿透等 API 资料多；生态和调试体验强。 | 体积和内存占用明显更高；对一个小型常驻悬浮岛偏重；主进程安全和打包维护成本更高。 | 备选兜底。若 Tauri 窗口行为卡住，再用 Electron 快速验证。 |
| Wails | WebView + Go 后端；支持 frameless、transparent、AlwaysOnTop 等悬浮窗口模式；整体比 Electron 轻。 | 后端技术栈会从 Rust 切到 Go；本方案已依赖 Rust 生态里的 `notify`、`sysinfo` 和 Tauri command/event 模型。 | 仅当团队更偏 Go 时考虑，不作为当前首选。 |
| Neutralinojs | 很轻；能用本机 WebView 跑窗口；API 简单。 | 原生系统能力较薄，复杂进程扫描、文件监听、窗口控制通常要额外 native 扩展或 sidecar；长期 adapter 能力不如 Tauri 直接。 | 不推荐作为 MVP 主方案。 |
| Swift/AppKit | macOS 窗口控制最原生；透明、置顶、空间行为、激活策略可控性最好；运行开销低。 | macOS-only；UI 和状态层需要原生重写；后续跨平台弱；与 Rust/parser 共享代码需要额外桥接。 | 如果产品确认只做 macOS 原生工具，可作为 V2 方向；MVP 不优先。 |

关键判断：

- 当前产品是本地开发者工具，不依赖 App Store 分发，因此 Tauri 2 的 macOS 透明窗口限制可以接受，但必须在 M1 手动验收。
- 真实 adapter 需要频繁做本地文件、进程、路径和权限处理，Rust 侧能力与 Tauri 更匹配。
- 前端 UI 是轻量状态面板，Vue 3 + Pinia 足够，避免不必要的前端生态依赖。
- 保留 Electron 作为窗口能力兜底，不在一开始引入它的重量。

调研依据：

- Tauri 2 官方配置文档：窗口支持 `alwaysOnTop`、`decorations`、`transparent`、`windowEffects` 等配置；其中 macOS `transparent` 需要 `macos-private-api`。
- Electron 官方窗口文档：支持 frameless window、transparent window、BrowserWindow 窗口控制。
- Wails 官方 frameless 文档：支持 frameless、transparent background、AlwaysOnTop overlay 模式。
- Neutralinojs 官方文档：支持本机 window 模式和窗口拖拽 API，但系统集成深度比 Tauri / Electron 更薄。

## 3. 总体架构

```text
┌─────────────────────────────────────────────┐
│ Tauri Windows                               │
│  Vue UI                                     │
│  - Island collapsed view                    │
│  - Island expanded task list                │
│  - Island task detail view                  │
│  - Full settings window                     │
│  - Full diagnostics window                  │
│                                             │
│  Pinia Store                                │
│  - tasks                                    │
│  - events                                   │
│  - preferences                              │
│  - adapter diagnostics                      │
└───────────────────────▲─────────────────────┘
                        │ Tauri invoke / events
┌───────────────────────┴─────────────────────┐
│ Rust Core                                    │
│  - adapter registry                          │
│  - task aggregator                           │
│  - discovery commands                        │
│  - hook ingest service                        │
│  - watch service                             │
│  - privacy filtering                         │
│  - window commands                           │
└───────────▲───────────────────────▲─────────┘
            │                       │
┌───────────┴───────────┐   ┌───────┴─────────┐
│ Agent Adapters         │   │ System Services │
│ - mock/manual          │   │ - config store   │
│ - hook adapter         │   │ - file watch    │
│ - codex discovery      │   │ - hook install  │
│ - claude discovery     │   │ - open path/app │
└────────────────────────┘   └─────────────────┘
```

设计上采用前端负责展示和交互、Rust 侧负责本地系统能力与数据采集。adapter 输出统一的 `AgentTask` 与 `AgentEvent`，UI 不直接依赖 Codex 或 Claude Code 的私有数据结构。

## 4. 模块拆分

### 4.1 前端模块

建议目录：

```text
src/
  app/
    App.vue
    IslandApp.vue
    FullWindowApp.vue
  components/
    island/
      IslandCollapsed.vue
      IslandExpanded.vue
      TaskCard.vue
      TaskDetail.vue
    settings/
      SettingsPanel.vue
      DiagnosticsPanel.vue
    primitives/
      StatusDot.vue
      IconButton.vue
      DurationText.vue
  stores/
    taskStore.ts
    preferencesStore.ts
  composables/
    useTauriEvents.ts
    useDurationTicker.ts
  domain/
    taskTypes.ts
    taskPriority.ts
    privacy.ts
  bridge/
    tauriApi.ts
    eventBus.ts
  styles/
    tokens.css
```

关键职责：

- `taskStore`：Pinia store，维护任务、事件和 adapter 状态。
- `taskPriority`：实现压缩态排序规则，优先展示 `waiting-user`、`failed`、`tool-running`。
- `privacy`：根据隐私模式隐藏路径和标题。
- `tauriApi`：封装所有 `invoke` 和 Tauri event 监听，避免组件直接调用系统接口。
- `IslandCollapsed`：小尺寸常驻视图，只展示主任务状态和任务数量。
- `IslandExpanded`：悬浮岛下拉列表视图，展示活跃任务和次要任务。
- `TaskDetail`：悬浮岛内的单任务轻量详情，展示最近事件和快捷操作。
- `FullWindowApp`：承载设置、诊断等完整功能窗口。
- `DiagnosticsPanel`：在独立诊断窗口中展示 discovery 数据源、权限状态、解析结果和降级原因。

### 4.2 Rust 模块

建议目录：

```text
src-tauri/src/
  main.rs
  commands/
    mod.rs
    discovery.rs
    window.rs
    settings.rs
    task.rs
  adapters/
    mod.rs
    types.rs
    mock.rs
    codex.rs
    claude_code.rs
  services/
    file_watch.rs
    app_open.rs
    config_store.rs
    hook_installer.rs
    hook_ingest.rs
    hook_spool.rs
  aggregator/
    mod.rs
    task_state.rs
    event_normalizer.rs
```

关键职责：

- `adapters/types.rs`：定义 Rust 侧 `AgentTask`、`AgentEvent`、`AdapterDiagnostic`。
- `adapters/mock.rs`：读取本地 JSON 或内置样例，驱动 UI 开发。
- `adapters/codex.rs`：探测 Codex 配置文件和 hook 接入候选路径。
- `adapters/claude_code.rs`：探测 Claude Code 配置文件和 hook 接入候选路径。
- `aggregator`：合并 adapter 结果，去重、更新时间、推断 stale 状态。
- `services/file_watch.rs`：用 `notify` 监听 mock JSON 或真实日志变化。
- `services/hook_installer.rs`：预览、安装、卸载 Claude Code / Codex hook，保证备份、幂等和只删除自身条目。
- `services/hook_ingest.rs`：消费 hook helper 写入的本地 JSONL 事件，按来源接入设置过滤后归一化为 `AgentEvent`。
- `commands/discovery.rs`：暴露 `run_discovery(source?)` 给诊断页。
- `commands/window.rs`：处理拖拽、置顶、鼠标穿透、Dock 显示、显示隐藏、位置记忆。

## 5. 数据模型

前后端共享 TypeScript 与 Rust 两套类型，字段保持同构，序列化走 JSON。

```ts
type AgentSource = "codex" | "claude-code" | "manual";

type TaskStatus =
  | "discovering"
  | "running"
  | "thinking"
  | "tool-running"
  | "waiting-user"
  | "completed"
  | "failed"
  | "paused"
  | "stale";

interface AgentTask {
  id: string;
  source: AgentSource;
  title: string;
  cwd?: string;
  status: TaskStatus;
  startedAt?: string;
  updatedAt: string;
  durationSeconds?: number;
  lastAction?: string;
  waitingReason?: string;
  errorSummary?: string;
  windowHint?: WindowHint;
  events: AgentEvent[];
}
```

Hook 接入设置独立于 discovery，但和安装状态保持一致：开关打开表示 Agent Island hook 已安装并接收状态，开关关闭表示 Agent Island 自己的 hook command 已被卸载。

```ts
interface HookSourceSettings {
  codex: boolean;
  claudeCode: boolean;
  lastErrors: Partial<Record<AgentSource, HookOperationError>>;
}
```

默认两个来源都为 `false`。设置面板只有在 discovery 发现本机已安装对应工具时才显示开关。打开开关走安装预览和确认；关闭开关必须精确删除 Agent Island 自己的 hook command，成功后该来源的 hook payload 才不能再触发 Agent Island。

Hook 安装、卸载、修复、自检失败时，错误状态写入 `config.json` 和 `install-manifest.json`。设置窗口重新打开后必须恢复失败状态，并提供按失败动作映射的重试按钮。

补充建议：

- `id` 使用 `source + stable session key`，如果真实来源缺失 session id，则使用 `source + cwd + process id + startedAt`。
- `updatedAt` 由 adapter 事件时间或文件 mtime 推断。
- `durationSeconds` 可由前端定时派生，不必每秒从 Rust 推送。
- `events` 在 store 中按 `taskId` 单独索引，任务列表只保留最近摘要，避免重复渲染大量事件。

## 6. Adapter 设计

### 6.1 统一接口

Rust 侧可定义异步 trait：

```rust
#[async_trait]
pub trait AgentAdapter: Send + Sync {
    fn source(&self) -> AgentSource;
    async fn discover(&self) -> anyhow::Result<Vec<AgentTask>>;
    async fn diagnostics(&self) -> anyhow::Result<AdapterDiagnostic>;
    async fn open_task(&self, task_id: &str) -> anyhow::Result<()>;
}
```

watch 可以先不强制放进 trait。MVP 阶段先用轮询加文件监听组合实现，降低真实 adapter 不稳定时的复杂度。

### 6.2 Manual / Mock Adapter

用途：

- UI 开发和截图。
- 验证排序、详情、隐私模式和错误降级。
- 自动化测试不依赖用户机器上是否安装 Codex 或 Claude Code。

输入文件建议：

```text
~/.config/agent-island/mock-tasks.json
```

支持能力：

- 读取 3-5 个固定任务样例。
- 支持通过 JSON 修改任务状态，1 秒内反映到 UI。
- 支持模拟 waiting、failed、tool-running、completed 等状态。

### 6.3 Codex Discovery

首版不假设 Codex 私有数据结构稳定，先实现 discovery：

- 探测 `~/.codex/hooks.json`、`~/.codex/config.toml` 等配置文件。
- 检查候选文件是否存在、是否可读、最近修改时间。
- 对可读 JSONL、JSON、日志文件做轻量解析，只提取事件类型、时间、cwd、工具名和等待状态。
- 解析失败时返回 diagnostic，不让 UI 崩溃。

输出目标：

- `processes`: 不再用于安装判断，保留为空数组以兼容诊断模型。
- `candidatePaths`: 可读/不可读路径与原因。
- `parsedSessions`: 能归一化成 `AgentTask` 的会话摘要。
- `fallbackTask`: 如果只发现配置文件但没有 hook 事件，生成一个 `discovering` 降级任务。

### 6.4 Claude Code Discovery

首版策略与 Codex 一致：

- 探测 `~/.claude/settings.json`、`~/.claude/settings.local.json` 等配置文件。
- 只提取短摘要、工具类型、等待确认、错误摘要和时间。
- 不展示完整 prompt 或 assistant 回复正文。
- 无法解析时降级为“发现到 Claude Code 配置文件，但详细状态不可用”。

### 6.5 Hook Adapter

真实状态采集采用官方 hook 作为主路径，详见 [hook-integration-plan.md](hook-integration-plan.md)。

核心策略：

- Claude Code / Codex hook 只调用 Agent Island 本地 helper。
- helper 从 stdin 读取官方 hook JSON，做字段最小化和敏感内容过滤，然后追加到本地 JSONL spool。
- helper 不输出 stdout/stderr，不返回 decision、permissionDecision、additionalContext、systemMessage 等字段，并在所有异常路径返回 `exit 0`。
- Rust ingest service 监听 spool，把事件映射为 `AgentEvent` 和 `TaskStatus`。
- discovery 继续保留，用于配置文件发现、候选路径诊断、hook 未启用或未 trust 时的降级展示。

必须遵守的配置约束：

- 不静默安装 hook；必须由用户在设置窗口确认。
- 不修改用户已有 hook handler，不修改 hook 开关，不绕过 Codex trust。
- 所有写入先 dry-run、再备份、再原子写入。
- 卸载只删除 Agent Island 自己的 command；如果用户后续改过配置，不做整文件回滚。

## 7. 状态推断与排序

### 7.1 状态推断

状态来源优先级：

1. adapter 明确事件：`waiting-for-user`、`session-failed`、`tool-started`。
2. 最近事件类型：工具开始后未完成则为 `tool-running`。
3. 进程存在且最近有心跳则为 `running` 或 `thinking`。
4. 超过阈值未更新则为 `stale`。
5. 进程结束且最后事件正常收尾则为 `completed`。

建议阈值：

- `stale`: `updatedAt` 超过 10 分钟且无进程心跳。
- `completed` 展示保留：进入压缩态完成确认队列，用户确认后从压缩态移除并归档。
- UI 刷新：前端每秒更新运行时长，Rust 事件按变化推送。

### 7.2 压缩态排序

权重：

```text
waiting-user: 100
failed: 90
tool-running: 70
thinking: 60
running: 50
completed: 20
paused: 10
stale: 5
discovering: 1
```

同权重下按 `updatedAt` 倒序。压缩态文案格式：

```text
{sourceLabel} {statusLabel} · {activeCount} 个任务
```

当存在等待用户任务时，右侧显示强调标记；当存在未确认的 `completed` 任务时，压缩态按完成任务逐行追加确认项；隐私模式下只保留来源和状态。

## 8. Tauri 窗口行为

窗口配置：

- `decorations: false`。
- `transparent: true`。
- `alwaysOnTop: true`。
- `resizable: false`，展开态通过内容尺寸和窗口 resize 控制。
- 初始尺寸：压缩态约 `280 x 48`。
- 展开态尺寸：`400 x min(content, 60vh)`。

行为：

- 使用 Tauri drag region 实现拖拽；拖拽结束后保存当前物理坐标，不做边缘吸附。
- 首次启动默认定位到当前 monitor 工作区右上角；位置保存到本地配置后，启动时恢复。保存位置不在当前可见屏幕内时，仅在启动恢复阶段重定位到可用屏幕内。
- 点击压缩态后，主窗口根据当前 monitor 工作区剩余空间向下或向上展开，只展示任务列表和任务详情；靠近屏幕底部时保持底边锚点，让面板向上打开。
- 设置、诊断等完整功能从悬浮岛按钮打开独立普通桌面窗口。
- 快捷键切换显示/隐藏。
- 鼠标穿透默认关闭；开启后仅通过快捷键或 hover 策略临时接收事件。
- Dock 栏显示默认关闭；用户打开后通过 Tauri `set_dock_visibility` 立即显示 Dock 图标，启动时按持久化配置恢复。

需要重点验证 macOS 下透明窗口、阴影、鼠标穿透、Dock 显示和拖拽之间的兼容性。

## 9. UI 方案

视觉方向：

- 深色半透明胶囊。
- 细边框、轻微 blur、克制阴影。
- 状态色只用于小圆点、边缘提示和等待状态强调。
- 动画控制在 120-180ms。

主要视图：

- 压缩态：状态点、主状态文本、任务数/等待标记；未确认完成任务逐行显示确认项。
- 悬浮岛展开态：下拉任务卡片列表，最多优先展示 5-7 个活跃任务。
- 悬浮岛详情态：轻量元信息、最近事件、错误/等待原因、操作按钮。
- 独立设置窗口：隐私模式、路径隐藏、标题隐藏、快捷键、鼠标穿透、Dock 栏显示。
- 设置窗口还包含悬浮岛透明度、关键状态通知、通知声音和完成任务自动确认；这些属于本地 UI 偏好，不改变 agent 执行流程。
- 独立诊断窗口：adapter 来源、权限、候选路径、解析状态。

隐私模式：

- `hideProjectPath`: 路径显示为项目名或完全隐藏。
- `hideTaskTitle`: 标题显示为 `{sourceLabel} task`。
- `compactOnly`: 只影响压缩态，压缩态只显示来源和状态；展开列表继续按 `hideProjectPath` 和 `hideTaskTitle` 处理。

外观与提醒：

- `appearance.islandOpacity`: 固定应用到压缩态悬浮岛和展开任务面板背景，不使用整体 `opacity`，避免文字和状态点变淡。
- `notifications.enabled`: 控制 `waiting-user`、`failed`、`completed` 关键状态系统通知。用户开启时请求系统权限，Rust task watcher 统一在任务进入关键状态时发送通知；同一状态停留期间的后续同类事件不重复通知，启动或重新加载已有任务时不回放历史通知。
- `notifications.sound`: 控制关键状态通知声音。默认使用系统默认通知音；设置窗口提供 Basso、Ping、Glass、Hero、Pop、Sosumi、Tink 和无声选项；选择无声时仍发送通知但不设置声音。
- `autoAcknowledge.enabled` / `autoAcknowledge.delaySeconds`: 控制完成任务自动确认。到期后复用完成确认逻辑归档 `completed` 任务，不处理 `paused`、`waiting-user` 或 `failed`。
- `showInDock`: 控制 macOS Dock 图标是否显示。默认关闭，切换时通过 Rust window command 立即应用，启动时按本地配置恢复。

## 10. 本地配置与权限说明

配置文件建议：

```text
~/Library/Application Support/Agent Island/config.json
```

配置内容：

- 窗口位置。
- 隐私模式。
- enabled adapters。
- mock JSON 路径。
- discovery 候选路径覆盖。
- 快捷键。
- 鼠标穿透选项。
- Dock 栏显示选项。

首次启用 adapter 时，诊断页需要展示将读取的范围：

- 进程列表。
- 候选配置目录。
- 候选日志/会话文件。

读取失败必须展示明确原因，例如无权限、路径不存在、格式不支持。

## 11. 命令与事件接口

Tauri commands：

```ts
get_tasks(): Promise<AgentTask[]>
run_discovery(source?: AgentSource): Promise<AdapterDiagnostic[]>
get_settings(): Promise<AppSettings>
update_settings(patch: Partial<AppSettings>): Promise<AppSettings>
get_hook_install_status(): Promise<HookInstallStatus[]>
preview_hook_install(source: AgentSource, scope: "user" | "project"): Promise<HookInstallPreview>
install_hooks(source: AgentSource, scope: "user" | "project"): Promise<HookInstallResult>
uninstall_hooks(source: AgentSource, scope: "user" | "project"): Promise<HookUninstallResult>
open_task(taskId: string): Promise<void>
open_workdir(path: string): Promise<void>
copy_task_summary(taskId: string): Promise<void>
set_mouse_passthrough(enabled: boolean): Promise<void>
set_dock_visibility(visible: boolean): Promise<void>
set_window_mode(expanded: boolean): Promise<void>
open_app_window(kind: "settings" | "diagnostics"): Promise<void>
toggle_window_visibility(): Promise<void>
```

Tauri events：

```text
agent-task-updated
agent-task-removed
agent-event-created
adapter-diagnostic-updated
settings-updated
hook-install-status-updated
```

前端启动后先调用 `get_tasks`，再订阅事件。adapter 出错时推送 diagnostic，不抛到 UI 根组件。

## 12. 测试策略

详细测试分层、macOS E2E 边界和 native smoke 规则见
[../development/e2e-testing.md](../development/e2e-testing.md)。涉及新行为契约时，先按
[../development/spec-driven-development.md](../development/spec-driven-development.md)
在 `docs/specs/` 建立需求、设计、任务和验收映射。

前端：

- `taskPriority` 单元测试：确保 waiting、failed、tool-running 排序正确。
- `privacy` 单元测试：确保路径和标题隐藏。
- `TaskCard.vue` 组件测试：不同状态展示正确。
- mock adapter 集成测试：JSON 变化后 store 更新。

Rust：

- adapter parser 单元测试：输入样例日志，输出 `AgentTask`。
- config store 测试：读写和默认值迁移。
- aggregator 测试：去重、stale 推断、completed 保留策略。

手动验收：

- 启动后出现置顶悬浮岛。
- 可拖拽移动并重启后恢复位置。
- mock adapter 下至少展示 3 个任务。
- 修改 mock JSON 后 1 秒内更新 UI。
- waiting-user 任务提升到压缩态主展示。
- 开启隐私模式后隐藏完整路径和标题。
- adapter 失败时 UI 显示降级状态。

## 13. 里程碑实施计划

### M0: 方案与线框

- 补充 `docs/ui-wireframe.md`，定义压缩态、展开态、详情态。
- 明确字段展示密度和交互路径。
- 准备 mock task JSON 样例。

### M1: Tauri 壳

- 初始化 Tauri 2 + Vue 3 + TypeScript + Vite。
- 配置透明、置顶、无边框窗口。
- 实现拖拽、展开/收起、位置记忆。
- 建立基础 CSS tokens 和状态色。

### M2: 状态核心

- 实现共享 task/event 类型。
- 实现 Pinia store。
- 实现 mock adapter 和 Rust command。
- 实现任务排序、运行时长、隐私模式。

### M3: Discovery

- 实现 Codex discovery 命令。
- 实现 Claude Code discovery 命令。
- 增加 diagnostics 页面。
- 输出进程、候选路径、权限和解析结果。

### M4: 首个真实集成

- 实现 hook helper、JSONL spool 和 ingest service。
- 实现 Claude Code / Codex hook dry-run、安装、卸载和自检。
- 将真实 hook 事件归一化到 `AgentTask` / `AgentEvent`。
- 保留 adapter 降级路径。

### M5: 打磨与打包

- 增加快捷键。
- 增加打开应用、打开目录、复制摘要。
- 增加鼠标穿透选项。
- 完成 macOS 打包配置。
- 走一遍 MVP 验收清单。

## 14. 风险与应对

- Codex / Claude Code 本地数据结构不稳定：先做 discovery 和 diagnostics，不直接承诺完整解析。
- Hook 配置影响用户环境：不静默安装，先 dry-run 和备份；安装器只追加自身 command，卸载只删除自身 command。
- Hook 运行影响 agent：helper 不输出、不阻断、不注入上下文，异常也返回 `exit 0`，Codex trust 流程不绕过。
- 无法可靠跳回具体终端窗口：MVP 先支持打开工作目录，应用跳转作为增强能力。
- 透明置顶窗口和鼠标穿透在 macOS 上交互复杂：鼠标穿透默认关闭，并单独做手动验收。
- 隐私边界容易被 adapter 破坏：adapter 只产出摘要字段，完整正文不进入 store。
- 实时 watch 复杂度过高：MVP 可先用轮询，稳定后再针对确定数据源增加文件监听。

## 15. 推荐首批任务拆分

1. 初始化 Tauri 2 + Vue 3 项目骨架。
2. 创建 `AgentTask` / `AgentEvent` 类型和 mock JSON。
3. 实现压缩态、展开态、详情态 UI。
4. 实现 Pinia store 和任务优先级排序。
5. 实现 Rust mock adapter command。
6. 实现窗口拖拽、置顶、位置记忆。
7. 实现 diagnostics 页面骨架。
8. 实现 Codex / Claude Code discovery 命令。
9. 根据 discovery 结果选择首个真实 adapter。
10. 补齐隐私模式、快捷键和 macOS 打包。
