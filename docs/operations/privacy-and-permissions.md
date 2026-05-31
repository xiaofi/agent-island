# Privacy and Permissions

Agent Island 的默认隐私模型是本地、最小化、可审计。

## 默认行为

- 不上传任何会话信息。
- 不展示完整 prompt 或回复正文。
- 不读取 transcript 内容作为主路径。
- 不保存完整工具输入、工具输出、shell command、patch 或文件内容。
- Hook 接收日志只保存裁剪后的诊断字段并保留 5 天，不保存 prompt、回复正文、完整工具输入/输出、完整 shell command、完整 patch、transcript 内容或 `transcriptPath`。
- UI 可开启隐私模式，隐藏项目路径和任务标题。
- Claude Code 和 Codex 的真实 hook 接入默认关闭，必须由用户在设置窗口分别打开。未发现本机安装对应工具时，不显示开关，只显示未发现安装。

## Hook 权限

Hook 安装会修改 Claude Code / Codex 的用户级或项目级配置，因此必须满足：

- 用户显式确认。
- 按来源单独确认；打开 Claude Code 接入不能隐式打开 Codex，打开 Codex 接入也不能隐式打开 Claude Code。
- 安装前展示 dry-run diff。
- 写入前创建备份。
- 使用结构化 JSON/TOML parser 合并配置。
- 原子写入。
- 写入 manifest，记录 Agent Island 自己添加的 command。
- 卸载只删除 Agent Island 自己的 command。

关闭某来源开关必须卸载 Agent Island 自己的 hook command，不能只做 UI 隐藏或本地暂停。卸载成功后，对应 Claude Code / Codex 后续不应再调用 Agent Island helper。卸载失败时开关保持打开并展示失败原因。

安装、卸载、修复或自检失败后，失败状态必须持久化。用户关闭再打开 Agent Island 时，设置和诊断窗口仍要显示失败来源、失败动作、失败时间、简短原因和重试入口。一次性 toast 不能替代持久化错误状态。

## 不允许的行为

- 静默安装 hook。
- 未经用户打开对应来源接入开关就安装 Agent Island hook、接收、落盘或展示该来源真实 hook 状态。
- 修改用户已有 hook handler。
- 修改用户 hook 开关。
- 绕过 Codex trust review。
- 用 managed hooks 强制采集个人本地状态。
- 在 hook helper 中输出 stdout/stderr。
- 在 hook helper 中返回任何会影响 agent 执行的 decision/context 字段。

## 降级策略

如果 hook 不可用：

- 不阻断 Claude Code / Codex。
- Agent Island 降级到配置文件 discovery 和候选路径诊断。
- 诊断窗口显示 hook 未安装、未 trust、配置不可写或被用户禁用等状态。
- 如果关闭某来源开关时卸载失败，诊断窗口显示失败原因和手动删除指引；不能假装该来源已经关闭。
- 如果失败状态存在，诊断窗口持续显示并提供重试；重试成功后再清除。

Hook 细节见 [../architecture/hook-ingestion.md](../architecture/hook-ingestion.md) 和 [../architecture/hook-integration-plan.md](../architecture/hook-integration-plan.md)。
