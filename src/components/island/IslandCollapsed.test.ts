// @vitest-environment happy-dom

import { mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";
import IslandCollapsed from "@/components/island/IslandCollapsed.vue";
import { startWindowDrag } from "@/bridge/tauriApi";
import type { AgentTask, TaskStatus } from "@/domain/taskTypes";

vi.mock("@/bridge/tauriApi", () => ({
  startWindowDrag: vi.fn(),
}));

function task(id: string, title: string, status: TaskStatus): AgentTask {
  return {
    id,
    source: "codex",
    title,
    status,
    updatedAt: "2026-05-30T01:00:00.000Z",
    events: [],
  };
}

describe("IslandCollapsed", () => {
  beforeEach(() => {
    vi.mocked(startWindowDrag).mockReset();
  });

  it("keeps a single attention item in the compact island shape", async () => {
    const wrapper = mount(IslandCollapsed, {
      props: {
        alertTasks: [task("task-cmux", "cmux", "completed")],
        runningCount: 0,
        loading: false,
        expanded: false,
      },
    });

    expect(wrapper.find(".collapsed-island").classes()).not.toContain("collapsed-island--stacked");
    expect(wrapper.findAll(".collapsed-island__row")).toHaveLength(1);
    expect(wrapper.text()).toContain("codex · 已完成 · cmux");
    expect(wrapper.text()).toContain("显示全部任务");
    expect(wrapper.text()).not.toContain("暂无任务进行中");

    await wrapper.find(".collapsed-island__ack").trigger("click");

    expect(wrapper.emitted("acknowledgeCompleted")).toEqual([[["task-cmux"]]]);
    expect(wrapper.emitted("expand")).toBeUndefined();
  });

  it("shows a running summary when one attention item coexists with running work", async () => {
    const wrapper = mount(IslandCollapsed, {
      props: {
        alertTasks: [task("task-cmux", "cmux", "completed")],
        runningCount: 1,
        loading: false,
        expanded: false,
      },
    });

    expect(wrapper.find(".collapsed-island").classes()).toContain("collapsed-island--stacked");
    const rows = wrapper.findAll(".collapsed-island__row");
    expect(rows).toHaveLength(2);
    expect(rows[0].text()).toContain("codex · 已完成 · cmux");
    expect(rows[1].text()).toContain("1 个任务进行中");

    await rows[1].trigger("click");
    expect(wrapper.emitted("expand")).toHaveLength(1);
  });

  it("renders multiple attention rows above the running summary row", async () => {
    const wrapper = mount(IslandCollapsed, {
      props: {
        alertTasks: [task("task-cmux", "cmux", "completed"), task("task-agent-island", "agent-island", "waiting-user")],
        runningCount: 3,
        loading: false,
        expanded: false,
      },
    });

    expect(wrapper.find(".collapsed-island").classes()).toContain("collapsed-island--stacked");
    const alertRows = wrapper.findAll(".collapsed-island__row--alert");
    expect(alertRows).toHaveLength(2);
    expect(alertRows[0].text()).toContain("codex · 已完成 · cmux");
    expect(alertRows[1].text()).toContain("codex · 等待处理 · agent-island");

    const summaryRow = wrapper.find(".collapsed-island__row--summary");
    expect(summaryRow.text()).toContain("3 个任务进行中");
    expect(summaryRow.text()).toContain("显示全部任务");
    expect(summaryRow.find(".status-dot--running").exists()).toBe(true);

    await alertRows[1].trigger("click");
    expect(wrapper.emitted("expand")).toBeUndefined();

    await summaryRow.trigger("click");
    expect(wrapper.emitted("expand")).toHaveLength(1);
  });

  it("keeps completion acknowledgement on completed alert rows", async () => {
    const wrapper = mount(IslandCollapsed, {
      props: {
        alertTasks: [task("task-cmux", "cmux", "completed"), task("task-agent-island", "agent-island", "failed")],
        runningCount: 1,
        loading: false,
        expanded: false,
      },
    });

    const ackButtons = wrapper.findAll(".collapsed-island__ack");
    expect(ackButtons).toHaveLength(1);

    await ackButtons[0].trigger("click");

    expect(wrapper.emitted("acknowledgeCompleted")).toEqual([[["task-cmux"]]]);
    expect(wrapper.emitted("expand")).toBeUndefined();
  });

  it("caps attention rows and points users to the full task list from the summary row", async () => {
    const wrapper = mount(IslandCollapsed, {
      props: {
        alertTasks: [
          task("task-1", "one", "completed"),
          task("task-2", "two", "waiting-user"),
          task("task-3", "three", "failed"),
          task("task-4", "four", "paused"),
          task("task-5", "five", "stale"),
        ],
        runningCount: 2,
        loading: false,
        expanded: false,
      },
    });

    expect(wrapper.findAll(".collapsed-island__row--alert")).toHaveLength(4);
    expect(wrapper.text()).toContain("另有 2 条任务需要关注");

    await wrapper.find(".collapsed-island__row--overflow").trigger("click");
    expect(wrapper.emitted("expand")).toBeUndefined();

    await wrapper.find(".collapsed-island__row--summary").trigger("click");
    expect(wrapper.emitted("expand")).toHaveLength(1);
  });

  it("switches the summary action label when the task list is expanded", () => {
    const wrapper = mount(IslandCollapsed, {
      props: {
        alertTasks: [],
        runningCount: 2,
        loading: false,
        expanded: true,
      },
    });

    expect(wrapper.find(".collapsed-island__row--summary").text()).toContain("收起列表");
    expect(wrapper.text()).not.toContain("显示全部任务");
  });

  it("marks collapsed content as draggable and blocks text selection while dragging", async () => {
    const wrapper = mount(IslandCollapsed, {
      props: {
        alertTasks: [task("task-cmux", "cmux", "completed")],
        runningCount: 0,
        loading: false,
        expanded: false,
      },
    });

    const root = wrapper.find(".collapsed-island");
    const text = wrapper.find(".collapsed-island__text");

    expect(wrapper.find(".collapsed-island__left").attributes("data-tauri-drag-region")).toBe("");
    expect(text.attributes("data-tauri-drag-region")).toBe("");
    expect(wrapper.find(".collapsed-island__meta").attributes("data-tauri-drag-region")).toBeUndefined();

    await root.trigger("pointerdown", {
      button: 0,
      clientX: 10,
      clientY: 10,
      pointerId: 1,
    });
    await root.trigger("pointermove", {
      clientX: 22,
      clientY: 10,
    });

    expect(startWindowDrag).toHaveBeenCalledTimes(1);

    await wrapper.find(".collapsed-island__row--summary").trigger("click");
    expect(wrapper.emitted("expand")).toBeUndefined();
  });

  it("keeps the summary action clickable", async () => {
    const wrapper = mount(IslandCollapsed, {
      props: {
        alertTasks: [task("task-cmux", "cmux", "completed")],
        runningCount: 0,
        loading: false,
        expanded: false,
      },
    });

    const summaryAction = wrapper.find("button.collapsed-island__meta");

    expect(summaryAction.text()).toBe("显示全部任务");
    expect(summaryAction.attributes("data-tauri-drag-region")).toBeUndefined();

    await summaryAction.trigger("click");

    expect(wrapper.emitted("expand")).toHaveLength(1);
    expect(startWindowDrag).not.toHaveBeenCalled();
  });
});
