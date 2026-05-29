# Hook Ingestion Summary

Claude Code / Codex 的真实状态采集采用官方 hook 作为主路径。Hook 只做旁路观测，不做拦截、不做注入、不改变 agent 的执行结果。

## 数据流

```text
Claude Code / Codex hook
        │ stdin JSON
        ▼
agent-island-hook helper
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

## Helper 约束

- 从 stdin 读取 hook payload。
- 只保存最小状态字段。
- 不输出 stdout/stderr。
- 不返回 decision、permissionDecision、additionalContext、systemMessage 等会影响 agent 行为的字段。
- 所有异常路径返回 `exit 0`。
- Agent Island 未运行时也能安全退出；事件先写本地 JSONL spool。

## 默认禁止采集

- prompt 原文。
- assistant 回复正文。
- 完整工具输入。
- 完整工具输出。
- 完整 shell command。
- 完整 patch 或文件内容。
- transcript 文件内容。

## 安装原则

- 不静默安装；用户必须在设置窗口确认。
- 先 dry-run，再备份，再原子写入。
- 不修改用户已有 hook，不修改 hook 开关，不绕过 Codex trust。
- 卸载只删除 Agent Island 自己的 command。

详细方案见 [hook-integration-plan.md](hook-integration-plan.md)。
