<script setup lang="ts">
import { computed, onMounted } from "vue";
import FullWindowApp from "@/app/FullWindowApp.vue";
import IslandApp from "@/app/IslandApp.vue";
import { useDurationTicker } from "@/composables/useDurationTicker";
import { useTauriEvents } from "@/composables/useTauriEvents";
import { usePreferencesStore } from "@/stores/preferencesStore";
import { useTaskStore } from "@/stores/taskStore";

const taskStore = useTaskStore();
const preferencesStore = usePreferencesStore();

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
});
</script>

<template>
  <IslandApp v-if="windowKind === 'island'" />
  <FullWindowApp v-else :kind="windowKind" />
</template>
