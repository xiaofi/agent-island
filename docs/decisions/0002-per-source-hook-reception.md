# 0002 Per-Source Hook Reception

## 状态

Accepted

## 背景

Agent Island 需要接入 Claude Code 和 Codex 的官方 hook 来展示真实运行状态。两个工具的配置、信任机制和用户期望不同，不能因为用户启用了一个来源，就隐式接收另一个来源的状态。

用户期望关闭某个来源时，对 Claude Code / Codex 没有任何持续运行时影响。因此开关不能只隐藏 UI 或只让 helper 丢弃事件；关闭必须删除 Agent Island 自己的 hook command。

## 决策

Claude Code 和 Codex 的 hook 接入按来源独立控制：

- 默认两个来源都不接入真实 hook。
- 设置窗口先做 discovery；只有发现本机安装了对应工具时，才显示该来源开关。
- 未发现 Claude Code 或 Codex 安装时，只显示“未发现软件安装”，不显示开关。
- 设置窗口分别提供“接入 Claude Code 状态”和“接入 Codex 状态”开关。
- 打开开关表示安装 Agent Island hook 并接收该来源状态。
- 关闭开关表示卸载 Agent Island 自己的 hook command，成功后该来源不再调用 Agent Island helper。
- helper 和 ingest 仍按当前设置过滤来源，作为卸载失败、旧版本残留或积压事件的防御。
- 卸载失败时，开关必须保持打开并展示失败原因，不能显示为已关闭。
- 安装、卸载、修复或自检失败必须持久化；设置窗口重新打开后仍显示失败状态，并提供对应的重试按钮。

## 后果

好处：

- 用户授权粒度清晰，Claude Code 和 Codex 互不牵连。
- 关闭某来源后，对对应 AI agent 没有持续 hook 调用影响。
- 未安装的工具不会显示无效开关，用户不需要理解不可用状态。
- 失败不会因为窗口关闭或应用重启而被遗忘，减少配置错误积累。

代价：

- 开关关闭会改写配置，必须做好备份、原子写入、失败回滚和错误提示。
- helper 和 ingest 仍需要读取设置并做来源过滤，作为防御层。
- 需要维护持久化失败状态和重试动作映射。
