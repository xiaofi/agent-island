import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type {
  AdapterDiagnostic,
  AgentBridgeSubscriptions,
  AgentEvent,
  AgentSource,
  AgentTask,
  AppSettings,
  HookOperation,
} from "@/domain/taskTypes";
import { cloneMockDiagnostics, cloneMockSettings, cloneMockTasks } from "@/mock/tasks";

const isTauri = () => Boolean(window.__TAURI_INTERNALS__);

export const isRunningInTauri = isTauri;

let mockTasks = cloneMockTasks();
let mockSettings = cloneMockSettings();
let sequence = 0;

export async function getTasks(): Promise<AgentTask[]> {
  if (isTauri()) {
    return invoke<AgentTask[]>("get_tasks");
  }

  return structuredClone(mockTasks);
}

export async function runDiscovery(source?: AgentSource): Promise<AdapterDiagnostic[]> {
  if (isTauri()) {
    return invoke<AdapterDiagnostic[]>("run_discovery", { source });
  }

  const diagnostics = cloneMockDiagnostics();
  return source ? diagnostics.filter((diagnostic) => diagnostic.source === source) : diagnostics;
}

export async function getSettings(): Promise<AppSettings> {
  if (isTauri()) {
    return invoke<AppSettings>("get_settings");
  }

  const saved = window.localStorage.getItem("agent-island-settings");
  if (saved) {
    mockSettings = mergeSettings(JSON.parse(saved));
  }
  return structuredClone(mockSettings);
}

export async function updateSettings(patch: Partial<AppSettings>): Promise<AppSettings> {
  if (isTauri()) {
    return invoke<AppSettings>("update_settings", { patch });
  }

  mockSettings = {
    ...mockSettings,
    ...patch,
    privacy: {
      ...mockSettings.privacy,
      ...patch.privacy,
    },
    appearance: {
      ...mockSettings.appearance,
      ...patch.appearance,
    },
    notifications: {
      ...mockSettings.notifications,
      ...patch.notifications,
    },
    autoAcknowledge: {
      ...mockSettings.autoAcknowledge,
      ...patch.autoAcknowledge,
    },
    hookSource: {
      ...mockSettings.hookSource,
      ...patch.hookSource,
      lastErrors: {
        ...mockSettings.hookSource.lastErrors,
        ...patch.hookSource?.lastErrors,
      },
    },
    islandWindow: {
      ...mockSettings.islandWindow,
      ...patch.islandWindow,
    },
  };
  window.localStorage.setItem("agent-island-settings", JSON.stringify(mockSettings));
  return structuredClone(mockSettings);
}

function mergeSettings(saved: Partial<AppSettings>): AppSettings {
  return {
    ...mockSettings,
    ...saved,
    privacy: {
      ...mockSettings.privacy,
      ...saved.privacy,
    },
    appearance: {
      ...mockSettings.appearance,
      ...saved.appearance,
    },
    notifications: {
      ...mockSettings.notifications,
      ...saved.notifications,
    },
    autoAcknowledge: {
      ...mockSettings.autoAcknowledge,
      ...saved.autoAcknowledge,
    },
    hookSource: {
      ...mockSettings.hookSource,
      ...saved.hookSource,
      lastErrors: {
        ...mockSettings.hookSource.lastErrors,
        ...saved.hookSource?.lastErrors,
      },
    },
    islandWindow: {
      ...mockSettings.islandWindow,
      ...saved.islandWindow,
    },
  };
}

export async function setHookSourceEnabled(
  source: Extract<AgentSource, "codex" | "claude-code">,
  enabled: boolean,
): Promise<AppSettings> {
  if (isTauri()) {
    return invoke<AppSettings>("set_hook_source_enabled", { source, enabled });
  }

  mockSettings = {
    ...mockSettings,
    hookSource: {
      ...mockSettings.hookSource,
      [source === "codex" ? "codex" : "claudeCode"]: enabled,
      lastErrors: {
        ...mockSettings.hookSource.lastErrors,
        [source === "codex" ? "codex" : "claudeCode"]: undefined,
      },
    },
  };
  window.localStorage.setItem("agent-island-settings", JSON.stringify(mockSettings));
  return structuredClone(mockSettings);
}

export async function retryHookSourceOperation(
  source: Extract<AgentSource, "codex" | "claude-code">,
  operation: HookOperation,
): Promise<AppSettings> {
  if (isTauri()) {
    return invoke<AppSettings>("retry_hook_source_operation", { source, operation });
  }

  return setHookSourceEnabled(source, operation !== "uninstall");
}

export async function runHookSelfTest(source: Extract<AgentSource, "codex" | "claude-code">): Promise<AppSettings> {
  if (isTauri()) {
    return invoke<AppSettings>("run_hook_self_test", { source });
  }

  return structuredClone(mockSettings);
}

