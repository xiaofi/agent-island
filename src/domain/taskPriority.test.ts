import { describe, expect, it } from "vitest";
import { needsAttention, pickPrimaryTask, sortTasksByPriority } from "@/domain/taskPriority";
import type { AgentTask } from "@/domain/taskTypes";

const baseTask: AgentTask = {
  id: "base",
  source: "manual",
  title: "Base task",
  status: "running",
  updatedAt: "2026-05-29T10:00:00.000Z",
  events: [],
};

describe("taskPriority", () => {
  it("promotes waiting-user tasks above running tasks", () => {
    const tasks: AgentTask[] = [
      { ...baseTask, id: "running", status: "running" },
      { ...baseTask, id: "waiting", status: "waiting-user", updatedAt: "2026-05-29T09:00:00.000Z" },
      { ...baseTask, id: "tool", status: "tool-running", updatedAt: "2026-05-29T11:00:00.000Z" },
    ];

    expect(sortTasksByPriority(tasks).map((task) => task.id)).toEqual(["waiting", "tool", "running"]);
    expect(pickPrimaryTask(tasks)?.id).toBe("waiting");
  });

  it("promotes finished and interrupted tasks above active work", () => {
    const tasks: AgentTask[] = [
      { ...baseTask, id: "tool", status: "tool-running", updatedAt: "2026-05-29T12:00:00.000Z" },
      { ...baseTask, id: "completed", status: "completed", updatedAt: "2026-05-29T09:00:00.000Z" },
      { ...baseTask, id: "failed", status: "failed", updatedAt: "2026-05-29T08:00:00.000Z" },
    ];

    expect(sortTasksByPriority(tasks).map((task) => task.id)).toEqual(["failed", "completed", "tool"]);
    expect(pickPrimaryTask(tasks)?.id).toBe("failed");
  });

  it("keeps user-paused tasks out of attention priority", () => {
    const tasks: AgentTask[] = [
      { ...baseTask, id: "paused", status: "paused", updatedAt: "2026-05-29T12:00:00.000Z" },
      { ...baseTask, id: "running", status: "running", updatedAt: "2026-05-29T10:00:00.000Z" },
    ];

    expect(needsAttention(tasks[0])).toBe(false);
    expect(sortTasksByPriority(tasks).map((task) => task.id)).toEqual(["running", "paused"]);
  });

  it("uses updatedAt when statuses have the same priority", () => {
    const tasks: AgentTask[] = [
      { ...baseTask, id: "older", updatedAt: "2026-05-29T08:00:00.000Z" },
      { ...baseTask, id: "newer", updatedAt: "2026-05-29T12:00:00.000Z" },
    ];

    expect(sortTasksByPriority(tasks).map((task) => task.id)).toEqual(["newer", "older"]);
  });
});
