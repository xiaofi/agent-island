import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { getTasks, runDiscovery } from "@/bridge/tauriApi";
import { maskTask } from "@/domain/privacy";
import { isActiveTask, needsAttention, pickPrimaryTask, sortTasksByPriority } from "@/domain/taskPriority";
import type { AdapterDiagnostic, AgentEvent, AgentSource, AgentTask } from "@/domain/taskTypes";
import { usePreferencesStore } from "@/stores/preferencesStore";

const quietModeStatuses = new Set<AgentTask["status"]>(["waiting-user", "failed", "completed"]);

export const useTaskStore = defineStore("tasks", () => {
  const tasks = ref<AgentTask[]>([]);
  const diagnostics = ref<AdapterDiagnostic[]>([]);
  const loading = ref(false);
  const now = ref(Date.now());
  const acknowledgedCompletedEventKeys = ref(loadAcknowledgedCompletedEventKeys());

  const preferences = usePreferencesStore();

  const sortedTasks = computed(() => sortTasksByPriority(tasks.value));
  const displayTasks = computed(() =>
    preferences.settings.quietMode ? sortedTasks.value.filter((task) => quietModeStatuses.has(task.status)) : sortedTasks.value,
  );
  const completedAlertTasks = computed(() =>
    displayTasks.value.filter((task) => task.status === "completed" && !isAcknowledgedCompletedTask(task)),
  );
  const completedAlertTask = computed(() => completedAlertTasks.value[0]);
  const attentionTasks = computed(() =>
    displayTasks.value.filter((task) => needsAttention(task) && !isAcknowledgedCompletedTask(task)),
  );
  const activeTasks = computed(() => displayTasks.value.filter(isActiveTask));
  const primaryTask = computed(() => {
    if (attentionTasks.value.length) {
      return attentionTasks.value[0];
    }

    return pickPrimaryTask(activeTasks.value.length ? activeTasks.value : displayTasks.value);
  });
  const waitingCount = computed(() => tasks.value.filter((task) => task.status === "waiting-user").length);

  const visibleTasks = computed(() => displayTasks.value.map((task) => maskTask(task, preferences.privacy)));
  const visiblePrimaryTask = computed(() => {
    const task = primaryTask.value;
    return task ? maskTask(task, preferences.privacy) : undefined;
  });

  async function load() {
    loading.value = true;
    try {
      tasks.value = filterAcknowledgedCompletedTasks(await getTasks());
      diagnostics.value = await runDiscovery();
    } finally {
      loading.value = false;
    }
  }

  async function refreshTasks() {
    tasks.value = filterAcknowledgedCompletedTasks(await getTasks());
  }

  async function refreshDiagnostics(source?: AgentSource) {
    const result = await runDiscovery(source);
    for (const diagnostic of result) {
      upsertDiagnostic(diagnostic);
    }
  }

  function upsertTask(task: AgentTask) {
    if (isAcknowledgedCompletedTask(task)) {
      removeTask(task.id);
      return;
    }

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

  function acknowledgeCompletedTask(taskId: string) {
    acknowledgeCompletedTasks([taskId]);
  }

  function acknowledgeCompletedTasks(taskIds: string[]) {
    const nextTaskIds = new Set(taskIds.filter((taskId) => taskId.length > 0));
    const nextEventKeys = tasks.value
      .filter((task) => nextTaskIds.has(task.id) && task.status === "completed")
      .map(completedAcknowledgementKey);

    if (!nextEventKeys.length) {
      return;
    }

    acknowledgedCompletedEventKeys.value = new Set([...acknowledgedCompletedEventKeys.value, ...nextEventKeys]);
    saveAcknowledgedCompletedEventKeys(acknowledgedCompletedEventKeys.value);
    tasks.value = tasks.value.filter((task) => !nextTaskIds.has(task.id));
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
    displayTasks,
    completedAlertTasks,
    completedAlertTask,
    attentionTasks,
    activeTasks,
    primaryTask,
    waitingCount,
    visibleTasks,
    visiblePrimaryTask,
    load,
    refreshTasks,
    refreshDiagnostics,
    upsertTask,
    removeTask,
    addEvent,
    upsertDiagnostic,
    acknowledgeCompletedTask,
    acknowledgeCompletedTasks,
    bumpClock,
    elapsedSeconds,
  };

  function isAcknowledgedCompletedTask(task: AgentTask) {
    return task.status === "completed" && acknowledgedCompletedEventKeys.value.has(completedAcknowledgementKey(task));
  }

  function filterAcknowledgedCompletedTasks(nextTasks: AgentTask[]) {
    return nextTasks.filter((task) => !isAcknowledgedCompletedTask(task));
  }
});

function completedAcknowledgementKey(task: AgentTask) {
  const completedEvent = task.events.reduce<AgentEvent | undefined>((latest, event) => {
    if (event.type !== "session-completed") {
      return latest;
    }

    if (!latest) {
      return event;
    }

    return new Date(event.timestamp).getTime() > new Date(latest.timestamp).getTime() ? event : latest;
  }, undefined);

  return `${task.id}::${completedEvent?.id ?? "completed"}::${completedEvent?.timestamp ?? task.updatedAt}`;
}

function loadAcknowledgedCompletedEventKeys() {
  try {
    const saved = window.localStorage.getItem("agent-island-acknowledged-completed-events");
    const parsed = saved ? JSON.parse(saved) : [];
    return new Set(typeof parsed === "object" && Array.isArray(parsed) ? parsed.filter((id) => typeof id === "string") : []);
  } catch {
    return new Set<string>();
  }
}

function saveAcknowledgedCompletedEventKeys(eventKeys: Set<string>) {
  window.localStorage.setItem("agent-island-acknowledged-completed-events", JSON.stringify([...eventKeys].slice(-200)));
}
