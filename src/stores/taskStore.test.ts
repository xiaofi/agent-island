// @vitest-environment happy-dom

import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { AgentEvent, AgentTask, TaskStatus } from "@/domain/taskTypes";
import { useTaskStore } from "@/stores/taskStore";
import { usePreferencesStore } from "@/stores/preferencesStore";

function event(id: string, type: AgentEvent["type"], timestamp: string, summary: string): AgentEvent {
  return {
    id,
    taskId: "codex-task",
    type,
    timestamp,
    summary,
  };
}

function completedEvent(id: string, timestamp: string): AgentEvent {
  return event(id, "session-completed", timestamp, "完成任务");
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

function task(status: TaskStatus, events: AgentEvent[] = []): AgentTask {
  return {
    id: "codex-task",
    source: "codex",
    title: "cmux",
    status,
    updatedAt: "2026-05-30T01:00:00.000Z",
    events,
  };
}

describe("taskStore completed acknowledgements", () => {
  beforeEach(() => {
    window.localStorage.clear();
    setActivePinia(createPinia());
    vi.clearAllMocks();
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

  it("automatically acknowledges completed tasks after the configured delay", async () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-05-30T01:00:00.000Z"));

    try {
      const preferencesStore = usePreferencesStore();
      preferencesStore.settings.autoAcknowledge.enabled = true;
      preferencesStore.settings.autoAcknowledge.delaySeconds = 300;
      const store = useTaskStore();

      store.upsertTask(completedTask("complete-1", "2026-05-30T01:00:00.000Z"));
      expect(store.completedAlertTasks).toHaveLength(1);

      await vi.advanceTimersByTimeAsync(299_000);
      expect(store.completedAlertTasks).toHaveLength(1);

      await vi.advanceTimersByTimeAsync(1_000);
      expect(store.completedAlertTasks).toHaveLength(0);
    } finally {
      vi.useRealTimers();
    }
  });

  it("does not auto acknowledge a manually acknowledged task again", async () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-05-30T01:00:00.000Z"));

    try {
      const preferencesStore = usePreferencesStore();
      preferencesStore.settings.autoAcknowledge.enabled = true;
      preferencesStore.settings.autoAcknowledge.delaySeconds = 300;
      const store = useTaskStore();

      store.upsertTask(completedTask("complete-1", "2026-05-30T01:00:00.000Z"));
      store.acknowledgeCompletedTask("codex-task");

      await vi.advanceTimersByTimeAsync(300_000);

      expect(store.completedAlertTasks).toHaveLength(0);
      expect(store.tasks).toHaveLength(0);
    } finally {
      vi.useRealTimers();
    }
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

  it("manually clears a stuck task until a later event updates it", () => {
    const store = useTaskStore();

    store.upsertTask(task("tool-running"));
    store.completeAndAcknowledgeTask("codex-task");

    expect(store.tasks).toHaveLength(0);

    store.upsertTask(task("tool-running"));
    expect(store.tasks).toHaveLength(0);

    store.upsertTask({ ...task("thinking"), updatedAt: "2026-05-30T01:02:00.000Z" });
    expect(store.tasks).toHaveLength(1);
    expect(store.displayTasks[0].status).toBe("thinking");
  });
});

describe("taskStore live events", () => {
  beforeEach(() => {
    window.localStorage.clear();
    setActivePinia(createPinia());
  });

  it("keeps canonical task fields when a supplemental event arrives", () => {
    const store = useTaskStore();
    store.upsertTask({
      ...task("completed", [completedEvent("complete-1", "2026-05-30T01:00:00.000Z")]),
      lastAction: "会话本轮完成",
    });

    store.addEvent(event("notification-1", "heartbeat", "2026-05-30T01:01:00.000Z", "收到通知"));

    expect(store.tasks[0].events[0].id).toBe("notification-1");
    expect(store.tasks[0].lastAction).toBe("会话本轮完成");
    expect(store.tasks[0].updatedAt).toBe("2026-05-30T01:00:00.000Z");
  });

  it("requires a task update to refresh canonical task fields", () => {
    const store = useTaskStore();
    const completion = completedEvent("complete-1", "2026-05-30T01:02:00.000Z");
    store.upsertTask({
      ...task("tool-running"),
      lastAction: "正在运行 Bash",
    });

    store.addEvent(completion);

    expect(store.tasks[0].events[0].id).toBe("complete-1");
    expect(store.tasks[0].status).toBe("tool-running");
    expect(store.tasks[0].lastAction).toBe("正在运行 Bash");
    expect(store.tasks[0].updatedAt).toBe("2026-05-30T01:00:00.000Z");

    store.upsertTask({
      ...task("completed", [completion]),
      updatedAt: "2026-05-30T01:02:00.000Z",
      lastAction: "完成任务",
    });

    expect(store.tasks[0].status).toBe("completed");
    expect(store.tasks[0].lastAction).toBe("完成任务");
    expect(store.tasks[0].updatedAt).toBe("2026-05-30T01:02:00.000Z");
  });
});
