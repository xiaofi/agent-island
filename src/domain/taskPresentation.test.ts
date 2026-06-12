import { describe, expect, it } from "vitest";
import { projectNameFromPath } from "@/domain/taskPresentation";

describe("taskPresentation", () => {
  it("extracts the project name from a full path", () => {
    expect(projectNameFromPath("/Users/spf/project/agent-island")).toBe("agent-island");
  });
});
