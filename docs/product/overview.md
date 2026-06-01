# Product Overview

Agent Island 是一个本地桌面悬浮状态层，用于让开发者快速知道 Claude Code、Codex 等 AI 编程代理当前在做什么。

## 目标

- 常驻桌面的轻量悬浮岛。
- 展示当前活跃 agent 任务数量、最重要状态和最近动作。
- 明确区分运行中、思考中、执行工具、等待用户、完成、失败、暂停、过期。
- 支持展开任务列表和单任务详情。
- 支持从任务跳回相关应用、终端窗口或项目目录。
- 首版 macOS 优先，本地运行，不依赖云端服务。

## 非目标

- 不替代 Claude Code、Codex、终端或 IDE。
- 不接管 agent 的执行流程。
- 不默认读取或展示完整对话内容。
- 不做跨机器同步、团队协作或复杂项目管理。
- MVP 不修改、取消或注入 agent 任务。

## 核心界面

- 压缩态：小型动态岛，优先显示需要关注的任务；没有或只有一条关注任务时保持单行悬浮岛，超过一条关注任务时切换为卡片式多行。
- 展开态：任务列表根据悬浮岛位置向下或向上展开，显示来源、会话标题、状态、最近动作、运行时长和工作目录；列表字号与压缩态保持同一信息密度。
- 详情态：展示单任务元信息、最近事件、等待原因或错误摘要。
- 完整窗口：设置、诊断和后续复杂功能打开独立桌面窗口。

压缩态中，`waiting-user`、`failed`、`completed`、`paused`、`stale` 这类需要关注的任务逐条展示；`discovering`、`running`、`thinking`、`tool-running` 合并为“N 个任务进行中”。底部/单行右侧入口在收起时显示“显示全部任务”，展开后显示“收起列表”。

任务标题优先来自对应 agent 的本机会话历史标题；Codex 通过 session id 查本地索引，Claude Code 可通过本机 transcript 指针读取 `aiTitle`、标题或 `summary` 字段。如果无法查到标题，则回退为当前工作目录名。

## 状态优先级

1. `waiting-user`
2. `failed`
3. `tool-running`
4. `thinking` / `running`
5. `completed`
6. `paused` / `stale`

详细字段和交互见 [spec.md](spec.md)。
