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

    await wrapper.find(".collapsed-island__ack").trigger("click");

    expect(wrapper.emitted("acknowledgeCompleted")).toEqual([[["task-cmux", "task-agent-island"]]]);
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
