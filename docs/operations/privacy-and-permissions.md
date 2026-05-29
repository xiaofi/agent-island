# Privacy and Permissions

Agent Island 的默认隐私模型是本地、最小化、可审计。

## 默认行为

- 不上传任何会话信息。
- 不展示完整 prompt 或回复正文。
- 不读取 transcript 内容作为主路径。
- 不保存完整工具输入、工具输出、shell command、patch 或文件内容。
- UI 可开启隐私模式，隐藏项目路径和任务标题。

## Hook 权限

Hook 安装会修改 Claude Code / Codex 的用户级或项目级配置，因此必须满足：

- 用户显式确认。
- 安装前展示 dry-run diff。
- 写入前创建备份。
- 使用结构化 JSON/TOML parser 合并配置。
- 原子写入。
- 写入 manifest，记录 Agent Island 自己添加的 command。
- 卸载只删除 Agent Island 自己的 command。

## 不允许的行为

- 静默安装 hook。
- 修改用户已有 hook handler。
- 修改用户 hook 开关。
- 绕过 Codex trust review。
- 用 managed hooks 强制采集个人本地状态。
- 在 hook helper 中输出 stdout/stderr。
- 在 hook helper 中返回任何会影响 agent 执行的 decision/context 字段。

## 降级策略

如果 hook 不可用：

- 不阻断 Claude Code / Codex。
- Agent Island 降级到 process discovery 和候选路径诊断。
- 诊断窗口显示 hook 未安装、未 trust、配置不可写或被用户禁用等状态。

Hook 细节见 [../architecture/hook-ingestion.md](../architecture/hook-ingestion.md) 和 [../architecture/hook-integration-plan.md](../architecture/hook-integration-plan.md)。
