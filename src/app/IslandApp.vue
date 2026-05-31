<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { Bug, ChevronLeft, Settings } from "@lucide/vue";
import IslandCollapsed from "@/components/island/IslandCollapsed.vue";
import IslandExpanded from "@/components/island/IslandExpanded.vue";
import TaskDetail from "@/components/island/TaskDetail.vue";
import IconButton from "@/components/primitives/IconButton.vue";
import { openAppWindow, setWindowMode, startWindowDrag } from "@/bridge/tauriApi";
import { maskTask } from "@/domain/privacy";
import { needsAttention } from "@/domain/taskPriority";
import type { AgentTask } from "@/domain/taskTypes";
import { usePreferencesStore } from "@/stores/preferencesStore";
import { useTaskStore } from "@/stores/taskStore";

type IslandMode = "collapsed" | "list" | "detail";

const taskStore = useTaskStore();
const preferencesStore = usePreferencesStore();
const mode = ref<IslandMode>("collapsed");
const isPanelOpen = ref(false);
const isPanelClosing = ref(false);
const isLayoutExpanded = ref(false);
const selectedTaskId = ref<string>();
let transitionToken = 0;

const selectedRawTask = computed(() => taskStore.tasks.find((task) => task.id === selectedTaskId.value));
const selectedVisibleTask = computed(() => {
  const task = selectedRawTask.value;
  return task ? maskTask(task, preferencesStore.privacy) : undefined;
});
const collapsedAlertTasks = computed(() =>
  taskStore.sortedTasks.filter(needsAttention).map((task) => maskTask(task, preferencesStore.privacy)),
);
const runningSummaryCount = computed(() => taskStore.tasks.filter(isRunningSummaryTask).length);
const hasStackedCollapsedRows = computed(
  () =>
    collapsedAlertTasks.value.length > 1 ||
    (collapsedAlertTasks.value.length === 1 && (runningSummaryCount.value > 0 || taskStore.loading)),
);

const collapsedHeight = computed(() => {
  const visibleAlertRowCount = Math.min(collapsedAlertTasks.value.length, 4);
  if (!hasStackedCollapsedRows.value) {
    return 44;
  }

  return Math.max(44, 12 + (visibleAlertRowCount + 1) * 30);
});

function isRunningSummaryTask(task: AgentTask) {
  return (
    task.status === "discovering" ||
    task.status === "running" ||
    task.status === "thinking" ||
    task.status === "tool-running"
  );
}

async function showList() {
  mode.value = "list";

  if (isLayoutExpanded.value) {
    isPanelOpen.value = true;
    return;
  }

  const token = ++transitionToken;
  isPanelClosing.value = false;
  isLayoutExpanded.value = true;
  await applyWindowMode(true);

  if (token === transitionToken) {
    isPanelOpen.value = true;
  }
}

function collapse() {
  if (isPanelClosing.value) {
    return;
  }

  transitionToken += 1;
  isPanelClosing.value = true;

  if (!isPanelOpen.value) {
    void finishCollapse(transitionToken);
    return;
  }

  isPanelOpen.value = false;
}

function toggleIsland() {
  if (isLayoutExpanded.value) {
    collapse();
    return;
  }

  void showList();
}

function selectTask(taskId: string) {
  selectedTaskId.value = taskId;
  mode.value = "detail";
  isPanelOpen.value = true;
}

async function handlePanelAfterLeave() {
  await finishCollapse(transitionToken);
}

async function finishCollapse(token: number) {
  await applyWindowMode(false);

  if (token !== transitionToken) {
    return;
  }

  selectedTaskId.value = undefined;
  mode.value = "collapsed";
  isLayoutExpanded.value = false;
  isPanelClosing.value = false;
}

watch(
  collapsedHeight,
  () => {
    if (!isLayoutExpanded.value) {
      void applyWindowMode(false);
    }
  },
  { immediate: true },
);

async function applyWindowMode(expanded: boolean) {
  try {
    await setWindowMode(expanded, collapsedHeight.value);
  } catch (error) {
    console.warn("[agent-island] failed to update island window mode", error);
  }
}
</script>

<template>
  <main class="app-shell" :class="{ 'app-shell--expanded': isLayoutExpanded }">
    <section class="island-window" :style="{ '--collapsed-height': `${collapsedHeight}px` }">
      <div class="island-trigger" :class="{ 'island-trigger--stacked': hasStackedCollapsedRows }">
        <IslandCollapsed
          :alert-tasks="collapsedAlertTasks"
          :running-count="runningSummaryCount"
          :loading="taskStore.loading"
          :expanded="isLayoutExpanded"
          @acknowledge-completed="taskStore.acknowledgeCompletedTasks"
          @expand="toggleIsland"
        />
      </div>

      <Transition name="island-drop" @after-leave="handlePanelAfterLeave">
        <div v-if="isPanelOpen" class="panel panel--island">
          <header class="panel__header">
            <div class="panel__title" @pointerdown="startWindowDrag">
              <IconButton v-if="mode === 'detail'" label="返回列表" @click="showList">
                <ChevronLeft :size="16" />
              </IconButton>
              <div>
                <p class="panel__eyebrow">Agent Island</p>
                <h1>{{ mode === "detail" ? "任务详情" : "活跃任务" }}</h1>
              </div>
            </div>

            <div class="panel__actions">
              <IconButton label="打开诊断窗口" @click="openAppWindow('diagnostics')">
                <Bug :size="16" />
              </IconButton>
              <IconButton label="打开设置窗口" @click="openAppWindow('settings')">
                <Settings :size="16" />
              </IconButton>
            </div>
          </header>

          <IslandExpanded
            v-if="mode === 'list'"
            :tasks="taskStore.visibleTasks"
            :selected-task-id="selectedTaskId"
            @select-task="selectTask"
            @acknowledge-completed="taskStore.acknowledgeCompletedTask"
          />
          <TaskDetail
            v-else-if="mode === 'detail' && selectedVisibleTask"
            :task="selectedVisibleTask"
            :raw-task="selectedRawTask"
            @back="showList"
          />
        </div>
      </Transition>
    </section>
  </main>
</template>
