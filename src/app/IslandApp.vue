<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch, type CSSProperties } from "vue";
import { Bug, ChevronLeft, Power, Settings } from "@lucide/vue";
import IslandCollapsed from "@/components/island/IslandCollapsed.vue";
import IslandExpanded from "@/components/island/IslandExpanded.vue";
import TaskDetail from "@/components/island/TaskDetail.vue";
import IconButton from "@/components/primitives/IconButton.vue";
import {
  openAppWindow,
  quitApp,
  saveIslandWindowPosition,
  setWindowMode,
  startWindowDrag,
  subscribeWindowFocusChanged,
  type IslandPanelDirection,
} from "@/bridge/tauriApi";
import { needsAttention } from "@/domain/taskPriority";
import type { AgentTask } from "@/domain/taskTypes";
import { usePreferencesStore } from "@/stores/preferencesStore";
import { useTaskStore } from "@/stores/taskStore";

type IslandMode = "collapsed" | "list" | "detail";
const PANEL_TRANSITION_GUARD_MS = 260;
const COLLAPSE_FALLBACK_DELAY_MS = 900;

const taskStore = useTaskStore();
const preferencesStore = usePreferencesStore();
const mode = ref<IslandMode>("collapsed");
const isPanelOpen = ref(false);
const isPanelOpening = ref(false);
const isPanelClosing = ref(false);
const isLayoutExpanded = ref(false);
const isWindowModePending = ref(false);
const panelDirection = ref<IslandPanelDirection>("down");
const selectedTaskId = ref<string>();
let transitionToken = 0;
let panelOpenFallbackTimer: number | undefined;
let collapseFallbackTimer: number | undefined;
let pendingFocusCollapse = false;
let unsubscribeWindowFocusChanged: (() => void) | undefined;
let isUnmounted = false;

const selectedRawTask = computed(() => taskStore.tasks.find((task) => task.id === selectedTaskId.value));
const selectedVisibleTask = computed(() => selectedRawTask.value);
const collapsedAlertTasks = computed(() => taskStore.displayTasks.filter(needsAttention));
const islandStyle = computed<CSSProperties>(() => ({
  "--collapsed-height": `${collapsedHeight.value}px`,
  "--island-opacity": String(preferencesStore.settings.appearance.islandOpacity),
}));
const runningSummaryCount = computed(() =>
  preferencesStore.settings.quietMode ? 0 : taskStore.tasks.filter(isRunningSummaryTask).length,
);
const showLoadingSummary = computed(() => !preferencesStore.settings.quietMode && taskStore.loading);
const hasCollapsedAlertOverflow = computed(() => collapsedAlertTasks.value.length > 4);
const visibleCollapsedAlertRowCount = computed(() =>
  hasCollapsedAlertOverflow.value ? 3 : Math.min(collapsedAlertTasks.value.length, 4),
);
const hasStackedCollapsedRows = computed(
  () =>
    collapsedAlertTasks.value.length > 1 ||
    (collapsedAlertTasks.value.length === 1 && (runningSummaryCount.value > 0 || showLoadingSummary.value)),
);
const showCollapsedSummaryRow = computed(
  () =>
    !hasStackedCollapsedRows.value ||
    collapsedAlertTasks.value.length > 1 ||
    runningSummaryCount.value > 0 ||
    showLoadingSummary.value ||
    isLayoutExpanded.value ||
    hasCollapsedAlertOverflow.value,
);
const isIslandTransitioning = computed(
  () => isWindowModePending.value || isPanelOpening.value || isPanelClosing.value,
);

