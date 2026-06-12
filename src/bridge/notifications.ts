import { isPermissionGranted, requestPermission } from "@tauri-apps/plugin-notification";
import { isRunningInTauri, sendTestNotificationCommand } from "@/bridge/tauriApi";
import type { NotificationSound } from "@/domain/taskTypes";

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

export type TestNotificationResult = "sent" | "permission-denied" | "failed";

export async function sendTestNotification(sound: NotificationSound): Promise<TestNotificationResult> {
  if (!isRunningInTauri()) {
    return "sent";
  }

  const granted = await ensureNotificationsPermission();
  if (!granted) {
    return "permission-denied";
  }

  try {
    await sendTestNotificationCommand(sound);
    return "sent";
  } catch (error) {
    console.warn("[agent-island] failed to send test notification", error);
    return "failed";
  }
}
