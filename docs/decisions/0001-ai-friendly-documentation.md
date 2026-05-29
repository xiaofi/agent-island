# ADR 0001: AI-Friendly Documentation Architecture

## Status

Accepted

## Context

Agent Island 的文档已经包含产品 spec、技术方案和 hook 接入方案。这些文档信息完整，但篇幅较长，AI agent 每次执行任务时不适合全部读取。项目还需要同时支持人类维护者和 AI coding agent。

## Decision

采用分层、多文件、任务导向的文档结构：

- 根目录保留 `README.md`、`AGENTS.md`、`llms.txt` 三个入口。
- `docs/README.md` 作为完整文档索引。
- `docs/ai/context-map.md` 作为 AI agent 的任务路由表。
- `docs/product/`、`docs/architecture/`、`docs/development/`、`docs/operations/` 放短而稳定的主题文档。
- `docs/product/spec.md`、`docs/architecture/technical-plan.md`、`docs/architecture/hook-integration-plan.md` 保留为详细设计来源。
- 架构级取舍写入 `docs/decisions/`。
- 外部调研写入 `docs/research/`。

## Consequences

优点：

- AI agent 可以按任务读取最小必要上下文。
- 长设计文档不用被拆散，降低同步成本。
- 人类读者有清晰入口和阅读路径。
- 后续架构变化可以通过 ADR 追踪。

代价：

- 新增或调整重要入口时，需要同步更新 `llms.txt` 和 `docs/ai/context-map.md`。
- 短文档和长文档之间存在轻微重复，需要控制短文档只写稳定结论。

## References

- Diataxis: https://diataxis.fr/
- llms.txt: https://llmstxt.org/
- OpenAI Codex AGENTS.md: https://developers.openai.com/codex/guides/agents-md
- Claude Code memory: https://docs.anthropic.com/en/docs/claude-code/memory