const collapsedHeight = computed(() => {
  if (!hasStackedCollapsedRows.value) {
    return 44;
  }

  const overflowRowCount = hasCollapsedAlertOverflow.value ? 1 : 0;
  const summaryRowCount = showCollapsedSummaryRow.value ? 1 : 0;
  return Math.max(44, 12 + (visibleCollapsedAlertRowCount.value + overflowRowCount + summaryRowCount) * 30);
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
  if (isIslandTransitioning.value) {
    return;
  }

  mode.value = "list";

  if (isLayoutExpanded.value) {
    isPanelOpen.value = true;
    return;
  }

  const token = ++transitionToken;
  isPanelClosing.value = false;
  isWindowModePending.value = true;
  const nextPanelDirection = await applyWindowMode(true);

  if (token === transitionToken) {
    panelDirection.value = nextPanelDirection;
    isLayoutExpanded.value = true;
    isPanelOpen.value = true;
    isPanelOpening.value = true;
    isWindowModePending.value = false;
    schedulePanelOpenFallback(token);
  }
}

function collapse() {
  if (isIslandTransitioning.value) {
    return;
  }

  transitionToken += 1;
  isPanelClosing.value = true;

  if (!isPanelOpen.value) {
    void finishCollapse(transitionToken);
    return;
  }

  isPanelOpen.value = false;
  scheduleCollapseFallback(transitionToken);
}

function requestFocusCollapse() {
  if (!isLayoutExpanded.value && !isPanelOpen.value) {
    return;
  }

  pendingFocusCollapse = true;

  if (!isIslandTransitioning.value) {
    collapse();
  }
}

function toggleIsland() {
  if (isLayoutExpanded.value) {
    collapse();
    return;
  }

  void showList();
}

function selectTask(taskId: string) {
  if (isIslandTransitioning.value) {
    return;
  }

  selectedTaskId.value = taskId;
  mode.value = "detail";
  isPanelOpen.value = true;
}

function completeAndAcknowledgeTask(taskId: string) {
  if (isIslandTransitioning.value) {
    return;
  }

  taskStore.completeAndAcknowledgeTask(taskId);
  selectedTaskId.value = undefined;
  mode.value = "list";
}

function handlePanelAfterEnter() {
  finishPanelOpen(transitionToken);
}

async function handlePanelAfterLeave() {
  await finishCollapse(transitionToken);
}

function schedulePanelOpenFallback(token: number) {
  clearPanelOpenFallback();
  panelOpenFallbackTimer = window.setTimeout(() => {
    finishPanelOpen(token);
  }, PANEL_TRANSITION_GUARD_MS);
}

function clearPanelOpenFallback() {
  if (panelOpenFallbackTimer === undefined) {
    return;
  }

  window.clearTimeout(panelOpenFallbackTimer);
  panelOpenFallbackTimer = undefined;
}

function finishPanelOpen(token: number) {
  if (token !== transitionToken || !isPanelOpening.value) {
    return;
  }

  clearPanelOpenFallback();
  isPanelOpening.value = false;

  if (pendingFocusCollapse) {
    collapse();
  }
}

function scheduleCollapseFallback(token: number) {
  clearCollapseFallback();
  collapseFallbackTimer = window.setTimeout(() => {
    void finishCollapse(token);
  }, COLLAPSE_FALLBACK_DELAY_MS);
}

function clearCollapseFallback() {
  if (collapseFallbackTimer === undefined) {
    return;
  }

  window.clearTimeout(collapseFallbackTimer);
  collapseFallbackTimer = undefined;
}

async function finishCollapse(token: number) {
  if (token !== transitionToken || !isPanelClosing.value) {
    return;
  }

  clearCollapseFallback();
  isWindowModePending.value = true;
  await applyWindowMode(false);

  if (token !== transitionToken) {
    return;
  }

  selectedTaskId.value = undefined;
  mode.value = "collapsed";
  isLayoutExpanded.value = false;
  panelDirection.value = "down";
  isPanelOpening.value = false;
  isPanelClosing.value = false;
  isWindowModePending.value = false;
  pendingFocusCollapse = false;
  clearPanelOpenFallback();
}

watch(
  collapsedHeight,
  () => {
    if (!isLayoutExpanded.value && !isWindowModePending.value) {
      void applyWindowMode(false);
    }
  },
  { immediate: true },
);

async function applyWindowMode(expanded: boolean): Promise<IslandPanelDirection> {
  try {
    return await setWindowMode(expanded, collapsedHeight.value, panelDirection.value);
  } catch (error) {
    console.warn("[agent-island] failed to update island window mode", error);
    return "down";
  }
}

async function handleWindowDrag() {
  await startWindowDrag();
  try {
    await saveIslandWindowPosition();
  } catch (error) {
    console.warn("[agent-island] failed to save island window position", error);
  }
}

async function handleQuitApp() {
  try {
    await quitApp();
  } catch (error) {
    console.warn("[agent-island] failed to quit app", error);
  }
}

onMounted(() => {
  isUnmounted = false;

  void subscribeWindowFocusChanged((focused) => {
    if (focused) {
      pendingFocusCollapse = false;
      return;
    }

    requestFocusCollapse();
  }).then((unsubscribe) => {
    if (isUnmounted) {
      unsubscribe();
      return;
    }

    unsubscribeWindowFocusChanged = unsubscribe;
  });
});

onBeforeUnmount(() => {
  isUnmounted = true;
  unsubscribeWindowFocusChanged?.();
  clearPanelOpenFallback();
  clearCollapseFallback();
});
</script>

<template>
  <main
    class="app-shell"
    :class="{
      'app-shell--expanded': isLayoutExpanded,
      'app-shell--expand-up': isLayoutExpanded && panelDirection === 'up',
    }"
  >
    <section class="island-window" :style="islandStyle">
      <div class="island-trigger" :class="{ 'island-trigger--stacked': hasStackedCollapsedRows }">
        <IslandCollapsed
          :alert-tasks="collapsedAlertTasks"
          :running-count="runningSummaryCount"
          :loading="showLoadingSummary"
          :expanded="isLayoutExpanded"
          :busy="isIslandTransitioning"
          :show-summary="showCollapsedSummaryRow"
          :empty-text="preferencesStore.settings.quietMode ? '暂无需关注任务' : '暂无任务进行中'"
          @acknowledge-completed="taskStore.acknowledgeCompletedTasks"
          @expand="toggleIsland"
        />
      </div>

      <Transition name="island-drop" @after-enter="handlePanelAfterEnter" @after-leave="handlePanelAfterLeave">
        <div v-if="isPanelOpen" class="panel panel--island">
          <header class="panel__header">
            <div class="panel__title" @pointerdown="handleWindowDrag">
              <IconButton v-if="mode === 'detail'" label="返回列表" @click="showList">
                <ChevronLeft :size="16" />
              </IconButton>
              <div data-tauri-drag-region>
                <p class="panel__eyebrow" data-tauri-drag-region>Agent Island</p>
                <h1 data-tauri-drag-region>{{ mode === "detail" ? "任务详情" : "活跃任务" }}</h1>
              </div>
            </div>

            <div class="panel__actions">
              <IconButton label="打开诊断窗口" @click="openAppWindow('diagnostics')">
                <Bug :size="16" />
              </IconButton>
              <IconButton label="打开设置窗口" @click="openAppWindow('settings')">
                <Settings :size="16" />
              </IconButton>
              <IconButton label="退出 Agent Island" @click="handleQuitApp">
                <Power :size="16" />
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
            @complete-and-acknowledge="completeAndAcknowledgeTask"
            @back="showList"
          />
        </div>
      </Transition>
    </section>
  </main>
</template>
