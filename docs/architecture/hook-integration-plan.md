# Agent Island Hook Integration Plan

## 1. 目标

Agent Island 采用 Claude Code / Codex 官方 hook 作为真实状态采集的主路径。Hook 只做旁路观测：把生命周期事件转成最小化的本地状态事件，供悬浮岛展示当前任务是否在运行、执行工具、等待用户、完成或失败。

这套方案必须满足三个约束：

- 不接管 Claude Code / Codex 的执行流程。
- 不默认读取、保存或展示完整 prompt、回复正文、工具入参和工具输出。
- 不破坏用户已有 hook、settings、config、trust 状态；安装、升级、卸载都必须可审计、可回滚。

## 2. 官方能力依据

### 2.1 Claude Code

Claude Code hooks 是官方生命周期扩展点，支持 command、HTTP、prompt、agent、MCP tool 等类型。Hook 触发时会把 JSON 上下文传给 handler；command hook 通过 stdin 接收，HTTP hook 通过 POST body 接收。

Agent Island 只使用 command hook，原因是它可以在本地快速落盘并立即退出，不依赖 Agent Island 主程序是否正在运行。

可用关键信息：

- 公共字段：`session_id`、`transcript_path`、`cwd`、`permission_mode`、`hook_event_name`。
- 关键事件：`SessionStart`、`UserPromptSubmit`、`PreToolUse`、`PermissionRequest`、`PostToolUse`、`PostToolUseFailure`、`Notification`、`Stop`、`StopFailure`、`SessionEnd`、`CwdChanged`。
- 配置来源可来自 `~/.claude/settings.json`、`.claude/settings.json`、`.claude/settings.local.json`、插件 hooks 等。
- `exit 0` 且不输出 stdout 表示无决策，正常继续；多数非零错误也是非阻断，但 `exit 2` 在部分事件会阻断动作。

设计要求：

- Agent Island hook helper 永远不返回 `decision`、`permissionDecision`、`additionalContext`、`terminalSequence`。
- helper 正常和异常路径都返回 `exit 0`。
- helper 不向 stdout/stderr 输出内容，避免影响 Claude Code UI、上下文或 debug 噪音。

### 2.2 Codex

Codex hooks 是官方生命周期扩展点，默认启用。Codex 会从活动 config layer 旁边发现 `hooks.json` 或 inline `[hooks]` 配置，也可以加载已启用插件内的 hooks。多个 hook 来源会合并执行，高优先级配置不会替换低优先级 hooks。

可用关键信息：

- 公共字段：`session_id`、`transcript_path`、`cwd`、`hook_event_name`、`model`。
- 关键事件：`SessionStart`、`UserPromptSubmit`、`PreToolUse`、`PermissionRequest`、`PostToolUse`、`SubagentStart`、`SubagentStop`、`Stop`、`PreCompact`、`PostCompact`。
- `transcript_path` 可作为便利字段，但 transcript 格式不是稳定接口，不能把 transcript 解析作为主路径。
- 非 managed command hooks 需要用户 review/trust；这是 Codex 的安全机制，不能绕过或偷偷处理。

设计要求：

- 不使用 `statusMessage`，避免每次 hook 运行在界面上产生提示。
- 不输出 stdout/stderr，避免向 developer context 或 UI 注入内容。
- 不返回 `continue: false`、`stopReason`、`systemMessage`、`permissionDecision` 等会影响 Codex 行为或界面的字段。
- 如果 Codex 因未信任而跳过 Agent Island hook，Agent Island 只在诊断页提示，不影响 Codex 原任务。

## 3. 总体架构

```text
Claude Code / Codex
  lifecycle hook
        │ stdin JSON
        ▼
agent-island-hook helper
  - parse minimal fields
  - redact sensitive fields
  - append local JSONL spool
  - always exit 0
        │
        ▼
~/Library/Application Support/Agent Island/events/*.jsonl
        │ notify / cursor poll
        ▼
Rust Hook Ingest Service
  - read new events
  - dedupe
  - normalize
  - state machine
        │ Tauri event
        ▼
Vue / Pinia task store
        │
        ▼
Floating island / diagnostics window
```

核心取舍：

