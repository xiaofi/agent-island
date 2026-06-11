import { isPermissionGranted, requestPermission } from "@tauri-apps/plugin-notification";
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