export async function openTask(taskId: string): Promise<void> {
  if (isTauri()) {
    return invoke("open_task", { taskId });
  }

  console.info(`[agent-island] open task: ${taskId}`);
}

export async function openWorkdir(path?: string): Promise<void> {
  if (!path) {
    return;
  }

  if (isTauri()) {
    return invoke("open_workdir", { path });
  }

  console.info(`[agent-island] open workdir: ${path}`);
}

export async function copyTaskSummary(task: AgentTask): Promise<void> {
  const summary = `${task.source} · ${task.title} · ${task.status} · ${task.cwd ?? ""}`.trim();

  if (navigator.clipboard) {
    await navigator.clipboard.writeText(summary);
    return;
  }

  if (isTauri()) {
    return invoke("copy_task_summary", { taskId: task.id });
  }
}

export async function setMousePassthrough(enabled: boolean): Promise<void> {
  if (isTauri()) {
    return invoke("set_mouse_passthrough", { enabled });
  }
}

export async function setDockVisibility(visible: boolean): Promise<void> {
  if (isTauri()) {
    return invoke("set_dock_visibility", { visible });
  }
}

export async function saveIslandWindowPosition(): Promise<void> {
  if (isTauri()) {
    return invoke("save_island_window_position");
  }
}

export type IslandPanelDirection = "down" | "up";

export async function setWindowMode(
  expanded: boolean,
  collapsedHeight = 44,
  expansionDirection?: IslandPanelDirection,
): Promise<IslandPanelDirection> {
  if (isTauri()) {
    return invoke<IslandPanelDirection>("set_window_mode", { expanded, collapsedHeight, expansionDirection });
  }

  return "down";
}

export async function startWindowDrag(): Promise<void> {
  if (isTauri()) {
    return getCurrentWindow().startDragging();
  }
}

export async function subscribeWindowFocusChanged(onFocusChanged: (focused: boolean) => void): Promise<() => void> {
  if (isTauri()) {
    return getCurrentWindow().onFocusChanged(({ payload }) => onFocusChanged(payload));
  }

  const handleFocus = () => onFocusChanged(true);
  const handleBlur = () => onFocusChanged(false);
  window.addEventListener("focus", handleFocus);
  window.addEventListener("blur", handleBlur);

  return () => {
    window.removeEventListener("focus", handleFocus);
    window.removeEventListener("blur", handleBlur);
  };
}

export async function openAppWindow(kind: "settings" | "diagnostics"): Promise<void> {
  if (isTauri()) {
    return invoke("open_app_window", { kind });
  }

  window.open(`/?window=${kind}`, `agent-island-${kind}`, "width=760,height=620");
}

export async function subscribeAgentEvents(subscriptions: AgentBridgeSubscriptions): Promise<() => void> {
  if (isTauri()) {
    const unlisteners: UnlistenFn[] = await Promise.all([
      listen<AgentTask>("agent-task-updated", (event) => subscriptions.onTaskUpdated(event.payload)),
      listen<string>("agent-task-removed", (event) => subscriptions.onTaskRemoved(event.payload)),
      listen<AgentEvent>("agent-event-created", (event) => subscriptions.onEventCreated(event.payload)),
      listen<AdapterDiagnostic>("adapter-diagnostic-updated", (event) =>
        subscriptions.onDiagnosticUpdated(event.payload),
      ),
      listen<AppSettings>("settings-updated", (event) => subscriptions.onSettingsUpdated(event.payload)),
    ]);

    return () => {
      for (const unlisten of unlisteners) {
        unlisten();
      }
    };
  }

  const interval = window.setInterval(() => {
    sequence += 1;
    const task = mockTasks[sequence % mockTasks.length];
    const event: AgentEvent = {
      id: `${task.id}-mock-${sequence}`,
      taskId: task.id,
      type: task.status === "waiting-user" ? "waiting-for-user" : "heartbeat",
      timestamp: new Date().toISOString(),
      summary: nextMockAction(task.status, sequence),
    };

    const updated: AgentTask = {
      ...task,
      updatedAt: event.timestamp,
      lastAction: event.summary,
      events: [event, ...task.events].slice(0, 10),
    };

    mockTasks = mockTasks.map((item) => (item.id === updated.id ? updated : item));
    subscriptions.onEventCreated(event);
    subscriptions.onTaskUpdated(structuredClone(updated));
  }, 4500);

  return () => window.clearInterval(interval);
}

function nextMockAction(status: AgentTask["status"], tick: number) {
  if (status === "waiting-user") {
    return "仍在等待用户确认";
  }

  const actions = ["读取文件", "分析状态事件", "更新任务摘要", "检查 discovery 来源", "运行 mock 心跳"];
  return actions[tick % actions.length];
}
