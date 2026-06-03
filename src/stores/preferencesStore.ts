import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type { AgentSource, AppSettings, HookOperation } from "@/domain/taskTypes";
import {
  getSettings,
  retryHookSourceOperation,
  runHookSelfTest,
  setHookSourceEnabled,
  setMousePassthrough,
  updateSettings,
} from "@/bridge/tauriApi";

export const usePreferencesStore = defineStore("preferences", () => {
  const settings = ref<AppSettings>({
    privacy: {
      hideProjectPath: false,
      hideTaskTitle: false,
      compactOnly: false,
    },
    quietMode: false,
    mousePassthrough: false,
    enabledAdapters: ["manual", "codex", "claude-code"],
    hookSource: {
      codex: false,
      claudeCode: false,
      lastErrors: {},
    },
    islandWindow: {},
  });
  const loaded = ref(false);

  const privacy = computed(() => settings.value.privacy);

  async function load() {
    settings.value = await getSettings();
    loaded.value = true;
    try {
      await setMousePassthrough(settings.value.mousePassthrough);
    } catch (error) {
      console.warn("[agent-island] failed to apply mouse passthrough preference", error);
    }
  }

  async function patch(patchValue: Partial<AppSettings>) {
    settings.value = await updateSettings(patchValue);
  }

  async function setPrivacy<K extends keyof AppSettings["privacy"]>(key: K, value: AppSettings["privacy"][K]) {
    await patch({
      privacy: {
        ...settings.value.privacy,
        [key]: value,
      },
    });
  }

  async function setMousePassthroughPreference(enabled: boolean) {
    settings.value = await updateSettings({ mousePassthrough: enabled });
    await setMousePassthrough(enabled);
  }

  async function setQuietMode(enabled: boolean) {
    settings.value = await updateSettings({ quietMode: enabled });
  }

  async function setHookSource(source: Extract<AgentSource, "codex" | "claude-code">, enabled: boolean) {
    settings.value = await setHookSourceEnabled(source, enabled);
  }

  async function retryHookSource(source: Extract<AgentSource, "codex" | "claude-code">, operation: HookOperation) {
    settings.value = await retryHookSourceOperation(source, operation);
  }

  async function selfTestHookSource(source: Extract<AgentSource, "codex" | "claude-code">) {
    settings.value = await runHookSelfTest(source);
  }

  function replaceSettings(next: AppSettings) {
    settings.value = next;
    loaded.value = true;
  }

  return {
    settings,
    privacy,
    loaded,
    load,
    patch,
    setPrivacy,
    setQuietMode,
    setMousePassthroughPreference,
    setHookSource,
    retryHookSource,
    selfTestHookSource,
    replaceSettings,
  };
});
