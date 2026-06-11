// @vitest-environment happy-dom

import { mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { nextTick } from "vue";
import { beforeEach, describe, expect, it, vi } from "vitest";
import SettingsPanel from "@/components/settings/SettingsPanel.vue";
import * as notificationsBridge from "@/bridge/notifications";
import type { AdapterDiagnostic, AppSettings } from "@/domain/taskTypes";
import { usePreferencesStore } from "@/stores/preferencesStore";
import { useTaskStore } from "@/stores/taskStore";

const baseSettings: AppSettings = {
  privacy: {
    hideProjectPath: false,
    hideTaskTitle: false,
    compactOnly: false,
  },
  appearance: {
    islandOpacity: 0.92,
  },
  notifications: {
    enabled: false,
    sound: "default",
  },
  autoAcknowledge: {
    enabled: false,
    delaySeconds: 900,
  },
  quietMode: false,
  mousePassthrough: false,
  showInDock: false,
  enabledAdapters: ["manual", "codex", "claude-code"],
  hookSource: {
    codex: false,
    claudeCode: false,
    lastErrors: {},
  },
  islandWindow: {},
};

function diagnostic(source: "codex" | "claude-code", status: AdapterDiagnostic["status"]): AdapterDiagnostic {
  return {
    source,
    status,
    summary: status === "unavailable" ? `未发现 ${source}` : `发现 ${source}`,
    processes: [],
    candidatePaths: [],
    parsedSessions: status === "unavailable" ? 0 : 1,
    updatedAt: new Date("2026-05-30T00:00:00.000Z").toISOString(),
  };
}

function mountPanel(settings: AppSettings, diagnostics: AdapterDiagnostic[]) {
  const pinia = createPinia();
  setActivePinia(pinia);

  const preferencesStore = usePreferencesStore();
  preferencesStore.replaceSettings(settings);

  const taskStore = useTaskStore();
  for (const item of diagnostics) {
    taskStore.upsertDiagnostic(item);
  }

  return mount(SettingsPanel, {
    global: {
      plugins: [pinia],
    },
  });
}

describe("SettingsPanel hook source controls", () => {
  beforeEach(() => {
    window.localStorage.clear();
    vi.restoreAllMocks();
  });

  it("shows an unavailable message instead of hook switches when sources are not discovered", () => {
    const wrapper = mountPanel(structuredClone(baseSettings), [
      diagnostic("codex", "unavailable"),
      diagnostic("claude-code", "unavailable"),
    ]);

    expect(wrapper.text()).toContain("未发现 Codex 安装");
    expect(wrapper.text()).toContain("未发现 Claude Code 安装");
    expect(wrapper.text()).not.toContain("此开关只影响 Codex");
    expect(wrapper.text()).not.toContain("此开关只影响 Claude Code");
  });

  it("restores persisted hook failures and exposes retry", async () => {
    const settings = structuredClone(baseSettings);
    settings.hookSource.codex = true;
    settings.hookSource.lastErrors.codex = {
      operation: "uninstall",
      code: "config-write-failed",
      message: "配置不可写",
      occurredAt: new Date("2026-05-30T01:00:00.000Z").toISOString(),
      retryAction: "uninstall",
    };

    const wrapper = mountPanel(settings, [diagnostic("codex", "partial"), diagnostic("claude-code", "unavailable")]);

    expect(wrapper.text()).toContain("卸载失败");
    expect(wrapper.text()).toContain("配置不可写");

    const retryButton = wrapper.findAll("button").find((button) => button.text() === "重试");
    expect(retryButton).toBeTruthy();
    await retryButton!.trigger("click");
    await nextTick();
    await nextTick();

    expect(wrapper.text()).toContain("未接入");
    expect(wrapper.text()).not.toContain("卸载失败");
  });

  it("shows quiet mode and Dock visibility settings", () => {
    const settings = structuredClone(baseSettings);
    settings.quietMode = true;
    settings.showInDock = true;

    const wrapper = mountPanel(settings, [diagnostic("codex", "unavailable"), diagnostic("claude-code", "unavailable")]);

    expect(wrapper.text()).toContain("安静模式");
    expect(wrapper.text()).toContain("显示在 Dock 栏");
    expect(wrapper.findAll("input[type='checkbox']").some((input) => (input.element as HTMLInputElement).checked)).toBe(true);
  });

  it("shows appearance, notification, and auto acknowledgement settings", async () => {
    const settings = structuredClone(baseSettings);
    settings.appearance.islandOpacity = 0.7;
    settings.notifications.enabled = true;
    settings.notifications.sound = "Ping";
    settings.autoAcknowledge.enabled = true;
    settings.autoAcknowledge.delaySeconds = 1800;

    const wrapper = mountPanel(settings, [diagnostic("codex", "unavailable"), diagnostic("claude-code", "unavailable")]);

    expect(wrapper.text()).toContain("悬浮岛透明度");
    expect(wrapper.text()).toContain("70%");
    expect(wrapper.text()).toContain("关键状态通知");
    expect(wrapper.text()).toContain("通知声音");
    expect(wrapper.text()).toContain("测试通知");
    expect(wrapper.text()).toContain("自动确认完成任务");

    const autoAckSelect = wrapper.findAll("select").find((select) => select.text().includes("15 分钟"));
    expect(autoAckSelect).toBeTruthy();
    await autoAckSelect!.setValue("300");
    await nextTick();

    expect(usePreferencesStore().settings.autoAcknowledge.delaySeconds).toBe(300);

    const soundSelect = wrapper.findAll("select").find((select) => select.text().includes("系统默认"));
    expect(soundSelect).toBeTruthy();
    await soundSelect!.setValue("Hero");
    await nextTick();

    expect(usePreferencesStore().settings.notifications.sound).toBe("Hero");

    const sendTestNotification = vi.spyOn(notificationsBridge, "sendTestNotification").mockResolvedValue("sent");
    const testButton = wrapper.findAll("button").find((button) => button.text() === "发送测试通知");
    expect(testButton).toBeTruthy();
    await testButton!.trigger("click");
    await nextTick();

    expect(sendTestNotification).toHaveBeenCalledWith("Hero");
    expect(wrapper.text()).toContain("已发送测试通知");
  });
});
