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
   - 需要关注的任务状态。
   - 多个进行中任务的合并状态。
   - 显示全部任务 / 收起全部任务入口。
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
- 中间：单条关注任务文本，或进行中任务合并文本。
- 右侧：任务列表入口，收起时显示“显示全部任务”，展开后显示“收起全部任务”。
- `waiting-user`、`failed`、`completed`、`stale` 视为需要关注的任务，优先在压缩态展示。
- `paused` 表示用户主动暂停或中断，不触发压缩态关注提示，也不继续显示在展开任务列表；后续同一会话恢复运行时重新显示。
- `discovering`、`running`、`thinking`、`tool-running` 合并为“N 个任务进行中”，用户不需要在压缩态逐条区分进行中任务。
- 没有或只有一条关注任务时保持单行悬浮岛形态；超过一条关注任务时切换为卡片式多行。
- 多条关注任务时，上方逐条展示关注任务，底部固定展示完整列表入口；有进行中任务时入口文本展示进行中合并项，没有进行中任务时展示“X 条任务需要关注”。
- 未确认的 `completed` 任务展示确认按钮；用户点击确认后，这些完成任务从压缩态移除并归档。
- 开启自动确认后，未确认的 `completed` 任务会在用户设置的延迟后自动归档；自动归档与手动确认使用同一套完成确认逻辑。
- 详情态提供“清除任务状态”兜底操作，用于把当前异常展示任务按完成且已确认归档；归档键绑定当前 `task.id + updatedAt`，后续同一任务收到新状态事件时重新显示。
- 压缩态和展开任务列表使用同一套紧凑字号层级，优先展示更多任务标题和最近动作。

示例：

```text
Claude · 等待处理 · 检查 API server 测试失败      显示全部任务

Claude · 等待处理 · 检查 API server 测试失败
Manual · 已完成 · 打包前检查清单                  ✓
2 个任务进行中                                    显示全部任务
```

### 6.2 展开态

展开态用于查看任务列表。展开方向根据悬浮岛所在屏幕位置自适应：下方空间足够时向下展开；悬浮岛靠近屏幕底部、下方空间不足时向上展开，并保持压缩态触发区域贴近原位置。

每个任务卡片展示：

- 来源：Codex / Claude Code。
- 任务标题：从当前对话目标、最近用户输入或工作目录推断。
- 状态：running / waiting / tool / completed / failed / idle。
- 最近动作：例如“读取文件”“运行测试”“等待命令批准”。
- 运行时长。
- 工作目录。
- 字号与压缩态保持一致的紧凑层级，减少单卡高度并展示更多标题和最近动作。

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
  - 清除任务状态：将当前任务展示状态视作已完成并确认，从列表中移除；只影响 Agent Island 的本地展示归档，不修改 agent、hook spool 或原始事件。

### 6.4 完整功能窗口

悬浮岛只承载轻量状态浏览，不承载完整设置或诊断流程。

展开面板右上角提供诊断、设置和退出应用按钮。设置、诊断等完整功能通过悬浮岛中的按钮打开独立桌面窗口：

- 设置窗口：悬浮岛透明度、关键状态通知、自动确认、安静模式、Dock 栏显示、状态采集、当前版本、GitHub 地址、简要更新说明和 GitHub Releases 链接。
- 诊断窗口：adapter 数据源、权限状态、候选路径和解析结果。
- 退出应用：关闭 Agent Island 程序，不修改 agent 任务、hook 配置或本地状态。
- 后续历史归档、adapter 管理等复杂功能也进入独立窗口。

### 6.5 状态采集设置

设置窗口增加“状态采集”区域，按来源分开展示 Claude Code 和 Codex。每个来源都是独立卡片，不互相隐式启用。设置面板先运行 discovery：

