import { isPermissionGranted, requestPermission, sendNotification } from "@tauri-apps/plugin-notification";
import type { AgentTask } from "@/domain/taskTypes";
import { sourceLabel } from "@/domain/privacy";
import { isRunningInTauri } from "@/bridge/tauriApi";

export async function ensureNotificationsPermission() {
  if (!isRunningInTauri()) {
    return true;
  }

  if (await isPermissionGranted()) {
    return true;
  }

  return (await requestPermission()) === "granted";
}

export async function sendTaskCompletedNotification(task: AgentTask) {
  if (!isRunningInTauri()) {
    return;
  }

  if (!(await ensureNotificationsPermission())) {
    return;
  }

  const source = sourceLabel(task.source);
  const title = task.title ? `${source} 任务已完成` : "任务已完成";
  const body = task.title || source;

  sendNotification({
    title: "Agent Island",
    body: `${title}: ${body}`,
  });
}
