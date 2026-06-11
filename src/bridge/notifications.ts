import { isPermissionGranted, requestPermission, sendNotification } from "@tauri-apps/plugin-notification";
import { isRunningInTauri } from "@/bridge/tauriApi";
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
    const soundName = notificationSoundName(sound);
    sendNotification({
      title: "Agent Island 测试通知",
      body: "如果你看到这条通知，系统通知已生效。",
      group: "agent-island-tests",
      ...(soundName ? { sound: soundName } : {}),
    });
    return "sent";
  } catch (error) {
    console.warn("[agent-island] failed to send test notification", error);
    return "failed";
  }
}

function notificationSoundName(sound: NotificationSound): string | undefined {
  if (sound === "none") {
    return undefined;
  }

  if (sound === "default") {
    return "NSUserNotificationDefaultSoundName";
  }

  return sound;
}
