# Hook Ingestion Summary

Claude Code / Codex 的真实状态采集采用官方 hook 作为主路径。Hook 只做旁路观测，不做拦截、不做注入、不改变 agent 的执行结果。

Hook 接入按来源独立受用户设置控制。只有用户在设置窗口打开对应来源的“接入状态”开关并完成安装后，Agent Island 才会把自己的 hook command 写入该来源配置；Claude Code 和 Codex 互不隐式启用。

## 数据流

```text
Claude Code / Codex hook
        │ stdin JSON
        ▼
agent-island-hook helper
        │ check source enabled setting
        │ append sanitized JSONL
        ▼
~/Library/Application Support/Agent Island/events/*.jsonl
        │ Rust watcher compacts to last 3 days when due
        ▼
Rust hook ingest service
        │ AgentEvent / AgentTask
        ▼
Vue / Pinia / Island UI
```

## 会话标题

- Helper 会保存最小化的 `sessionId`；Claude Code 事件还会保存本机 `transcriptPath` 指针，用于稳定定位对应会话文件，但不保存 transcript 内容。
- Rust ingest 先查对应来源的会话历史标题：Codex 读取 `~/.codex/session_index.jsonl` 的 `thread_name`；Claude Code 优先按 `transcriptPath` 读取会话文件里的 `aiTitle`、标题或 `summary` 记录，失败后再用 `sessionId` 查本机会话索引或匹配的项目会话文件。
- 找不到历史标题时，任务标题回退为当前工作目录名，保持现有降级逻辑。

## 完成态

- Claude Code 的 `Stop`、`SessionEnd`、`SubagentStop` 和 `TaskCompleted` 都归一化为 `completed`。
- `TaskCompleted` 需要写入 Claude Code hook 配置；老版本安装缺少该事件时，重新打开或修复 Claude Code 接入后才会收到这类完成事件。
- `Notification`、`CwdChanged` 这类不改变任务状态的事件不会覆盖之前的完成态；只有 `Notification.permission_prompt` 会归一化为 `waiting-user`。
- 未识别的未来 hook event 默认只作为 heartbeat 事件展示，不改变已有状态，避免新 schema 把完成态或等待态误拉回运行态。

## 超时态

- `PreToolUse` 后超过 10 分钟仍未收到新的状态改变事件时，任务归一化为 `stale`，表示工具运行状态可能已经失去可信度，但不判定为失败。
- Rust watcher 除了监听 `events/*.jsonl` 变化，也会每 5 分钟检查一次是否需要刷新任务快照；只有存在可能进入 `stale` 的 `tool-running` 任务，或 spool 实际触发压缩时，才会重建快照。
- Spool 压缩和快照重建解耦：启动时压缩一次；后续只有事件文件自上次压缩后发生变化，并且距离上次压缩至少 1 小时，或单个 spool 文件超过 5 MiB，才再次压缩。

## 暂停态

- Codex 桌面端手动暂停不会通过 hook helper 写入 `events/codex.jsonl`；对应信号在本机会话 JSONL 里表现为 `event_msg.payload.type = "turn_aborted"`，`reason = "interrupted"`。
- Rust ingest 只在已有 Codex hook 事件提供 `transcriptPath` 时，按行扫描本机会话 JSONL 的事件元数据行，提取 `timestamp` 和 `reason`，不保存 prompt、回复或 transcript 正文。
- 如果 `turn_aborted` 比最新 hook 状态更新，任务归一化为 `paused`；`paused` 表示用户已经主动中断，本轮不会继续占用压缩态或展开任务列表。后续新的 `UserPromptSubmit`、`PreToolUse`、`PostToolUse` 等 hook 状态会恢复正常运行态并重新显示。

## Hook 接收日志

- `events/*.jsonl` 是 append-only hook 事件 spool；helper 写入路径只追加当前事件，Rust watcher 在启动和满足压缩条件时按 `timestamp` 裁剪，只保留最近 3 天的可解析事件。
- Helper 会额外写入 `~/Library/Application Support/Agent Island/logs/hook-receipts-YYYY-MM-DD.jsonl`，用于排查 hook 是否到达、是否解析成功、是否写入状态事件。
- 接收日志保留 5 天，helper 每次运行时按文件日期清理过期日志。
- 日志只保存裁剪后的诊断字段：`source`、`event`、`sessionKey`、`cwd`、`toolName`、`notificationType`、`permissionMode`、解析结果、payload 大小、是否出现 prompt/tool input/tool response 字段，以及写入结果。
- 日志不保存 prompt 原文、assistant 回复正文、完整工具输入、完整工具输出、完整 shell command、完整 patch、transcript 内容或 `transcriptPath`。

## Helper 约束

- 从 stdin 读取 hook payload。
- 只保存最小状态字段。
- 不输出 stdout/stderr。
- 不返回 decision、permissionDecision、additionalContext、systemMessage 等会影响 agent 行为的字段。
- 所有异常路径返回 `exit 0`。
- Agent Island 未运行时也能安全退出；事件先写本地 JSONL spool。
- 正常关闭某来源开关后，该来源配置中不应再有 Agent Island hook command，Claude Code / Codex 不会再调用 helper。
- helper 仍在写 spool 前读取本地设置作为防御；如果当前 `source` 未启用接入，直接静默退出且不落盘。
- Rust ingest service 再次按设置过滤来源，防止旧版本或卸载失败残留事件在关闭接入后继续进入 UI。

## 后续 Windows 支持 TODO

- 把文档和代码里的 macOS app support 示例路径抽象为平台路径；Windows 侧预期落到 `%APPDATA%\Agent Island\events\*.jsonl` 和对应 `logs` 目录。
- 替换 Rust 侧 `libc::flock` 为跨平台文件锁封装，Windows 实现需要验证 `LockFileEx` 或等价库在 append、读取和压缩替换时的语义。
- 替换 helper 里的 POSIX 假设：Windows 不能依赖 `/bin/sh` 或 Python `fcntl.flock`，需要使用 `.exe` sidecar 或 Windows 专用锁实现。
- 复核 `events/*.jsonl` 压缩策略在 Windows 上的原子替换行为，避免打开中的文件导致 `rename` / `replace` 失败或阻塞 hook 写入。
- 更新 hook install/uninstall 的命令拼接、路径转义和 `.exe` 后缀处理，覆盖带空格路径和反斜杠路径。
- 明确 Windows 通知权限、通知音和无声模式的降级行为；macOS 内置 sound 名称不能直接复用。
- 增加 Windows CI 或本机验证矩阵：helper 真实执行、append lock、3 天 spool 压缩、hook 安装 dry-run/卸载、Tauri floating window 和通知 smoke test。

## 默认禁止采集

- prompt 原文。
- assistant 回复正文。
- 完整工具输入。
- 完整工具输出。
- 完整 shell command。
- 完整 patch 或文件内容。
- transcript 文件内容；标题解析只允许本机即时读取会话文件里的标题/summary 字段，不落盘保存正文。

## 安装原则

- 不静默安装；用户必须在设置窗口确认。
- 先 dry-run，再备份，再原子写入。
- 不修改用户已有 hook，不修改 hook 开关，不绕过 Codex trust。
- 卸载只删除 Agent Island 自己的 command。
- 每个来源独立启用和卸载。关闭某来源开关必须精确删除 Agent Island 自己的 hook command，让对应 AI agent 后续不再调用 Agent Island helper。

详细方案见 [hook-integration-plan.md](hook-integration-plan.md)。
