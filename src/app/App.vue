<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted } from "vue";
import FullWindowApp from "@/app/FullWindowApp.vue";
import IslandApp from "@/app/IslandApp.vue";
import { isRunningInTauri } from "@/bridge/tauriApi";
import { useDurationTicker } from "@/composables/useDurationTicker";
import { useTauriEvents } from "@/composables/useTauriEvents";
import { usePreferencesStore } from "@/stores/preferencesStore";
import { useTaskStore } from "@/stores/taskStore";

const taskStore = useTaskStore();
const preferencesStore = usePreferencesStore();
let taskPollingInterval: number | undefined;

useDurationTicker();
useTauriEvents();

const windowKind = computed(() => {
  const params = new URLSearchParams(window.location.search);
  const value = params.get("window");
  return value === "settings" || value === "diagnostics" ? value : "island";
});

onMounted(async () => {
  await preferencesStore.load();
  await taskStore.load();

  if (isRunningInTauri()) {
    taskPollingInterval = window.setInterval(() => {
      void taskStore.refreshTasks();
    }, 2000);
  }
});

onBeforeUnmount(() => {
  if (taskPollingInterval !== undefined) {
    window.clearInterval(taskPollingInterval);
  }
});
</script>

<template>
  <IslandApp v-if="windowKind === 'island'" />
  <FullWindowApp v-else :kind="windowKind" />
</template>