- Hook helper 不直接调用 Tauri command，不依赖 app 是否启动。
- 事件先进入 append-only 本地 JSONL，主程序启动后再消费。
- Discovery 仍保留，用于进程发现、路径诊断和 hook 未启用时的降级展示。
- Hook 接入按来源独立受用户设置控制；Claude Code 和 Codex 不互相隐式启用。

### 3.1 来源接入开关

Agent Island 区分三个状态：

1. **来源可发现**：可以做进程扫描和候选路径诊断。
2. **Agent Island hook 已安装**：用户配置中存在 Agent Island 自己的 hook command。
3. **来源接入已启用**：用户允许 Agent Island 安装该来源 hook 并接收真实 hook 事件。

设置开关采用一体化语义：打开表示“安装 Agent Island hook 并接收状态”，关闭表示“卸载 Agent Island 自己的 hook command”。关闭后，对应 Claude Code / Codex 配置中不应再调用 Agent Island helper。

设置模型建议：

```ts
interface HookSourceSettings {
  codex: boolean;
  claudeCode: boolean;
  lastErrors: Partial<Record<AgentSource, HookOperationError>>;
}

interface HookOperationError {
  operation: "install" | "uninstall" | "repair" | "self-test";
  code: string;
  message: string;
  occurredAt: string;
  retryAction: "install" | "uninstall" | "repair" | "self-test";
}
```

默认值为：

```json
{
  "hookSource": {
    "codex": false,
    "claudeCode": false,
    "lastErrors": {}
  }
}
```

运行时强制点：

- 设置窗口只在 discovery 发现本机安装了对应工具时显示开关；未发现安装时显示“未发现 Claude Code 安装”或“未发现 Codex 安装”，不提供开关。
- `set_hook_source_enabled(source, true)` 必须执行安装预览、用户确认、备份、原子写入和自检。
- `set_hook_source_enabled(source, false)` 必须执行卸载，精确删除 Agent Island 自己的 hook command；成功后才把设置改为 false。
- 如果卸载失败，设置保持 true，UI 显示“卸载失败”，因为 Agent Island hook 仍可能影响该来源运行。
- 安装、卸载、修复、自检失败必须写入持久化 `lastErrors[source]`，包括失败动作、错误码、简短原因、发生时间和可重试动作。
- 对应动作重试成功后必须清除该来源的 `lastErrors[source]`。
- `agent-island-hook --source codex` 在写入前仍读取 `hookSource.codex` 作为防御；为 false 时直接 `exit 0`。
- `agent-island-hook --source claude-code` 在写入前仍读取 `hookSource.claudeCode` 作为防御；为 false 时直接 `exit 0`。
- `hook_ingest.rs` 消费事件时再次按当前设置过滤来源；关闭开关后，旧 spool 或残留 hook 事件不能继续更新任务。
- `get_tasks()` 和 aggregator 不返回关闭来源的真实 hook 任务；diagnostics 可以展示安装残留、卸载失败或未发现安装。

## 4. 本地文件布局

```text
~/Library/Application Support/Agent Island/
  config.json
  hooks/
    agent-island-hook
    install-manifest.json
  events/
    codex.jsonl
    claude-code.jsonl
    dead-letter.jsonl
  cursors/
    codex.cursor
    claude-code.cursor
  backups/
    claude-settings-20260529T120000.json
    codex-hooks-20260529T120000.json
```

说明：

- `agent-island-hook` 是一个小型 CLI helper，作为 Tauri sidecar 或安装到 app support 目录。
- `install-manifest.json` 记录 Agent Island 修改过哪些文件、插入了哪些 command、对应备份路径、版本号和安装时间。
- `events/*.jsonl` 只保存归一化后的最小字段，不保存完整对话或工具输出。
- `dead-letter.jsonl` 保存解析失败的最小错误记录，不保存原始 payload。

## 5. Hook helper 行为

### 5.1 输入

helper 从 stdin 接收 Claude Code / Codex 的原始 hook JSON。

启动参数：

```text
agent-island-hook --source codex
agent-island-hook --source claude-code
```

### 5.2 输出

helper 不向 stdout 输出任何内容，不向 stderr 输出任何内容。所有成功、失败、超时、权限错误都转成内部日志或静默忽略，并返回 `exit 0`。

