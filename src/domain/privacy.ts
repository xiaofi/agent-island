import type { AgentTask, PrivacySettings } from "@/domain/taskTypes";

export function projectNameFromPath(path?: string) {
  if (!path) {
    return "";
  }

  const normalized = path.replace(/\/+$/, "");
  const segments = normalized.split("/");
  return segments[segments.length - 1] || normalized;
}

export function maskTask(task: AgentTask, privacy: PrivacySettings, options: { compact?: boolean } = {}): AgentTask {
  const compactOnly = privacy.compactOnly && options.compact;

  return {
    ...task,
    title: compactOnly ? "" : privacy.hideTaskTitle ? `${sourceLabel(task.source)} task` : task.title,
    cwd: privacy.hideProjectPath || compactOnly ? projectNameFromPath(task.cwd) : task.cwd,
  };
}

export function sourceLabel(source: AgentTask["source"]) {
  switch (source) {
    case "codex":
      return "Codex";
    case "claude-code":
      return "Claude Code";
    case "manual":
      return "Manual";
  }
}

export function statusLabel(status: AgentTask["status"]) {
  switch (status) {
    case "discovering":
      return "发现中";
    case "running":
      return "运行中";
    case "thinking":
      return "思考中";
    case "tool-running":
      return "执行工具";
    case "waiting-user":
      return "等待处理";
    case "completed":
      return "已完成";
    case "failed":
      return "出错";
    case "paused":
      return "已暂停";
    case "stale":
      return "可能停滞";
  }
}