- 如果发现本机已安装 Claude Code，显示 Claude Code 卡片和开关；未发现时只显示“未发现 Claude Code 安装”，不显示开关。
- 如果发现本机已安装 Codex，显示 Codex 卡片和开关；未发现时只显示“未发现 Codex 安装”，不显示开关。
- Claude Code 开关文案为“接入 Claude Code 状态”。
- Codex 开关文案为“接入 Codex 状态”。

每个来源卡片展示：

- 一体化开关：打开表示已安装 Agent Island hook 并接收该来源状态；关闭表示已卸载 Agent Island 自己的 hook command。
- 安装状态：未接入 / 接入中 / 已接入 / 需信任 / hooks disabled / 配置不可写 / 安装失败 / 卸载失败 / 自检失败。
- 最近事件时间：只显示归一化后的时间和状态，不显示 prompt、回复、完整工具输入或输出。
- 操作按钮：预览变更、修复接入、运行自检；失败状态下显示“重试”。

交互规则：

1. 未发现某来源安装时，不显示开关，避免用户打开一个无法工作的接入。
2. 用户未打开某来源开关时，Agent Island 不安装该来源 hook，不接收该来源 hook，也不展示该来源真实 hook 任务。
3. 打开开关时，先展示该来源会写入或读取的本地路径、将新增的 hook command 和 dry-run diff；用户确认后才安装或修复。
4. 关闭开关时，立即执行卸载流程，只删除 Agent Island 自己的 hook command。卸载成功后才显示为关闭。
5. 如果关闭开关时卸载失败，开关回到打开状态并显示“卸载失败”，因为 Agent Island command 仍可能被 Claude Code / Codex 调用。
6. 关闭后，Claude Code / Codex 配置中不再有 Agent Island hook command；对对应 AI agent 不应再有任何运行时影响。
7. Codex 和 Claude Code 相互独立。打开 Claude Code 接入不会接入 Codex；打开 Codex 接入也不会接入 Claude Code。
8. 安装、卸载、修复或自检失败后，失败状态必须持久化到本地状态。用户关闭再打开设置窗口时，仍能看到失败来源、失败动作、失败时间和简短原因。
9. 失败状态不能只通过 toast 展示一次。toast 可以作为即时反馈，但设置卡片必须保留错误提示，直到用户重试成功、修复成功或明确清除。
10. 失败状态下显示“重试”按钮。安装失败时重试安装；卸载失败时重试卸载；自检失败时重试自检；修复失败时重试修复。

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
5. `completed`：进入压缩态完成确认队列，用户确认后归档。
6. `stale`：折叠到次要区域。

`paused` 表示用户已经中断本轮任务，是默认归档隐藏的状态，不参与压缩态或展开列表排序。

## 9. 数据来源与适配器

项目采用 adapter 架构，每个工具独立实现发现和状态解析。真实状态采集优先采用 Claude Code / Codex 官方 hook：hook 只做旁路观测，将生命周期事件写入 Agent Island 本地事件队列；配置文件 discovery 和候选路径检测作为 diagnostics 与降级来源。

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
- 检测 Codex 配置文件，作为 hook 未启用或未信任时的接入判断和诊断来源。
- 探测本机可用的 Codex hook 配置和候选路径，用于诊断，不作为稳定状态主接口。
- 将当前目标、工作目录、最近工具调用和等待状态归一化为 `AgentTask`。

实现注意：

- Codex 的本地数据结构可能随版本变化，MVP 应先做 discovery 工具，显式输出可用路径和事件格式。
- 适配器不应硬编码单一私有路径；需要支持配置和版本探测。
- 不修改用户已有 hook；安装 hook 必须先 dry-run、备份、原子写入，卸载时只删除 Agent Island 自己的 command。
- Hook helper 不输出 stdout/stderr，不返回任何会影响 Codex 行为的 decision/context 字段，异常时也返回成功。
- 无法可靠解析时，应降级为“发现到 Codex 配置文件，但详细状态不可用”。

### 9.2 Claude Code Adapter

