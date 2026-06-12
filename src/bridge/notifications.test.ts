import { isPermissionGranted, requestPermission } from "@tauri-apps/plugin-notification";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { isRunningInTauri, sendTestNotificationCommand } from "@/bridge/tauriApi";
import { ensureNotificationsPermission, sendTestNotification } from "@/bridge/notifications";

vi.mock("@tauri-apps/plugin-notification", () => ({
  isPermissionGranted: vi.fn(),
  requestPermission: vi.fn(),
}));

vi.mock("@/bridge/tauriApi", () => ({
  isRunningInTauri: vi.fn(),
  sendTestNotificationCommand: vi.fn(),
}));

describe("notification bridge", () => {
  const isRunningInTauriMock = vi.mocked(isRunningInTauri);
  const isPermissionGrantedMock = vi.mocked(isPermissionGranted);
  const requestPermissionMock = vi.mocked(requestPermission);
  const sendTestNotificationCommandMock = vi.mocked(sendTestNotificationCommand);

  beforeEach(() => {
    vi.clearAllMocks();
    isRunningInTauriMock.mockReturnValue(true);
    isPermissionGrantedMock.mockResolvedValue(false);
    requestPermissionMock.mockResolvedValue("granted");
    sendTestNotificationCommandMock.mockResolvedValue();
  });

  it("skips native notification work outside Tauri", async () => {
    isRunningInTauriMock.mockReturnValue(false);

    await expect(sendTestNotification("Ping")).resolves.toBe("sent");

    expect(isPermissionGrantedMock).not.toHaveBeenCalled();
    expect(requestPermissionMock).not.toHaveBeenCalled();
    expect(sendTestNotificationCommandMock).not.toHaveBeenCalled();
  });

  it("requests permission before sending the native test notification", async () => {
    await expect(sendTestNotification("Hero")).resolves.toBe("sent");

    expect(requestPermissionMock).toHaveBeenCalledTimes(1);
    expect(sendTestNotificationCommandMock).toHaveBeenCalledWith("Hero");
  });

  it("does not send the test notification when permission is denied", async () => {
    requestPermissionMock.mockResolvedValue("denied");

    await expect(sendTestNotification("Ping")).resolves.toBe("permission-denied");

    expect(sendTestNotificationCommandMock).not.toHaveBeenCalled();
  });

  it("returns failed when the native test notification command fails", async () => {
    sendTestNotificationCommandMock.mockRejectedValue(new Error("notify failed"));

    await expect(sendTestNotification("default")).resolves.toBe("failed");
  });

  it("treats non-Tauri notification permission as granted", async () => {
    isRunningInTauriMock.mockReturnValue(false);

    await expect(ensureNotificationsPermission()).resolves.toBe(true);
  });
});
