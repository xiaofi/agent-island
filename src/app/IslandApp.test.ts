// @vitest-environment happy-dom

import { flushPromises, mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";
import IslandApp from "@/app/IslandApp.vue";
import { setWindowMode, subscribeWindowFocusChanged } from "@/bridge/tauriApi";
import type { AgentTask } from "@/domain/taskTypes";
import { usePreferencesStore } from "@/stores/preferencesStore";
import { useTaskStore } from "@/stores/taskStore";

vi.mock("@/bridge/tauriApi", () => ({
  getSettings: vi.fn(async () => ({
    privacy: {
      hideProjectPath: false,
      hideTaskTitle: false,
      compactOnly: false,
    },
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
    mousePassthrough: false,
    showInDock: false,
    enabledAdapters: ["manual", "codex", "claude-code"],
    hookSource: {
      codex: false,
      claudeCode: false,
      lastErrors: {},
    },
    islandWindow: {},
  })),
  getTasks: vi.fn(async () => []),
  isRunningInTauri: vi.fn(() => false),
  openAppWindow: vi.fn(),
  runDiscovery: vi.fn(async () => []),
  saveIslandWindowPosition: vi.fn(),
  setDockVisibility: vi.fn(),
  setWindowMode: vi.fn(),
  startWindowDrag: vi.fn(),
  subscribeWindowFocusChanged: vi.fn(async () => () => undefined),
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

function pausedTask(id: string, title: string): AgentTask {
  return {
    id,
    source: "codex",
    title,
    status: "paused",
    updatedAt: "2026-05-30T01:02:00.000Z",
    events: [],
  };
}

describe("IslandApp", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.mocked(setWindowMode).mockResolvedValue("down");
    vi.mocked(subscribeWindowFocusChanged).mockResolvedValue(() => undefined);
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

  it("keeps the expand entry when only multiple alert tasks are visible", async () => {
    vi.useFakeTimers();

    try {
      const taskStore = useTaskStore();
      taskStore.tasks = [completedTask("task-a", "A"), completedTask("task-b", "B")];

      const wrapper = mount(IslandApp);
      await flushPromises();

      const summaryButton = wrapper.find("button.collapsed-island__meta");
      expect(summaryButton.exists()).toBe(true);
      expect(summaryButton.text()).toBe("显示全部任务");
      expect(wrapper.text()).toContain("2 条任务需要关注");
      expect(wrapper.text()).not.toContain("暂无任务进行中");

      await summaryButton.trigger("click");
      await flushPromises();
      await vi.advanceTimersByTimeAsync(260);
      await flushPromises();

      expect(wrapper.find(".app-shell").classes()).toContain("app-shell--expanded");
      expect(wrapper.find("h1").text()).toBe("活跃任务");
    } finally {
      vi.useRealTimers();
    }
  });

  it("uses a card-like island shell when one alert coexists with running work", async () => {
    const taskStore = useTaskStore();
    taskStore.tasks = [completedTask("task-a", "A"), runningTask("task-b", "B")];

    const wrapper = mount(IslandApp);
    await flushPromises();

    expect(wrapper.find(".island-trigger").classes()).toContain("island-trigger--stacked");
  });

  it("does not promote user-paused tasks as collapsed alerts", async () => {
    const taskStore = useTaskStore();
    taskStore.tasks = [pausedTask("task-a", "A"), runningTask("task-b", "B")];

    const wrapper = mount(IslandApp);
    await flushPromises();

    expect(wrapper.find(".island-trigger").classes()).not.toContain("island-trigger--stacked");
    expect(wrapper.text()).toContain("1 个任务进行中");
    expect(wrapper.text()).not.toContain("已暂停");
  });

  it("hides ordinary running work in quiet mode", async () => {
    const preferencesStore = usePreferencesStore();
    preferencesStore.settings.quietMode = true;

    const taskStore = useTaskStore();
    taskStore.tasks = [runningTask("task-a", "A"), completedTask("task-b", "B")];
    taskStore.loading = true;

    const wrapper = mount(IslandApp);
    await flushPromises();

    expect(wrapper.text()).toContain("已完成");
    expect(wrapper.text()).not.toContain("任务进行中");
    expect(wrapper.text()).not.toContain("正在发现本地 agent");
    expect(wrapper.find(".island-trigger").classes()).not.toContain("island-trigger--stacked");
  });

  it("applies the configured island opacity and compact privacy text", async () => {
    const preferencesStore = usePreferencesStore();
    preferencesStore.settings.appearance.islandOpacity = 0.7;
    preferencesStore.settings.privacy.compactOnly = true;

    const taskStore = useTaskStore();
    taskStore.tasks = [completedTask("task-a", "Sensitive title")];

    const wrapper = mount(IslandApp);
    await flushPromises();

    expect(wrapper.find(".island-window").attributes("style")).toContain("--island-opacity: 0.7");
    expect(wrapper.text()).toContain("codex · 已完成");
    expect(wrapper.text()).not.toContain("Sensitive title");
  });

  it("clears a task from the detail view with the manual delete action", async () => {
    vi.useFakeTimers();

    try {
      const taskStore = useTaskStore();
      taskStore.tasks = [runningTask("task-a", "Stuck task")];

      const wrapper = mount(IslandApp);
      await flushPromises();

      await wrapper.find("button.collapsed-island__meta").trigger("click");
      await flushPromises();
      await vi.advanceTimersByTimeAsync(260);
      await flushPromises();

      await wrapper.find(".task-card").trigger("click");
      await flushPromises();

      expect(wrapper.text()).toContain("Stuck task");

      await wrapper.find('button[title="清除任务状态"]').trigger("click");
      await flushPromises();

      expect(taskStore.tasks).toHaveLength(0);
      expect(wrapper.text()).toContain("没有发现活跃 agent 会话");
    } finally {
      vi.useRealTimers();
    }
  });

  it("lays out the expanded panel above the trigger when the native window opens upward", async () => {
    vi.mocked(setWindowMode).mockResolvedValueOnce("down").mockResolvedValueOnce("up");
    const taskStore = useTaskStore();
    taskStore.tasks = [runningTask("task-a", "A")];

    const wrapper = mount(IslandApp);
    await flushPromises();
    await wrapper.find(".collapsed-island__row--summary").trigger("click");
    await flushPromises();

    expect(wrapper.find(".app-shell").classes()).toContain("app-shell--expanded");
    expect(wrapper.find(".app-shell").classes()).toContain("app-shell--expand-up");
  });

  it("ignores repeated summary toggles while native window mode is pending", async () => {
    vi.useFakeTimers();

    try {
      let resolveExpandedMode: (direction: "down") => void = () => undefined;
      const expandedMode = new Promise<"down">((resolve) => {
        resolveExpandedMode = resolve;
      });
      vi.mocked(setWindowMode).mockImplementation(async (expanded) => (expanded ? expandedMode : "down"));

      const taskStore = useTaskStore();
      taskStore.tasks = [runningTask("task-a", "A")];

      const wrapper = mount(IslandApp);
      await flushPromises();
      vi.mocked(setWindowMode).mockClear();

      const summaryButton = wrapper.find("button.collapsed-island__meta");
      await summaryButton.trigger("click");
      await summaryButton.trigger("click");

      expect(summaryButton.attributes("disabled")).toBeDefined();
      expect(setWindowMode).toHaveBeenCalledTimes(1);
      expect(vi.mocked(setWindowMode).mock.calls[0]?.[0]).toBe(true);

      resolveExpandedMode("down");
      await flushPromises();
      await vi.advanceTimersByTimeAsync(260);
      await flushPromises();

      expect(wrapper.find(".app-shell").classes()).toContain("app-shell--expanded");
      expect(wrapper.find("button.collapsed-island__meta").attributes("disabled")).toBeUndefined();
    } finally {
      vi.useRealTimers();
    }
  });

  it("finishes collapse if the panel leave callback is delayed", async () => {
    vi.useFakeTimers();

    try {
      const taskStore = useTaskStore();
      taskStore.tasks = [runningTask("task-a", "A")];

      const wrapper = mount(IslandApp);
      await flushPromises();

      await wrapper.find("button.collapsed-island__meta").trigger("click");
      await flushPromises();
      await vi.advanceTimersByTimeAsync(260);
      await flushPromises();

      expect(wrapper.find(".app-shell").classes()).toContain("app-shell--expanded");
      vi.mocked(setWindowMode).mockClear();

      await wrapper.find("button.collapsed-island__meta").trigger("click");
      expect(wrapper.find("button.collapsed-island__meta").attributes("disabled")).toBeDefined();

      await vi.advanceTimersByTimeAsync(900);
      await flushPromises();

      expect(vi.mocked(setWindowMode).mock.calls[0]?.[0]).toBe(false);
      expect(wrapper.find(".app-shell").classes()).not.toContain("app-shell--expanded");
      expect(wrapper.find("button.collapsed-island__meta").attributes("disabled")).toBeUndefined();
    } finally {
      vi.useRealTimers();
    }
  });

  it("collapses the expanded list when the island window loses focus", async () => {
    vi.useFakeTimers();

    try {
      let focusChanged: ((focused: boolean) => void) | undefined;
      vi.mocked(subscribeWindowFocusChanged).mockImplementation(async (handler) => {
        focusChanged = handler;
        return () => undefined;
      });

      const taskStore = useTaskStore();
      taskStore.tasks = [runningTask("task-a", "A")];

      const wrapper = mount(IslandApp);
      await flushPromises();

      await wrapper.find("button.collapsed-island__meta").trigger("click");
      await flushPromises();
      await vi.advanceTimersByTimeAsync(260);
      await flushPromises();

      expect(wrapper.find(".app-shell").classes()).toContain("app-shell--expanded");
      vi.mocked(setWindowMode).mockClear();

      focusChanged?.(false);
      await vi.advanceTimersByTimeAsync(900);
      await flushPromises();

      expect(vi.mocked(setWindowMode).mock.calls[0]?.[0]).toBe(false);
      expect(wrapper.find(".app-shell").classes()).not.toContain("app-shell--expanded");
      expect(wrapper.find("button.collapsed-island__meta").text()).toBe("显示全部任务");
    } finally {
      vi.useRealTimers();
    }
  });

  it("waits until the opening guard finishes before collapsing on focus loss", async () => {
    vi.useFakeTimers();

    try {
      let focusChanged: ((focused: boolean) => void) | undefined;
      vi.mocked(subscribeWindowFocusChanged).mockImplementation(async (handler) => {
        focusChanged = handler;
        return () => undefined;
      });

      const taskStore = useTaskStore();
      taskStore.tasks = [runningTask("task-a", "A")];

      const wrapper = mount(IslandApp);
      await flushPromises();

      await wrapper.find("button.collapsed-island__meta").trigger("click");
      await flushPromises();
      expect(wrapper.find("button.collapsed-island__meta").attributes("disabled")).toBeDefined();

      vi.mocked(setWindowMode).mockClear();
      focusChanged?.(false);
      await flushPromises();

      expect(vi.mocked(setWindowMode)).not.toHaveBeenCalled();

      await vi.advanceTimersByTimeAsync(260);
      await flushPromises();
      await vi.advanceTimersByTimeAsync(900);
      await flushPromises();

      expect(vi.mocked(setWindowMode).mock.calls[0]?.[0]).toBe(false);
      expect(wrapper.find(".app-shell").classes()).not.toContain("app-shell--expanded");
    } finally {
      vi.useRealTimers();
    }
  });
});
