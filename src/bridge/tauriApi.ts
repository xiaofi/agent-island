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
} from "@/domain/taskTypes";
import { cloneMockDiagnostics, cloneMockSettings, cloneMockTasks } from "@/mock/tasks";

const isTauri = () => Boolean(window.__TAURI_INTERNALS__);

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
    mockSettings = { ...mockSettings, ...JSON.parse(saved) };
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
  };
  window.localStorage.setItem("agent-island-settings", JSON.stringify(mockSettings));
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

export async function setWindowMode(expanded: boolean): Promise<void> {
  if (isTauri()) {
    return invoke("set_window_mode", { expanded });
  }
}

export async function startWindowDrag(): Promise<void> {
  if (isTauri()) {
    return getCurrentWindow().startDragging();
  }
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