### 5.3 采集字段

默认允许写入 spool 的字段：

```json
{
  "schemaVersion": 1,
  "source": "codex",
  "event": "PreToolUse",
  "sessionKey": "hash(source + session_id + salt)",
  "turnKey": "hash(turn_id + salt)",
  "cwd": "/Users/spf/project/agent-island",
  "timestamp": "2026-05-29T12:00:00Z",
  "toolName": "Bash",
  "actionSummary": "running Bash",
  "permissionMode": "default",
  "rawEventFields": {
    "hasTranscriptPath": true,
    "hasPrompt": false,
    "hasToolInput": true
  }
}
```

默认禁止写入 spool 的字段：

- `prompt` 原文。
- `last_assistant_message` 原文。
- `tool_input.content`、完整 shell command、完整 patch、完整文件内容。
- `tool_response`、工具 stdout/stderr、报错堆栈全文。
- transcript 文件内容。

可选增强：

- `cwd` 可以采集完整路径，但 UI 根据隐私设置隐藏；后续可增加“只保存项目名”模式。
- shell command 只做分类，例如 `npm test` 归为 `Bash:npm`，不保存参数。
- 文件路径只保存 basename 或 repo-relative path，默认不保存绝对文件路径。

## 6. 状态映射

| Hook event | AgentEvent | TaskStatus | 说明 |
| --- | --- | --- | --- |
| `SessionStart` | `session-started` | `running` | 创建或恢复任务 |
| `UserPromptSubmit` | `user-message` | `thinking` | 不保存 prompt 原文，只记录有新用户输入 |
| `PreToolUse` | `tool-started` | `tool-running` | 记录工具名和动作摘要 |
| `PermissionRequest` | `waiting-for-user` | `waiting-user` | 需要用户确认 |
| Claude `Notification.permission_prompt` | `waiting-for-user` | `waiting-user` | Claude 通知层等待用户 |
| `PostToolUse` | `tool-finished` | `thinking` | 工具完成，回到思考/运行 |
| Claude `PostToolUseFailure` | `session-failed` 或 `tool-finished` | `failed` 或 `thinking` | 根据错误类型和后续事件判断 |
| `Stop` | `session-completed` | `completed` | 当前 turn 完成 |
| Claude `StopFailure` | `session-failed` | `failed` | API 或执行错误 |
| Claude `SessionEnd` | `session-completed` | `completed` | 会话结束 |
| `SubagentStart` | `session-started` | `running` | 可作为子任务或父任务事件 |
| `SubagentStop` | `session-completed` | `completed` | 子任务结束 |
| `CwdChanged` | `heartbeat` | 保持当前状态 | 更新 cwd |

状态机规则：

- `waiting-user` 优先级最高，直到收到新的 `UserPromptSubmit`、`PostToolUse`、`Stop` 或更新事件解除。
- `tool-running` 如果超过 10 分钟没有 `PostToolUse`，降级为 `stale`，但不判定失败。
- `completed` 保留 5 分钟后进入次要区域或隐藏。
- 只发现进程但没有 hook 事件时，保持 `discovering` 或 `running` 降级任务。

## 7. 安装策略

### 7.1 默认原则

安装 hook 必须由用户在 Agent Island 设置窗口中显式点击启用。Agent Island 不在后台静默修改任何 Claude Code / Codex 配置。

接入也必须按来源单独确认。用户只打开 Claude Code 接入时，Codex hook 仍然保持关闭；用户只打开 Codex 接入时，Claude Code hook 仍然保持关闭。

单个来源的启用流程：

1. discovery 确认本机存在对应工具；未发现时不显示开关，只显示未安装提示。
2. 扫描该工具是否支持 hook、现有配置是否可读写。
3. 生成 dry-run diff，在设置窗口展示将新增的 hook command 和目标文件。
4. 用户确认后，创建备份。
5. 使用结构化 JSON/TOML parser 合并配置。
6. 原子写入临时文件，再 rename 覆盖。
7. 写入 `install-manifest.json`。
8. 将该来源的 `hookSource` 设置为 true。
9. 运行自检：触发 helper `--self-test --source <source>`，确认启用时 spool 可写。
10. 安装和自检都成功后，清除该来源历史失败状态。

