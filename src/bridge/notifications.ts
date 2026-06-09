import { isPermissionGranted, requestPermission, sendNotification } from "@tauri-apps/plugin-notification";
import type { AgentTask } from "@/domain/taskTypes";
import { sourceLabel } from "@/domain/privacy";
import { isRunningInTauri } from "@/bridge/tauriApi";

export async function ensureNotificationsPermission() {
  if (!isRunningInTauri()) {
    return true;
  }

  try {
    if (await isPermissionGranted()) {
      return true;
    }

    return (await requestPermission()) === "granted";
  } catch (error) {
    console.warn("[agent-island] failed to request notification permission", error);
    return false;
  }
}

export async function sendTaskStatusNotification(task: AgentTask) {
  if (!isRunningInTauri()) {
    return;
  }

  try {
    if (!(await isPermissionGranted())) {
      return;
    }

    const source = sourceLabel(task.source);
    const status = taskStatusNotificationLabel(task);
    const title = `${source} ${status}`;
    const body = task.title || source;

    sendNotification({ title, body, group: "agent-island-tasks" });
  } catch (error) {
    console.error("[agent-island] failed to send task status notification", error);
  }
}

function taskStatusNotificationLabel(task: AgentTask) {
  switch (task.status) {
    case "waiting-user":
      return "等待处理";
    case "failed":
      return "任务失败";
    case "completed":
      return "任务已完成";
    default:
      return "状态更新";
  }
}
