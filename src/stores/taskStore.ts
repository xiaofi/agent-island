import { defineStore } from "pinia";
import { computed, ref, watch } from "vue";
import { getTasks, runDiscovery } from "@/bridge/tauriApi";
import { maskTask } from "@/domain/privacy";
import { isActiveTask, needsAttention, pickPrimaryTask, sortTasksByPriority } from "@/domain/taskPriority";
import type { AdapterDiagnostic, AgentEvent, AgentEventType, AgentSource, AgentTask } from "@/domain/taskTypes";
import { usePreferencesStore } from "@/stores/preferencesStore";

const quietModeStatuses = new Set<AgentTask["status"]>(["waiting-user", "failed", "completed"]);
const archivedStatuses = new Set<AgentTask["status"]>(["paused"]);

export const useTaskStore = defineStore("tasks", () => {
  const tasks = ref<AgentTask[]>([]);
  const diagnostics = ref<AdapterDiagnostic[]>([]);
  const loading = ref(false);
  const now = ref(Date.now());
  const acknowledgedCompletedEventKeys = ref(loadAcknowledgedCompletedEventKeys());
  const manuallyClearedTaskKeys = ref(loadCompletedEventKeys("agent-island-manually-cleared-task-events"));
  const autoAcknowledgeTimers = new Map<string, number>();

  const preferences = usePreferencesStore();

  const sortedTasks = computed(() => sortTasksByPriority(tasks.value));
  const listableTasks = computed(() => sortedTasks.value.filter((task) => !archivedStatuses.has(task.status)));
  const displayTasks = computed(() =>
    preferences.settings.quietMode ? listableTasks.value.filter((task) => quietModeStatuses.has(task.status)) : listableTasks.value,
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

  watch(
    () => ({
      enabled: preferences.settings.autoAcknowledge.enabled,
      delaySeconds: preferences.settings.autoAcknowledge.delaySeconds,
      completedKeys: completedAlertTasks.value.map(completedAcknowledgementKey).join("|"),
    }),
    scheduleAutoAcknowledgeTimers,
    { immediate: true },
  );

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
    const nextTasks = filterAcknowledgedCompletedTasks(await getTasks());
    tasks.value = nextTasks;
  }

  async function refreshDiagnostics(source?: AgentSource) {
    const result = await runDiscovery(source);
    for (const diagnostic of result) {
      upsertDiagnostic(diagnostic);
    }
  }

  function upsertTask(task: AgentTask) {
    if (isAcknowledgedCompletedTask(task) || isManuallyClearedTask(task)) {
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
    clearAutoAcknowledgeTimerForTask(taskId);
    tasks.value = tasks.value.filter((task) => task.id !== taskId);
  }

  function addEvent(event: AgentEvent) {
    const task = tasks.value.find((item) => item.id === event.taskId);
    if (!task) {
      return;
    }

    // agent-event-created is supplemental; canonical task fields come from agent-task-updated.
    task.events = [event, ...task.events.filter((item) => item.id !== event.id)].slice(0, 10);
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

  function completeAndAcknowledgeTask(taskId: string) {
    const task = tasks.value.find((item) => item.id === taskId);
    if (!task) {
      return;
    }

    if (task.status === "completed") {
      acknowledgeCompletedTask(taskId);
      return;
    }

    const taskKey = manualClearanceKey(task);
    manuallyClearedTaskKeys.value = new Set([...manuallyClearedTaskKeys.value, taskKey]);
    saveCompletedEventKeys("agent-island-manually-cleared-task-events", manuallyClearedTaskKeys.value);
    removeTask(taskId);
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
    for (const eventKey of nextEventKeys) {
      clearAutoAcknowledgeTimer(eventKey);
    }
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
    listableTasks,
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
    completeAndAcknowledgeTask,
    acknowledgeCompletedTasks,
    bumpClock,
    elapsedSeconds,
  };

  function isAcknowledgedCompletedTask(task: AgentTask) {
    return task.status === "completed" && acknowledgedCompletedEventKeys.value.has(completedAcknowledgementKey(task));
  }

  function isManuallyClearedTask(task: AgentTask) {
    return manuallyClearedTaskKeys.value.has(manualClearanceKey(task));
  }

  function filterAcknowledgedCompletedTasks(nextTasks: AgentTask[]) {
    return nextTasks.filter((task) => !isAcknowledgedCompletedTask(task) && !isManuallyClearedTask(task));
  }

  function scheduleAutoAcknowledgeTimers() {
    clearAutoAcknowledgeTimers();

    if (!preferences.settings.autoAcknowledge.enabled) {
      return;
    }

    for (const task of completedAlertTasks.value) {
      const eventKey = completedAcknowledgementKey(task);
      const dueAt = completedAt(task) + preferences.settings.autoAcknowledge.delaySeconds * 1000;
      const remainingMs = dueAt - Date.now();

      if (remainingMs <= 0) {
        acknowledgeCompletedTask(task.id);
        continue;
      }

      const timer = window.setTimeout(() => {
        const currentTask = tasks.value.find((item) => item.id === task.id);
        if (
          currentTask?.status === "completed" &&
          completedAcknowledgementKey(currentTask) === eventKey &&
          preferences.settings.autoAcknowledge.enabled &&
          !isAcknowledgedCompletedTask(currentTask)
        ) {
          acknowledgeCompletedTask(currentTask.id);
        }
      }, remainingMs);

      autoAcknowledgeTimers.set(eventKey, timer);
    }
  }

  function clearAutoAcknowledgeTimerForTask(taskId: string) {
    const task = tasks.value.find((item) => item.id === taskId);
    if (task?.status === "completed") {
      clearAutoAcknowledgeTimer(completedAcknowledgementKey(task));
    }
  }

  function clearAutoAcknowledgeTimer(eventKey: string) {
    const timer = autoAcknowledgeTimers.get(eventKey);
    if (timer === undefined) {
      return;
    }

    window.clearTimeout(timer);
    autoAcknowledgeTimers.delete(eventKey);
  }

  function clearAutoAcknowledgeTimers() {
    for (const timer of autoAcknowledgeTimers.values()) {
      window.clearTimeout(timer);
    }
    autoAcknowledgeTimers.clear();
  }
});

function completedAcknowledgementKey(task: AgentTask) {
  const completedEvent = latestCompletedEvent(task);

  return `${task.id}::${completedEvent?.id ?? "completed"}::${completedEvent?.timestamp ?? task.updatedAt}`;
}

function completedAt(task: AgentTask) {
  const completedEvent = latestCompletedEvent(task);
  return new Date(completedEvent?.timestamp ?? task.updatedAt).getTime();
}

function manualClearanceKey(task: AgentTask) {
  return `${task.id}::${task.updatedAt}`;
}

function latestCompletedEvent(task: AgentTask) {
  return latestEventOfType(task, "session-completed");
}

function latestEventOfType(task: AgentTask, type?: AgentEventType) {
  if (!type) {
    return undefined;
  }

  return task.events.reduce<AgentEvent | undefined>((latest, event) => {
    if (event.type !== type) {
      return latest;
    }

    if (!latest) {
      return event;
    }

    return new Date(event.timestamp).getTime() > new Date(latest.timestamp).getTime() ? event : latest;
  }, undefined);
}

function loadAcknowledgedCompletedEventKeys() {
  return loadCompletedEventKeys("agent-island-acknowledged-completed-events");
}

function loadCompletedEventKeys(storageKey: string) {
  try {
    const saved = window.localStorage.getItem(storageKey);
    const parsed = saved ? JSON.parse(saved) : [];
    return new Set(typeof parsed === "object" && Array.isArray(parsed) ? parsed.filter((id) => typeof id === "string") : []);
  } catch {
    return new Set<string>();
  }
}

function saveAcknowledgedCompletedEventKeys(eventKeys: Set<string>) {
  saveCompletedEventKeys("agent-island-acknowledged-completed-events", eventKeys);
}

function saveCompletedEventKeys(storageKey: string, eventKeys: Set<string>) {
  window.localStorage.setItem(storageKey, JSON.stringify([...eventKeys].slice(-200)));
}
