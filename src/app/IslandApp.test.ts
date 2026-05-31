// @vitest-environment happy-dom

import { flushPromises, mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";
import IslandApp from "@/app/IslandApp.vue";
import type { AgentTask } from "@/domain/taskTypes";
import { useTaskStore } from "@/stores/taskStore";

vi.mock("@/bridge/tauriApi", () => ({
  getSettings: vi.fn(async () => ({
    privacy: {
      hideProjectPath: false,
      hideTaskTitle: false,
      compactOnly: false,
    },
    mousePassthrough: false,
    enabledAdapters: ["manual", "codex", "claude-code"],
    hookSource: {
      codex: false,
      claudeCode: false,
      lastErrors: {},
    },
  })),
  getTasks: vi.fn(async () => []),
  isRunningInTauri: vi.fn(() => false),
  openAppWindow: vi.fn(),
  runDiscovery: vi.fn(async () => []),
  setWindowMode: vi.fn(),
  startWindowDrag: vi.fn(),
  subscribeAgentEvents: vi.fn(async () => () => undefined),
}));

function completedTask(id: string, title: string): AgentTask {
  return {
    id,
    source: "codex",
    title,
    status: "completed",
    updatedAt: "2026-05-30T01:00:00.000Z",
    events: [],
  };
}

function runningTask(id: string, title: string): AgentTask {
  return {
    id,
    source: "codex",
    title,
    status: "thinking",
    updatedAt: "2026-05-30T01:01:00.000Z",
    events: [],
  };
}

describe("IslandApp", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it("keeps the compact island shell when there is only one attention alert", async () => {
    const taskStore = useTaskStore();
    taskStore.tasks = [completedTask("task-a", "A")];

    const wrapper = mount(IslandApp);
    await flushPromises();

    expect(wrapper.find(".island-trigger").classes()).not.toContain("island-trigger--stacked");
  });

  it("uses a card-like island shell when attention alerts are stacked over the running summary", async () => {
    const taskStore = useTaskStore();
    taskStore.tasks = [completedTask("task-a", "A"), completedTask("task-b", "B")];

    const wrapper = mount(IslandApp);
    await flushPromises();

    expect(wrapper.find(".island-trigger").classes()).toContain("island-trigger--stacked");
  });

  it("uses a card-like island shell when one alert coexists with running work", async () => {
    const taskStore = useTaskStore();
    taskStore.tasks = [completedTask("task-a", "A"), runningTask("task-b", "B")];

    const wrapper = mount(IslandApp);
    await flushPromises();

    expect(wrapper.find(".island-trigger").classes()).toContain("island-trigger--stacked");
  });
});