首版目标：

- 接入 Claude Code 官方 hook，采集 `SessionStart`、`UserPromptSubmit`、`PreToolUse`、`PermissionRequest`、`Notification`、`PostToolUse`、`Stop`、`SessionEnd` 等生命周期事件。
- 检测 Claude Code 配置文件，作为 hook 未启用时的接入判断和诊断来源。
- 根据 hook 事件或配置路径推断工作目录和状态，用于诊断，不作为稳定主接口。
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
- 首次启用每个 adapter 或 hook 接入来源时，明确说明将读取哪些本地路径、进程信息或写入哪些 hook 配置。
- Claude Code 和 Codex 的 hook 接入必须分别由用户打开；未打开的来源不能安装 Agent Island hook、接收、落盘或展示真实 hook 状态。
- 不提供隐藏项目路径、隐藏任务标题或压缩态隐私模式设置；任务列表展示归一化后的任务标题和工作目录信息。
- 关键状态通知默认关闭；开启后只发送来源、等待处理/失败/完成状态和任务标题，不发送 prompt、回复正文、完整工具输入输出、完整命令或 patch。用户可选择系统默认通知音、若干 macOS 内置声音或无声。
- 自动确认只归档 `completed` 任务，不处理 `paused`、`waiting-user` 或 `failed`。

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
- 默认出现在屏幕工作区右上角，后续记住上次位置。
- 支持压缩态和展开态动画。
- 支持快捷键显示/隐藏。
- 支持“显示在 Dock 栏”选项：
  - 默认关闭，保持后台悬浮工具形态。
  - 开启后应用显示 Dock 图标，便于从 Dock 切回。
- 支持固定透明度设置：
  - 应用于压缩态悬浮岛和展开任务面板背景。
  - 不降低文字、状态点和按钮本身的不透明度，避免可读性下降。
- 支持关键状态通知：
  - 用户打开后请求系统通知权限。
  - 任务进入 `waiting-user`、`failed` 或 `completed` 时发送一条系统通知，并使用用户选择的通知声音。
  - 通知声音默认使用系统默认通知音；用户可选择 Basso、Ping、Glass、Hero、Pop、Sosumi、Tink 或无声。
  - 任务停留在同一关键状态期间，后续同类状态事件不重复通知；应用启动或重新加载已有任务时不回放历史通知。
- 支持完成任务自动确认：
  - 用户可选择 5 分钟、15 分钟、30 分钟或 1 小时。
  - 到期后未手动确认的完成任务自动归档。
  - 同一任务后续新的完成事件仍会重新进入完成确认队列。

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

- 加入快捷键。
- 加入打开工作目录 / 应用跳转。
- 打包 macOS app。

## 16. 验收标准

- 应用启动后出现一个置顶悬浮岛。
- 悬浮岛不遮挡主要工作流，且可拖拽移动。
- mock adapter 下可以同时展示至少 3 个任务。
- 状态变化在 1 秒内反映到 UI。
- 等待用户的任务会被提升到压缩态主展示位置。
- adapter 失败时 UI 不崩溃，并显示明确降级状态。

## 17. 风险与待验证问题

- Claude Code 和 Codex 是否都有稳定、可读取的本地会话事件源。
- 不同版本的本地日志或会话格式是否兼容。
- macOS 透明置顶窗口、Dock 显示和拖拽之间的交互细节。
- 如何可靠跳回对应终端或桌面应用窗口。
- 在不读取完整对话内容的前提下，能否生成足够有用的任务标题。

## 18. 下一步建议

1. 先做 `docs/ui-wireframe.md`，确定悬浮岛三种形态。
2. 再初始化 Tauri 项目和 mock adapter。
3. 同时写一个 `agent-discovery` 小命令，探测本机 Claude Code / Codex 可用的数据来源。
4. 根据 discovery 结果决定优先实现 Codex adapter 还是 Claude Code adapter。
