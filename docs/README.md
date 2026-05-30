# Agent Island Docs

本文档目录采用 AI 友好的分层结构：入口短、主题分离、长文档保留为详细来源，避免一个超长文件承载所有上下文。

## 推荐阅读路径

### 快速理解项目

1. [../README.md](../README.md)
2. [product/overview.md](product/overview.md)
3. [architecture/overview.md](architecture/overview.md)

### 开始开发

1. [development/getting-started.md](development/getting-started.md)
2. [../AGENTS.md](../AGENTS.md)
3. [ai/context-map.md](ai/context-map.md)

### 做 hook 相关功能

1. [architecture/hook-ingestion.md](architecture/hook-ingestion.md)
2. [operations/privacy-and-permissions.md](operations/privacy-and-permissions.md)
3. [architecture/hook-integration-plan.md](architecture/hook-integration-plan.md)
4. [decisions/0002-per-source-hook-reception.md](decisions/0002-per-source-hook-reception.md)

### 做 macOS 悬浮窗和全屏展示

1. [architecture/overview.md](architecture/overview.md)
2. [decisions/0003-macos-fullscreen-overlay.md](decisions/0003-macos-fullscreen-overlay.md)

## 文档分层

| 层级 | 文件 | 用途 |
| --- | --- | --- |
| 入口 | `README.md`、`AGENTS.md`、`llms.txt` | 给人类和 AI 快速定位上下文 |
| 地图 | `docs/README.md`、`docs/ai/context-map.md` | 按任务路由到最小必要文档 |
| 概览 | `docs/product/`、`docs/architecture/` | 稳定结论和架构摘要 |
| 操作 | `docs/development/`、`docs/operations/` | 开发、测试、权限、隐私、发布等可执行流程 |
| 详细方案 | `docs/product/spec.md`、`docs/architecture/technical-plan.md`、`docs/architecture/hook-integration-plan.md` | 长文档，保留完整推理和方案 |
| 决策记录 | `docs/decisions/` | 架构级取舍和变更背景 |
| 调研 | `docs/research/` | 外部资料、引用和采用原则 |

## 维护原则

- 每个文档只回答一个问题。
- 概览文件保持短而稳定，复杂细节链接到长文档。
- AI 入口文件只放任务路由、约束和关键命令。
- 新架构决策写 ADR，不把背景散落在实现文件里。
- 涉及隐私、hook、用户配置修改的内容必须同时更新操作文档和 AI 上下文地图。