关闭接入流程：

1. 用户在设置窗口关闭某来源开关。
2. UI 将该开关置为 loading，不先展示为关闭。
3. 按 manifest 和当前配置精确删除 Agent Island 自己的 hook command；不删除、不改写用户已有 hook。
4. 如某个 matcher group 删除 Agent Island command 后为空，只删除这个空 group；其他用户配置保持原样。
5. 原子写入配置并更新 manifest。
6. 将该来源的 `hookSource` 设置为 false。
7. ingest 停止把该来源已有 spool 事件转成任务，UI 移除该来源真实 hook 任务。
8. 诊断页显示“未接入”。
9. 清除该来源历史失败状态。

卸载失败流程：

1. 如果配置不可写、JSON/TOML 无法解析、hash 冲突不可安全合并或原子写入失败，不能把开关显示为关闭。
2. UI 将开关恢复为打开状态，状态显示“卸载失败”，并在诊断页提供失败原因和手动删除指引。
3. helper 的本地设置防御可以临时阻止落盘，但这不能作为完成状态；只有配置里的 Agent Island command 删除成功，才算关闭成功。
4. 失败信息写入 `hookSource.lastErrors[source]` 和 `install-manifest.json` 的 lastError 字段。用户关闭再打开设置窗口时，仍显示卸载失败状态和“重试”按钮。

### 7.2 失败状态持久化

Hook 相关操作的失败不能只存在于一次性 toast。以下操作失败时都必须持久化：

- 安装。
- 卸载。
- 修复接入。
- 自检。

持久化位置：

- `config.json` 保存 UI 需要恢复的 `hookSource.lastErrors`。
- `install-manifest.json` 保存安装器审计需要的 `lastError`、最后一次目标文件、目标 hash 和建议重试动作。

展示规则：

- 设置窗口每次打开时读取持久化错误；如果某来源存在 `lastErrors[source]`，对应卡片显示失败状态、简短原因、发生时间和“重试”按钮。
- 诊断窗口显示同一份失败状态，并额外显示目标配置路径、是否可写、manifest hash 状态和手动处理建议。
- toast 只作为即时反馈，不能作为唯一错误载体。
- 用户重试成功后清除该来源失败状态；重试失败则用新的失败时间和原因覆盖旧失败。
- 用户可以点击“清除提示”，但只有在当前 discovery 确认没有 Agent Island command 残留，或用户明确选择忽略时才允许清除。清除提示不能修改真实配置。

### 7.3 优先级

优先采用官方插件 hook 机制，其次才修改用户级配置文件。

推荐顺序：

1. **插件 hook 层**：如果当前 Claude Code / Codex 版本支持插件 hooks，创建 Agent Island 插件并让用户启用/信任。插件和用户原配置分离，最容易卸载。
2. **用户级 hook 文件合并**：Codex 使用 `~/.codex/hooks.json`；Claude Code 使用 `~/.claude/settings.json` 的 `hooks` 节点。
3. **项目级 hook 文件合并**：只针对当前 repo 生效，适合用户不想全局启用时使用。

不使用 managed hooks，因为它面向企业策略，语义是强制执行，不适合个人本地悬浮岛。

### 7.3 Codex 配置合并

目标文件：

```text
~/.codex/hooks.json
```

新增结构示例：

```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "startup|resume|clear|compact",
        "hooks": [
          {
            "type": "command",
            "command": "\"/Users/spf/Library/Application Support/Agent Island/hooks/agent-island-hook\" --source codex",
            "timeout": 1
          }
        ]
      }
    ],
    "PreToolUse": [
      {
        "matcher": "*",
        "hooks": [
          {
            "type": "command",
            "command": "\"/Users/spf/Library/Application Support/Agent Island/hooks/agent-island-hook\" --source codex",
            "timeout": 1
          }
        ]
      }
    ]
  }
}
```

实际实现会为以下事件都追加同一个 helper command：

- `SessionStart`
- `UserPromptSubmit`
- `PreToolUse`
- `PermissionRequest`
- `PostToolUse`
- `SubagentStart`
- `SubagentStop`
- `Stop`

