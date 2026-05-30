// @vitest-environment happy-dom

import { mount } from "@vue/test-utils";
import { nextTick } from "vue";
import { afterEach, describe, expect, it, vi } from "vitest";
import IslandCollapsed from "@/components/island/IslandCollapsed.vue";
import type { AgentTask } from "@/domain/taskTypes";

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

function activeTask(id: string, title: string): AgentTask {
  return {
    id,
    source: "codex",
    title,
    status: "running",
    updatedAt: "2026-05-30T01:00:00.000Z",
    events: [],
  };
}

describe("IslandCollapsed", () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it("renders each completed task as its own confirmation row", async () => {
    const wrapper = mount(IslandCollapsed, {
      props: {
        completedTasks: [completedTask("task-cmux", "cmux"), completedTask("task-agent-island", "agent-island")],
        tasks: [],
        activeCount: 3,
        waitingCount: 0,
        loading: false,
      },
    });

    const rows = wrapper.findAll(".collapsed-island__row--confirmation");
    expect(rows).toHaveLength(2);
    expect(rows[0].text()).toContain("codex · 已完成 · cmux");
    expect(rows[1].text()).toContain("codex · 已完成 · agent-island");
    expect(wrapper.text()).toContain("3 任务");

    const ackButtons = wrapper.findAll(".collapsed-island__ack");
    expect(ackButtons).toHaveLength(2);

    await ackButtons[1].trigger("click");

    expect(wrapper.emitted("acknowledgeCompleted")).toEqual([[["task-agent-island"]]]);
  });

  it("caps completed confirmation rows and sends overflow users to the list", async () => {
    const wrapper = mount(IslandCollapsed, {
      props: {
        completedTasks: [
          completedTask("task-1", "one"),
          completedTask("task-2", "two"),
          completedTask("task-3", "three"),
          completedTask("task-4", "four"),
          completedTask("task-5", "five"),
          completedTask("task-6", "six"),
        ],
        tasks: [],
        activeCount: 6,
        waitingCount: 0,
        loading: false,
      },
    });

    expect(wrapper.findAll(".collapsed-island__row--confirmation")).toHaveLength(5);
    expect(wrapper.findAll(".collapsed-island__ack")).toHaveLength(4);
    expect(wrapper.text()).toContain("请点击展开列表查看");

    await wrapper.find(".collapsed-island__row--overflow").trigger("click");

    expect(wrapper.emitted("expand")).toHaveLength(1);
  });

  it("rotates active tasks on the carousel interval", async () => {
    vi.useFakeTimers();

    const wrapper = mount(IslandCollapsed, {
      props: {
        completedTasks: [],
        tasks: [activeTask("task-cmux", "cmux"), activeTask("task-agent-island", "agent-island")],
        activeCount: 2,
        waitingCount: 0,
        loading: false,
      },
    });

    expect(wrapper.text()).toContain("codex · 运行中 · cmux");

    vi.advanceTimersByTime(5000);
    await nextTick();

    expect(wrapper.text()).toContain("codex · 运行中 · agent-island");
  });
});
