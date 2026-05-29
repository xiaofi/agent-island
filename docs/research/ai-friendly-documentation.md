# AI-Friendly Documentation Research

本项目采用“AI 入口 + 人类入口 + 分层短文档 + 长文档详细来源”的文档结构。目标是让 AI agent 在有限上下文内快速找到正确资料，同时让人类维护者仍能读到完整设计背景。

## 调研来源

- Diataxis：把文档按 tutorial、how-to、reference、explanation 四类分开，避免一个文档同时承担学习、操作、查询和解释。来源：https://diataxis.fr/
- llms.txt：为 LLM 提供一个简单 Markdown 入口，指向项目关键文档和可选详细资料。来源：https://llmstxt.org/
- OpenAI Codex AGENTS.md：用仓库内 `AGENTS.md` 给 coding agent 提供项目规则、命令和约束。来源：https://developers.openai.com/codex/guides/agents-md
- Claude Code memory：使用项目内记忆文件保存团队约定、架构说明和常用命令，减少重复解释。来源：https://docs.anthropic.com/en/docs/claude-code/memory
- Google developer documentation style guide：强调清晰、任务导向、一致术语和可扫描结构。来源：https://developers.google.com/style

## 采用原则

### 1. 多入口

- `README.md`：人类读者入口。
- `AGENTS.md`：AI coding agent 指令入口。
- `llms.txt`：LLM 文档索引入口。
- `docs/README.md`：完整文档地图。

### 2. 按任务路由

AI 不应该每次读完整 spec 和完整技术方案。`docs/ai/context-map.md` 按任务列出最小必读文档和按需文件。

### 3. 短文档放稳定结论

`docs/product/`、`docs/architecture/`、`docs/development/` 和 `docs/operations/` 只放稳定结论、当前约束和入口链接。

### 4. 长文档保留完整推理

`docs/product/spec.md`、`docs/architecture/technical-plan.md`、`docs/architecture/hook-integration-plan.md` 保留详细方案，不拆散为大量重复内容，降低同步成本。

### 5. 决策单独归档

架构级文档组织变化写入 `docs/decisions/`，后续变更可以追加 ADR，而不是覆盖历史背景。

## 本项目文档层级

```text
README.md
AGENTS.md
llms.txt
docs/
  README.md
  ai/
  product/
  architecture/
  development/
  operations/
  research/
  decisions/
docs/product/spec.md
docs/architecture/technical-plan.md
docs/architecture/hook-integration-plan.md
```

## 维护检查清单

- 新增功能是否能从 `docs/ai/context-map.md` 找到上下文？
- 新增约束是否同步到 `AGENTS.md`？
- 新增 AI 可读入口是否同步到 `llms.txt`？
- 架构取舍是否有 ADR？
- 隐私或 hook 行为变化是否同步到 `docs/operations/privacy-and-permissions.md`？