合并规则：

- 如果目标文件不存在，创建只包含 Agent Island hooks 的新文件。
- 如果目标文件存在，保留所有已有 key、已有 hook group、已有 command 顺序。
- 只追加缺失的 Agent Island command；重复安装不会生成重复项。
- 如果同时存在 inline `[hooks]` in `config.toml`，不迁移、不合并到 TOML，只在诊断页提示 Codex 会合并多个来源。
- 不修改 `[features].hooks`。如果用户已设置 `hooks = false`，尊重该设置，只显示“Codex hooks disabled”诊断。
- 不自动调用 `--dangerously-bypass-hook-trust`。如果 Codex 要求 trust，提示用户在 Codex `/hooks` 中 review/trust。

### 7.4 Claude Code 配置合并

目标文件：

```text
~/.claude/settings.json
```

新增结构示例：

```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "startup|resume|clear|compact",
        "hooks": [
          {
            "type": "command",
            "command": "\"/Users/spf/Library/Application Support/Agent Island/hooks/agent-island-hook\" --source claude-code",
            "timeout": 1
          }
        ]
      }
    ],
    "PreToolUse": [
      {
        "matcher": "*",
        "hooks": [
          {
            "type": "command",
            "command": "\"/Users/spf/Library/Application Support/Agent Island/hooks/agent-island-hook\" --source claude-code",
            "timeout": 1
          }
        ]
      }
    ]
  }
}
```

实际实现会为以下事件追加 helper command：

- `SessionStart`
- `UserPromptSubmit`
- `PreToolUse`
- `PermissionRequest`
- `PostToolUse`
- `PostToolUseFailure`
- `Notification`
- `Stop`
- `StopFailure`
- `SessionEnd`
- `CwdChanged`

合并规则：

- 如果用户设置了 `"disableAllHooks": true`，不修改该值，不强行启用。
- 不修改用户已有 hook，不改 matcher，不改 timeout，不改 command。
- 如果 settings JSON 无法解析，不写入，提供手动修复提示。
- 如果已有 Agent Island command 但路径是旧版本，按 manifest 做升级替换，只替换 Agent Island 自己的 command。

## 8. 卸载与回滚

卸载必须支持两种模式：

- **移除 Agent Island hook**：解析目标配置，只删除 command 精确匹配 Agent Island helper 路径的 handler；如果某个 matcher group 删除后为空，则删除该 group；其他用户配置保持原样。
- **恢复备份**：仅当用户明确选择“恢复安装前备份”时，才把备份文件覆盖回去。默认不这样做，因为用户可能在安装后又改过自己的配置。

回滚保护：

- 每次写配置前都创建备份。
- `install-manifest.json` 保存修改前文件 hash、修改后文件 hash、Agent Island command 列表。
- 卸载时如果当前文件 hash 和 manifest 里的修改后 hash 不一致，说明用户后来改过文件；此时只做精确删除，不做整文件恢复。
- 删除失败时不强行改写，诊断页显示手动删除命令。

## 9. 对用户无感知无影响的工程约束

运行时约束：

- helper 总是 `exit 0`。
- helper 不写 stdout/stderr。
- helper 不返回任何 decision/context/UI message 字段。
- helper timeout 设为 1 秒，内部目标耗时控制在 20ms 内。
- Agent Island 未运行、spool 不可写、磁盘满、JSON 解析失败时，helper 静默退出。
- 写 spool 使用 append-only 小 JSON 行，避免锁住 Claude Code / Codex 主流程。

配置约束：

- 不静默安装；用户必须在 Agent Island 里确认。
- 不修改用户已有 hook handler。
- 不修改 Codex / Claude Code 的 hook 开关。
- 不修改 Codex trust 状态，不绕过 trust review。
- 不写 repo 文件，除非用户选择项目级接入。
- 所有写入都可 dry-run、可备份、可卸载。

隐私约束：

- 默认不保存 prompt、回复、工具输出、完整命令、完整 patch、文件内容。
- 默认只保存 session hash、cwd、事件名、工具名、状态摘要和时间。
- 用 per-install salt 哈希 `session_id` / `turn_id`，避免事件文件直接暴露原始 session id。
- 诊断页只展示采集字段类型，不展示被丢弃的敏感字段内容。

