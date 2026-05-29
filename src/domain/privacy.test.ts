import { describe, expect, it } from "vitest";
import { maskTask, projectNameFromPath } from "@/domain/privacy";
import type { AgentTask } from "@/domain/taskTypes";

const task: AgentTask = {
  id: "task-1",
  source: "codex",
  title: "Implement a secret project",
  cwd: "/Users/spf/project/agent-island",
  status: "running",
  updatedAt: "2026-05-29T10:00:00.000Z",
  events: [],
};

describe("privacy", () => {
  it("extracts the project name from a full path", () => {
    expect(projectNameFromPath("/Users/spf/project/agent-island")).toBe("agent-island");
  });

  it("masks path and title independently", () => {
    const masked = maskTask(task, {
      hideProjectPath: true,
      hideTaskTitle: true,
      compactOnly: false,
    });

    expect(masked.title).toBe("Codex task");
    expect(masked.cwd).toBe("agent-island");
  });
});
