import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type { AppSettings } from "@/domain/taskTypes";
import { getSettings, setMousePassthrough, updateSettings } from "@/bridge/tauriApi";

export const usePreferencesStore = defineStore("preferences", () => {
  const settings = ref<AppSettings>({
    privacy: {
      hideProjectPath: false,
      hideTaskTitle: false,
      compactOnly: false,
    },
    mousePassthrough: false,
    enabledAdapters: ["manual", "codex", "claude-code"],
  });
  const loaded = ref(false);

  const privacy = computed(() => settings.value.privacy);

  async function load() {
    settings.value = await getSettings();
    loaded.value = true;
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
    setMousePassthroughPreference,
    replaceSettings,
  };
});
