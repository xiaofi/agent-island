// @vitest-environment happy-dom

import { mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it } from "vitest";
import TaskCard from "@/components/island/TaskCard.vue";
import type { AgentTask } from "@/domain/taskTypes";

const baseTask: AgentTask = {
  id: "task-cmux",
  source: "codex",
  title: "cmux",
  cwd: "/Users/spf/project/cmux",
  status: "completed",
  updatedAt: "2026-05-30T01:00:00.000Z",
  events: [],
};

describe("TaskCard", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it("lets completed tasks be acknowledged without selecting the card", async () => {
    const wrapper = mount(TaskCard, {
      props: {
        task: baseTask,
      },
    });

    await wrapper.find(".task-card__ack").trigger("click");

    expect(wrapper.emitted("acknowledgeCompleted")).toHaveLength(1);
    expect(wrapper.emitted("select")).toBeUndefined();
  });

  it("does not show an acknowledge button for active tasks", () => {
    const wrapper = mount(TaskCard, {
      props: {
        task: { ...baseTask, status: "running" },
      },
    });

    expect(wrapper.find(".task-card__ack").exists()).toBe(false);
  });
});
