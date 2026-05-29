# Agent Island Spec

## 1. 项目概述

Agent Island 是一个轻量级桌面悬浮面板，用于集中展示 Claude Code、Codex 等本地 AI 编程代理正在执行的对话任务和当前状态。

它不是传统意义上的后台任务管理器，也不尝试接管 Claude Code 或 Codex 的运行流程。它的定位是一个“正在发生什么”的可视化状态层：让用户在不切回终端、IDE 或 Codex 桌面窗口的情况下，快速知道当前哪些代理会话还在跑、跑到哪一步、是否在等待输入、是否失败或完成。

## 2. 产品目标

- 以悬浮岛形式常驻桌面，尺寸小、视觉精细、信息密度适中。
- 展示当前活跃的 Claude Code / Codex 会话任务。
- 展示每个任务的简短标题、来源工具、工作目录、当前状态、运行时长和最近动作。
- 明确区分“正在思考 / 正在执行工具 / 等待用户 / 已完成 / 出错 / 已暂停”等状态。
- 支持点击展开详情，查看最近几条状态事件。
- 支持快速跳转回相关终端、窗口或项目目录。
- 首版优先 macOS，本地运行，不依赖云端服务。

## 3. 非目标

- 不替代 Claude Code、Codex 或终端本身。
- 不尝试读取或展示完整对话内容，除非用户显式授权。
- 不做复杂项目管理、排期、提醒或多人协作。
- 不做跨机器同步。
- 不在 MVP 中修改代理任务、取消任务或注入指令。

## 4. 目标用户

- 同时使用 Claude Code、Codex、终端和编辑器处理多个代码任务的开发者。
- 经常让多个 agent 会话并行跑，希望降低上下文切换成本的用户。
- 希望知道 agent 是“还在正常工作”还是“卡住等待输入”的用户。

## 5. 核心使用场景

1. 用户启动一个 Claude Code 任务和一个 Codex 任务。
2. Agent Island 自动发现两个活跃会话。
3. 悬浮岛压缩态显示：
   - 活跃任务数量。
   - 最重要任务的当前状态。
   - 是否有任务需要用户处理。
4. 用户点击悬浮岛展开详情。
5. 展开面板列出所有活跃任务：
   - `Codex · agent-island · 正在编辑 docs/product/spec.md · 03:12`
   - `Claude Code · api-server · 等待确认命令 · 08:45`
6. 用户点击某个任务，跳回对应应用或终端窗口。

## 6. 信息架构

### 6.1 压缩态

压缩态是默认形态，类似一个小型动态岛。

展示内容：

- 左侧：来源状态点或小图标。
- 中间：最重要的一条状态文本。
- 右侧：活跃任务数量或等待处理标记。

示例：

```text
Codex 正在运行 · 2 个任务
Claude 等待确认 · 1
```

### 6.2 展开态

展开态用于查看任务列表。

每个任务卡片展示：

- 来源：Codex / Claude Code。
- 任务标题：从当前对话目标、最近用户输入或工作目录推断。
- 状态：running / waiting / tool / completed / failed / idle。
- 最近动作：例如“读取文件”“运行测试”“等待命令批准”。
- 运行时长。
- 工作目录。

### 6.3 详情态

详情态用于查看单个任务最近事件。

展示：

- 任务元信息。
- 最近 5-10 条状态事件。
- 最近一次错误或等待原因。
- 操作按钮：
  - 打开应用。
  - 打开工作目录。
  - 复制任务摘要。

### 6.4 完整功能窗口

悬浮岛只承载轻量状态浏览，不承载完整设置或诊断流程。

设置、诊断等完整功能通过悬浮岛中的按钮打开独立桌面窗口：

- 设置窗口：隐私模式、路径隐藏、标题隐藏、快捷键、鼠标穿透。
- 诊断窗口：adapter 数据源、权限状态、候选路径和解析结果。
- 后续历史归档、adapter 管理等复杂功能也进入独立窗口。

## 7. 状态模型

### 7.1 Task

```ts
type AgentSource = "codex" | "claude-code";

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

### 7.2 Event

```ts
type AgentEventType =
  | "session-started"
  | "user-message"
  | "assistant-thinking"
  | "tool-started"
  | "tool-finished"
  | "waiting-for-user"
  | "session-completed"
  | "session-failed"
  | "heartbeat";

