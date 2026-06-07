// @vitest-environment happy-dom

import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it } from "vitest";
import type { AgentEvent, AgentTask, TaskStatus } from "@/domain/taskTypes";
import { useTaskStore } from "@/stores/taskStore";

function completedEvent(id: string, timestamp: string): AgentEvent {
  return {
    id,
    taskId: "codex-task",
    type: "session-completed",
    timestamp,
    summary: "完成任务",
  };
}

function completedTask(eventId: string, updatedAt: string): AgentTask {
  return {
    id: "codex-task",
    source: "codex",
    title: "cmux",
    status: "completed",
    updatedAt,
    events: [completedEvent(eventId, updatedAt)],
  };
}

function task(status: TaskStatus): AgentTask {
  return {
    id: "codex-task",
    source: "codex",
    title: "cmux",
    status,
    updatedAt: "2026-05-30T01:00:00.000Z",
    events: [],
  };
}

describe("taskStore completed acknowledgements", () => {
  beforeEach(() => {
    window.localStorage.clear();
    setActivePinia(createPinia());
  });

  it("acknowledges one completion without suppressing a later completion for the same task id", () => {
    const store = useTaskStore();

    store.upsertTask(completedTask("complete-1", "2026-05-30T01:00:00.000Z"));
    expect(store.completedAlertTasks).toHaveLength(1);

    store.acknowledgeCompletedTask("codex-task");
    expect(store.completedAlertTasks).toHaveLength(0);

    store.upsertTask(completedTask("complete-2", "2026-05-30T02:00:00.000Z"));

    expect(store.completedAlertTasks).toHaveLength(1);
    expect(store.completedAlertTasks[0].events[0].id).toBe("complete-2");
  });

  it("uses the completion timestamp even when the completion event id is stable", () => {
    const store = useTaskStore();

    store.upsertTask(completedTask("stable-complete-id", "2026-05-30T01:00:00.000Z"));
    store.acknowledgeCompletedTask("codex-task");

    store.upsertTask(completedTask("stable-complete-id", "2026-05-30T02:00:00.000Z"));

    expect(store.completedAlertTasks).toHaveLength(1);
    expect(store.completedAlertTasks[0].updatedAt).toBe("2026-05-30T02:00:00.000Z");
  });
});

describe("taskStore archived statuses", () => {
  beforeEach(() => {
    window.localStorage.clear();
    setActivePinia(createPinia());
  });

  it("keeps paused tasks out of the visible task list", () => {
    const store = useTaskStore();

    store.upsertTask(task("paused"));

    expect(store.tasks).toHaveLength(1);
    expect(store.displayTasks).toHaveLength(0);
    expect(store.visibleTasks).toHaveLength(0);
  });

  it("shows the same task again when a later event resumes it", () => {
    const store = useTaskStore();

    store.upsertTask(task("paused"));
    store.upsertTask({ ...task("thinking"), updatedAt: "2026-05-30T01:02:00.000Z" });

    expect(store.displayTasks).toHaveLength(1);
    expect(store.displayTasks[0].status).toBe("thinking");
  });
});
