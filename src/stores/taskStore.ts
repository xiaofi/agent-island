import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { getTasks, runDiscovery } from "@/bridge/tauriApi";
import { maskTask } from "@/domain/privacy";
import { isActiveTask, pickPrimaryTask, sortTasksByPriority } from "@/domain/taskPriority";
import type { AdapterDiagnostic, AgentEvent, AgentSource, AgentTask } from "@/domain/taskTypes";
import { usePreferencesStore } from "@/stores/preferencesStore";

export const useTaskStore = defineStore("tasks", () => {
  const tasks = ref<AgentTask[]>([]);
  const diagnostics = ref<AdapterDiagnostic[]>([]);
  const loading = ref(false);
  const now = ref(Date.now());

  const preferences = usePreferencesStore();

  const sortedTasks = computed(() => sortTasksByPriority(tasks.value));
  const activeTasks = computed(() => sortedTasks.value.filter(isActiveTask));
  const primaryTask = computed(() => pickPrimaryTask(activeTasks.value.length ? activeTasks.value : sortedTasks.value));
  const waitingCount = computed(() => tasks.value.filter((task) => task.status === "waiting-user").length);

  const visibleTasks = computed(() => sortedTasks.value.map((task) => maskTask(task, preferences.privacy)));
  const visiblePrimaryTask = computed(() => {
    const task = primaryTask.value;
    return task ? maskTask(task, preferences.privacy) : undefined;
  });

  async function load() {
    loading.value = true;
    try {
      tasks.value = await getTasks();
      diagnostics.value = await runDiscovery();
    } finally {
      loading.value = false;
    }
  }

  async function refreshDiagnostics(source?: AgentSource) {
    const result = await runDiscovery(source);
    for (const diagnostic of result) {
      upsertDiagnostic(diagnostic);
    }
  }

  function upsertTask(task: AgentTask) {
    const index = tasks.value.findIndex((item) => item.id === task.id);
    if (index >= 0) {
      tasks.value[index] = task;
      return;
    }

    tasks.value.push(task);
  }

  function removeTask(taskId: string) {
    tasks.value = tasks.value.filter((task) => task.id !== taskId);
  }

  function addEvent(event: AgentEvent) {
    const task = tasks.value.find((item) => item.id === event.taskId);
    if (!task) {
      return;
    }

    task.events = [event, ...task.events.filter((item) => item.id !== event.id)].slice(0, 10);
    task.updatedAt = event.timestamp;
    task.lastAction = event.summary;
  }

  function upsertDiagnostic(diagnostic: AdapterDiagnostic) {
    const index = diagnostics.value.findIndex((item) => item.source === diagnostic.source);
    if (index >= 0) {
      diagnostics.value[index] = diagnostic;
      return;
    }

    diagnostics.value.push(diagnostic);
  }

  function bumpClock() {
    now.value = Date.now();
  }

  function elapsedSeconds(task: AgentTask) {
    const start = task.startedAt ? new Date(task.startedAt).getTime() : new Date(task.updatedAt).getTime();
    return Math.max(0, Math.floor((now.value - start) / 1000));
  }

  return {
    tasks,
    diagnostics,
    loading,
    now,
    sortedTasks,
    activeTasks,
    primaryTask,
    waitingCount,
    visibleTasks,
    visiblePrimaryTask,
    load,
    refreshDiagnostics,
    upsertTask,
    removeTask,
    addEvent,
    upsertDiagnostic,
    bumpClock,
    elapsedSeconds,
  };
});
