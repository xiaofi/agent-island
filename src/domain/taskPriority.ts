import type { AgentTask, TaskStatus } from "@/domain/taskTypes";

export const statusWeight: Record<TaskStatus, number> = {
  "waiting-user": 100,
  failed: 90,
  "tool-running": 70,
  thinking: 60,
  running: 50,
  completed: 20,
  paused: 10,
  stale: 5,
  discovering: 1,
};

const activeStatuses = new Set<TaskStatus>([
  "discovering",
  "running",
  "thinking",
  "tool-running",
  "waiting-user",
  "failed",
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

export function pickPrimaryTask(tasks: AgentTask[]) {
  return sortTasksByPriority(tasks)[0];
}