interface AgentEvent {
  id: string;
  taskId: string;
  type: AgentEventType;
  timestamp: string;
  summary: string;
  metadata?: Record<string, unknown>;
}
```

## 8. 任务优先级规则

压缩态只能展示有限信息，需要排序。

优先级从高到低：

1. `waiting-user`：需要用户介入。
2. `failed`：刚失败，需要关注。
3. `tool-running`：正在执行命令、测试、编辑等动作。
4. `thinking` / `running`：正常运行中。
5. `completed`：短暂显示后归档。
6. `stale` / `paused`：折叠到次要区域。

## 9. 数据来源与适配器

项目采用 adapter 架构，每个工具独立实现发现和状态解析。真实状态采集优先采用 Claude Code / Codex 官方 hook：hook 只做旁路观测，将生命周期事件写入 Agent Island 本地事件队列；进程扫描、候选路径检测和本地会话文件解析作为 diagnostics 与降级来源。

```ts
interface AgentAdapter {
  source: AgentSource;
  discover(): Promise<AgentTask[]>;
  watch(onEvent: (event: AgentEvent) => void): Promise<() => void>;
  openTask(taskId: string): Promise<void>;
}
```

### 9.1 Codex Adapter

首版目标：

- 接入 Codex 官方 hook，采集 `SessionStart`、`UserPromptSubmit`、`PreToolUse`、`PermissionRequest`、`PostToolUse`、`Stop` 等生命周期事件。
- 检测正在运行的 Codex 相关进程，作为 hook 未启用或未信任时的降级来源。
- 探测本机可用的 Codex 会话、线程或终端状态来源，用于诊断，不作为稳定主接口。
- 将当前目标、工作目录、最近工具调用和等待状态归一化为 `AgentTask`。

实现注意：

- Codex 的本地数据结构可能随版本变化，MVP 应先做 discovery 工具，显式输出可用路径和事件格式。
- 适配器不应硬编码单一私有路径；需要支持配置和版本探测。
- 不修改用户已有 hook；安装 hook 必须先 dry-run、备份、原子写入，卸载时只删除 Agent Island 自己的 command。
- Hook helper 不输出 stdout/stderr，不返回任何会影响 Codex 行为的 decision/context 字段，异常时也返回成功。
- 无法可靠解析时，应降级为“发现到 Codex 正在运行，但详细状态不可用”。

### 9.2 Claude Code Adapter

首版目标：

- 接入 Claude Code 官方 hook，采集 `SessionStart`、`UserPromptSubmit`、`PreToolUse`、`PermissionRequest`、`Notification`、`PostToolUse`、`Stop`、`SessionEnd` 等生命周期事件。
- 检测正在运行的 Claude Code 会话，作为 hook 未启用时的降级来源。
- 根据本地会话记录、日志、shell 进程或配置路径推断工作目录和状态，用于诊断，不作为稳定主接口。
- 识别是否正在等待用户确认、运行工具、生成响应或已经停止。

实现注意：

- 不默认展示完整 prompt 或回复正文。
- 只提取短摘要、状态、时间和工具类型。
- 不修改用户已有 hook；安装 hook 必须先 dry-run、备份、原子写入，卸载时只删除 Agent Island 自己的 command。
- Hook helper 不输出 stdout/stderr，不返回任何会影响 Claude Code 行为的 decision/context 字段，异常时也返回成功。
- 本地路径和日志格式需要通过 discovery 阶段确认，不把未验证路径写死为唯一来源。

### 9.3 Manual Adapter

为了让 UI 和状态模型先跑通，提供一个手动或 mock adapter：

- 支持本地 JSON 文件输入。
- 支持模拟多个任务状态变化。
- 用于开发、截图和测试。

## 10. 隐私与权限

- 默认只在本机运行。
- 默认不上传任何会话信息。
- 默认不展示完整对话文本。
- 首次启用每个 adapter 时，明确说明将读取哪些本地路径或进程信息。
- 提供隐私模式：
  - 隐藏项目路径。
  - 隐藏任务标题。
  - 只显示来源和状态。

## 11. 技术方案

建议技术栈：

- 桌面壳：Tauri 2。
- UI：Vue 3 + TypeScript。
- 样式：CSS variables + 原生 CSS 类，后续按复杂度决定是否引入 CSS Modules。
- 本地状态：Pinia。
- 后台采集：Tauri Rust sidecar / commands。
- 文件监听：Rust notify。
- 进程检测：Rust sysinfo。
- 打包目标：macOS first，后续扩展 Windows / Linux。

选择 Tauri 的原因：

- 比 Electron 更轻，适合小型常驻桌面插件。
- 易于实现透明、置顶、无边框窗口。
- Rust 侧适合做文件监听、进程检测和系统集成。

## 12. 桌面窗口行为

- 默认置顶。
- 无边框、透明背景。
- 可拖拽移动。
- 记住上次位置。
- 支持压缩态和展开态动画。
- 支持快捷键显示/隐藏。
- 支持“鼠标穿透”选项：
  - 默认关闭，避免用户无法点击。
  - 开启后只在 hover 或快捷键唤醒时接收事件。

## 13. 视觉方向

关键词：

- 精细。
- 安静。
- 低打扰。
- 状态清晰。
- 不像通知中心，不像任务管理器。

设计原则：

- 深色半透明胶囊形态。
- 使用细腻边框、阴影和轻微 blur。
- 状态色只用于关键反馈，不大面积铺色。
- 动画短、克制，避免持续闪烁。
- 压缩态宽度约 220-320px，高度约 40-52px。
- 展开态宽度约 360-440px，高度不超过 60vh。

状态色建议：

- running：蓝色或青色点。
- tool-running：琥珀色点。
- waiting-user：橙色或红色强调。
- completed：绿色点，短暂显示。
- failed：红色点。
- stale：灰色点。

## 14. MVP 范围

MVP 只需要证明三件事：

1. 悬浮岛作为桌面窗口体验成立。
2. 状态模型能承载 Codex / Claude Code 的任务展示。
3. 至少一个真实 adapter 或 discovery 工具能识别本机 agent 会话。

MVP 功能：

- Tauri 桌面应用。
- 透明置顶悬浮窗口。
- mock adapter。
- 任务列表和详情展开。
- 独立设置窗口。
- 独立 adapter diagnostic 窗口。
- Codex / Claude Code discovery 命令，输出可解析的数据来源。

暂不做：

- 自动控制 agent。
- 云同步。
- 复杂历史归档。
- 插件市场。
- 团队共享。

## 15. 里程碑

### M0: Spec 与原型

- 完成项目 spec。
- 画出压缩态、展开态、详情态结构。
- 用 mock 数据验证 UI 信息密度。

### M1: 桌面壳

- 搭建 Tauri + Vue 3 项目。
- 实现透明置顶窗口。
- 实现拖拽、展开、收起、位置记忆。

### M2: 状态核心

- 实现 `AgentTask` / `AgentEvent` store。
- 实现 mock adapter。
- 实现任务排序和状态推断。

### M3: 本地发现

- 实现 Codex discovery。
- 实现 Claude Code discovery。
- 输出 adapter diagnostic 页面，展示发现的数据源、权限状态和解析结果。

### M4: 首个真实集成

- 选择 Codex 或 Claude Code 中更稳定的数据源。
- 实现 watch。
- 在悬浮岛中实时更新任务状态。

### M5: 打磨

- 加入隐私模式。
- 加入快捷键。
- 加入打开工作目录 / 应用跳转。
- 打包 macOS app。

## 16. 验收标准

- 应用启动后出现一个置顶悬浮岛。
- 悬浮岛不遮挡主要工作流，且可拖拽移动。
- mock adapter 下可以同时展示至少 3 个任务。
- 状态变化在 1 秒内反映到 UI。
- 等待用户的任务会被提升到压缩态主展示位置。
- 隐私模式开启后，不显示完整路径和任务标题。
- adapter 失败时 UI 不崩溃，并显示明确降级状态。

## 17. 风险与待验证问题

- Claude Code 和 Codex 是否都有稳定、可读取的本地会话事件源。
- 不同版本的本地日志或会话格式是否兼容。
- macOS 透明置顶窗口与鼠标穿透的交互细节。
- 如何可靠跳回对应终端或桌面应用窗口。
- 在不读取完整对话内容的前提下，能否生成足够有用的任务标题。

## 18. 下一步建议

1. 先做 `docs/ui-wireframe.md`，确定悬浮岛三种形态。
2. 再初始化 Tauri 项目和 mock adapter。
3. 同时写一个 `agent-discovery` 小命令，探测本机 Claude Code / Codex 可用的数据来源。
4. 根据 discovery 结果决定优先实现 Codex adapter 还是 Claude Code adapter。
