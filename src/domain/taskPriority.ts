import type { AgentTask, TaskStatus } from "@/domain/taskTypes";

export const statusWeight: Record<TaskStatus, number> = {
  "waiting-user": 100,
  failed: 95,
  completed: 90,
  stale: 85,
  "tool-running": 70,
  thinking: 60,
  running: 50,
  paused: 10,
  discovering: 1,
};

const attentionStatuses = new Set<TaskStatus>(["waiting-user", "completed", "failed", "stale"]);

const activeStatuses = new Set<TaskStatus>([
  "discovering",
  "running",
  "thinking",
  "tool-running",
  "waiting-user",
  "completed",
  "failed",
  "paused",
  "stale",
]);

export function compareTasksByPriority(a: AgentTask, b: AgentTask) {
  const weightDiff = statusWeight[b.status] - statusWeight[a.status];
  if (weightDiff !== 0) {
    return weightDiff;
  }

  return new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime();
}

export function sortTasksByPriority(tasks: AgentTask[]) {
  return [...tasks].sort(compareTasksByPriority);
}

export function isActiveTask(task: AgentTask) {
  return activeStatuses.has(task.status);
}

export function needsAttention(task: AgentTask) {
  return attentionStatuses.has(task.status);
}

export function pickPrimaryTask(tasks: AgentTask[]) {
  return sortTasksByPriority(tasks)[0];
}