需要诚实说明的边界：

- Codex 的非 managed hooks 需要用户 review/trust；这可能在 Codex 里出现一次性提示，不能也不应该绕过。
- Claude Code `/hooks` 菜单会显示已配置的 Agent Island hook，这是可审计性的一部分。
- “无感知”指不影响 agent 执行结果、不注入上下文、不弹运行提示、不改变用户已有配置语义；不是隐藏安装行为。

## 10. Rust / 前端模块设计

### 10.1 Rust 新增模块

```text
src-tauri/src/
  commands/
    hook.rs
  services/
    hook_installer.rs
    hook_helper.rs
    hook_ingest.rs
    hook_spool.rs
  adapters/
    hook_adapter.rs
```

职责：

- `hook_installer.rs`：扫描、dry-run、合并配置、备份、卸载。
- `hook_helper.rs`：helper 主逻辑，可复用于 sidecar binary。
- `hook_spool.rs`：append JSONL、rotate、cursor。
- `hook_ingest.rs`：监听 JSONL，归一化为 `AgentEvent`。
- `hook_adapter.rs`：把 hook events 合并进现有 adapter/aggregator。

### 10.2 Tauri commands

```ts
get_hook_install_status(): Promise<HookInstallStatus[]>
get_hook_source_settings(): Promise<HookSourceSettings>
set_hook_source_enabled(source: AgentSource, enabled: boolean): Promise<HookSourceSettings>
preview_hook_install(source: AgentSource, scope: "user" | "project"): Promise<HookInstallPreview>
install_hooks(source: AgentSource, scope: "user" | "project"): Promise<HookInstallResult>
uninstall_hooks(source: AgentSource, scope: "user" | "project"): Promise<HookUninstallResult>
run_hook_self_test(source: AgentSource): Promise<HookSelfTestResult>
```

### 10.3 前端入口

设置窗口增加 “状态采集” 区域：

- 未发现 Claude Code 安装时：显示“未发现 Claude Code 安装”，不显示 Claude Code 开关。
- 未发现 Codex 安装时：显示“未发现 Codex 安装”，不显示 Codex 开关。
- Claude Code 卡片：接入开关、未接入 / 接入中 / 已接入 / 需信任 / 被用户禁用 / 配置不可写 / 安装失败 / 卸载失败 / 自检失败。
- Codex 卡片：接入开关、未接入 / 接入中 / 已接入 / 需 trust / hooks disabled / 配置不可写 / 安装失败 / 卸载失败 / 自检失败。
- 按钮：预览变更、修复接入、运行自检；失败状态下显示重试。

交互细节：

- 首次打开某来源开关时，先进入安装预览，不直接写配置。
- 如果 hook 已安装且自检通过，打开开关可以只校验 manifest 并更新 `hookSource`。
- 关闭开关就是卸载 Agent Island 自己的 hook command；卸载成功后才显示为关闭。
- 关闭开关失败时恢复为打开，显示“卸载失败”，因为外部 agent 仍可能调用 Agent Island helper。
- 失败状态必须从持久化状态恢复。设置窗口重新打开后仍显示失败状态，不依赖上一次运行时内存。
- “重试”按钮根据失败动作调用对应操作：安装失败重试安装，卸载失败重试卸载，自检失败重试自检，修复失败重试修复。
- 卡片内必须明确说明“此开关只影响 Claude Code”或“此开关只影响 Codex”。
- 两个来源的状态和错误互不影响，一个来源配置不可写时不阻断另一个来源。

诊断窗口增加：

- 现有 hook 来源列表。
- Agent Island command 是否存在。
- helper 路径是否存在、是否可执行。
- spool 是否可写。
- 持久化失败状态、最近失败时间、失败动作和可重试动作。
- 最近 10 条归一化事件。
- 最近 ingest 错误。

## 11. 测试策略

单元测试：

- JSON/TOML 配置合并：保留用户字段、追加缺失 hook、重复安装幂等。
- 卸载：只删除 Agent Island command，不删除用户 command。
- hash 不一致：不整文件回滚，只精确删除。
- sanitizer：prompt、tool_input、tool_response、last_assistant_message 不进入输出。
- state machine：hook event 到 `TaskStatus` 的映射正确。

