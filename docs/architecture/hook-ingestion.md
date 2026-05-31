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
        │ sanitized JSONL
        ▼
~/Library/Application Support/Agent Island/events/*.jsonl
        │
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

## Hook 接收日志

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
