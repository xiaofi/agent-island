import { onBeforeUnmount, onMounted } from "vue";
import { connectAgentEventBus } from "@/bridge/eventBus";
import { usePreferencesStore } from "@/stores/preferencesStore";
import { useTaskStore } from "@/stores/taskStore";

export function useTauriEvents() {
  const taskStore = useTaskStore();
  const preferencesStore = usePreferencesStore();
  let cleanup: (() => void) | undefined;

  onMounted(async () => {
    cleanup = await connectAgentEventBus({
      onTaskUpdated: taskStore.upsertTask,
      onTaskRemoved: taskStore.removeTask,
      onEventCreated: taskStore.addEvent,
      onDiagnosticUpdated: taskStore.upsertDiagnostic,
      onSettingsUpdated: preferencesStore.replaceSettings,
    });
  });

  onBeforeUnmount(() => {
    cleanup?.();
  });
}