集成测试：

- 使用临时 HOME 模拟 `~/.codex/hooks.json` 和 `~/.claude/settings.json`。
- 安装、二次安装、卸载、用户手动改配置后卸载。
- helper 接收已启用来源的 Claude/Codex 样例 JSON 后写入最小 JSONL。
- app 不运行时 helper 能正常退出；app 启动后能消费积压事件。
- 关闭 Codex 开关后，`~/.codex` 配置中不再包含 Agent Island command，Codex 不再调用 helper；Claude Code 不受影响。
- 关闭 Claude Code 开关后，`~/.claude` 配置中不再包含 Agent Island command，Claude Code 不再调用 helper；Codex 不受影响。
- 卸载失败时开关保持打开，并展示失败原因。
- 安装、卸载、修复、自检失败后重启应用或重新打开设置窗口，失败状态仍然可见。
- 重试成功后清除失败状态；重试失败后更新失败时间和原因。

手动验收：

- 用户已有 hook 不变，安装后仍存在且顺序不变。
- Codex 未 trust 时任务不受影响，Agent Island 诊断页显示需 trust。
- Claude Code / Codex 执行 Bash、编辑文件、等待权限时，悬浮岛状态更新。
- 只打开 Claude Code 接入时，悬浮岛只展示 Claude Code hook 任务，不展示 Codex hook 任务。
- 只打开 Codex 接入时，悬浮岛只展示 Codex hook 任务，不展示 Claude Code hook 任务。
- 关闭某来源开关后，对应 agent 的后续任务中不再触发 Agent Island helper。
- helper 删除或不可执行时，Claude Code / Codex 不被阻断。
- 隐私模式开启后，UI 不显示完整 cwd / title。

## 12. 实施里程碑

### H0: 方案与样例

- 固化本方案。
- 收集 Claude Code / Codex 官方样例 payload。
- 定义 `HookEventEnvelope` 和 sanitizer 测试样例。

### H1: Spool 与 ingest

- 实现 JSONL spool。
- 实现 helper 逻辑与 `--self-test`。
- 实现 ingest service，把 JSONL 事件转成现有 `AgentEvent`。

### H2: 安装器 dry-run

- 实现 Claude / Codex 配置扫描。
- 实现 install preview，不写文件。
- 在诊断页展示 dry-run diff 和风险提示。

### H3: 安装 / 卸载

- 实现备份、原子写入、manifest。
- 实现幂等安装和精确卸载。
- 加配置合并和卸载单测。

### H4: 真实状态接入

- 启用 Claude Code hook。
- 启用 Codex hook。
- 用真实任务验证状态映射、等待用户、工具运行和完成状态。

### H5: 打磨

- 增加事件文件轮转和大小上限。
- 增加 “只保存项目名” 隐私模式。
- 增加诊断导出，但默认不包含敏感字段。

## 13. 风险与应对

| 风险 | 应对 |
| --- | --- |
| Hook 配置格式随版本变化 | 使用官方 parser/结构化解析；配置不可识别时不写入，只诊断 |
| Codex trust 流程导致 hook 不运行 | 不绕过 trust；在诊断页明确提示用户去 `/hooks` review/trust |
| Hook helper 变慢影响 agent | 本地 append-only，timeout 1 秒，异常直接 exit 0；后续用指标验证 P95 |
| 用户已有配置被破坏 | 写前备份、manifest、幂等 merge、精确卸载、hash 校验 |
| 采集到敏感内容 | 默认 denylist + allowlist 双层 sanitizer；测试覆盖敏感字段不落盘 |
| transcript 格式变化 | 不依赖 transcript 解析做主路径，只用 hook payload 的稳定字段 |
| 用户关闭 hooks | 尊重关闭状态，Agent Island 降级到 discovery，不强行启用 |
| 失败提示丢失导致错误积累 | 失败状态写入 config 和 manifest；设置与诊断窗口恢复展示，并提供重试 |

## 14. 官方参考

- Claude Code Hooks Reference: https://code.claude.com/docs/en/hooks
- Codex Hooks: https://developers.openai.com/codex/hooks
