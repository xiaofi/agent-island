import type { AdapterDiagnostic, AgentEvent, AgentTask, AppSettings } from "@/domain/taskTypes";

const now = Date.now();

function iso(offsetSeconds: number) {
  return new Date(now - offsetSeconds * 1000).toISOString();
}

function event(taskId: string, type: AgentEvent["type"], summary: string, offsetSeconds: number): AgentEvent {
  return {
    id: `${taskId}-${type}-${offsetSeconds}`,
    taskId,
    type,
    timestamp: iso(offsetSeconds),
    summary,
  };
}

export const mockTasks: AgentTask[] = [
  {
    id: "codex-agent-island",
    source: "codex",
    title: "实现 Agent Island MVP",
    cwd: "/Users/spf/project/agent-island",
    status: "tool-running",
    startedAt: iso(420),
    updatedAt: iso(12),
    lastAction: "写入 Vue 组件",
    windowHint: {
      appName: "Codex",
      windowTitle: "agent-island",
    },
    events: [
      event("codex-agent-island", "session-started", "启动 Codex 会话", 420),
      event("codex-agent-island", "user-message", "开始实现 MVP 版本", 380),
      event("codex-agent-island", "tool-started", "创建 Vue + Tauri 项目文件", 24),
      event("codex-agent-island", "tool-finished", "完成技术方案更新", 16),
      event("codex-agent-island", "tool-started", "写入组件与 store", 12),
    ],
  },
  {
    id: "claude-api-server",
    source: "claude-code",
    title: "检查 API server 测试失败",
    cwd: "/Users/spf/project/api-server",
    status: "waiting-user",
    startedAt: iso(980),
    updatedAt: iso(84),
    lastAction: "等待命令确认",
    waitingReason: "需要批准运行数据库迁移测试",
    windowHint: {
      appName: "Terminal",
      windowTitle: "claude api-server",
    },
    events: [
      event("claude-api-server", "session-started", "启动 Claude Code 会话", 980),
      event("claude-api-server", "tool-started", "读取测试日志", 640),
      event("claude-api-server", "tool-finished", "定位失败用例", 420),
      event("claude-api-server", "waiting-for-user", "等待用户确认命令", 84),
    ],
  },
  {
    id: "codex-docs-pass",
    source: "codex",
    title: "整理 adapter discovery 文档",
    cwd: "/Users/spf/project/agent-island",
    status: "thinking",
    startedAt: iso(320),
    updatedAt: iso(30),
    lastAction: "归纳 discovery 输出字段",
    events: [
      event("codex-docs-pass", "session-started", "启动文档任务", 320),
      event("codex-docs-pass", "assistant-thinking", "整理 adapter 诊断结构", 30),
    ],
  },
  {
    id: "manual-release-check",
    source: "manual",
    title: "打包前检查清单",
    cwd: "/Users/spf/project/agent-island",
    status: "completed",
    startedAt: iso(2200),
    updatedAt: iso(210),
    lastAction: "mock 数据验证完成",
    events: [
      event("manual-release-check", "session-started", "开始手动检查", 2200),
      event("manual-release-check", "session-completed", "检查完成", 210),
    ],
  },
];

export const mockSettings: AppSettings = {
  appearance: {
    islandOpacity: 0.92,
  },
  notifications: {
    enabled: false,
    sound: "default",
  },
  autoAcknowledge: {
    enabled: false,
    delaySeconds: 900,
  },
  quietMode: false,
  showInDock: false,
  enabledAdapters: ["manual", "codex", "claude-code"],
  hookSource: {
    codex: false,
    claudeCode: false,
    lastErrors: {},
  },
  islandWindow: {},
};

export const mockDiagnostics: AdapterDiagnostic[] = [
  {
    source: "codex",
    status: "partial",
    summary: "浏览器预览模式下使用 mock 数据；Tauri 运行时会检查 Codex 配置文件。",
    processes: [],
    candidatePaths: [
      {
        path: "~/.codex/hooks.json",
        exists: false,
        readable: false,
        reason: "浏览器预览无法读取本地路径",
      },
    ],
    parsedSessions: 1,
    updatedAt: new Date().toISOString(),
  },
  {
    source: "claude-code",
    status: "partial",
    summary: "浏览器预览模式下使用 mock 数据；Tauri 运行时会检查 Claude Code 配置文件。",
    processes: [],
    candidatePaths: [
      {
        path: "~/.claude/settings.json",
        exists: false,
        readable: false,
        reason: "浏览器预览无法读取本地路径",
      },
    ],
    parsedSessions: 1,
    updatedAt: new Date().toISOString(),
  },
];

export function cloneMockTasks() {
  return structuredClone(mockTasks);
}

export function cloneMockDiagnostics() {
  return structuredClone(mockDiagnostics);
}

export function cloneMockSettings() {
  return structuredClone(mockSettings);
}
