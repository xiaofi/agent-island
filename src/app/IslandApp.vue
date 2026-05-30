<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { Bug, ChevronLeft, Settings } from "@lucide/vue";
import IslandCollapsed from "@/components/island/IslandCollapsed.vue";
import IslandExpanded from "@/components/island/IslandExpanded.vue";
import TaskDetail from "@/components/island/TaskDetail.vue";
import IconButton from "@/components/primitives/IconButton.vue";
import { openAppWindow, setWindowMode, startWindowDrag } from "@/bridge/tauriApi";
import { maskTask } from "@/domain/privacy";
import { usePreferencesStore } from "@/stores/preferencesStore";
import { useTaskStore } from "@/stores/taskStore";

type IslandMode = "collapsed" | "list" | "detail";

const taskStore = useTaskStore();
const preferencesStore = usePreferencesStore();
const mode = ref<IslandMode>("collapsed");
const selectedTaskId = ref<string>();

const selectedRawTask = computed(() => taskStore.tasks.find((task) => task.id === selectedTaskId.value));
const selectedVisibleTask = computed(() => {
  const task = selectedRawTask.value;
  return task ? maskTask(task, preferencesStore.privacy) : undefined;
});
const completedAlertTasks = computed(() =>
  taskStore.completedAlertTasks.map((task) => maskTask(task, preferencesStore.privacy)),
);
const collapsedTasks = computed(() => {
  const completedAlertTaskIds = new Set(taskStore.completedAlertTasks.map((task) => task.id));
  const sortedTasks = taskStore.sortedTasks.filter((task) => !completedAlertTaskIds.has(task.id));

  if (completedAlertTaskIds.size) {
    return sortedTasks.map((task) => maskTask(task, preferencesStore.privacy));
  }

  const attentionTasks = taskStore.attentionTasks.filter((task) => !completedAlertTaskIds.has(task.id));
  if (attentionTasks.length) {
    return [maskTask(attentionTasks[0], preferencesStore.privacy)];
  }

  const activeTasks = taskStore.activeTasks.filter((task) => !completedAlertTaskIds.has(task.id));
  const tasks = activeTasks.length ? activeTasks : sortedTasks;
  return tasks.map((task) => maskTask(task, preferencesStore.privacy));
});

const isExpanded = computed(() => mode.value !== "collapsed");
const collapsedHeight = computed(() => {
  const completedRowCount = completedAlertTasks.value.length;
  if (!completedRowCount) {
    return 44;
  }

  return Math.max(44, 12 + completedRowCount * 32);
});

function showList() {
  mode.value = "list";
}

function collapse() {
  mode.value = "collapsed";
  selectedTaskId.value = undefined;
}

function toggleIsland() {
  if (isExpanded.value) {
    collapse();
    return;
  }

  showList();
}

function selectTask(taskId: string) {
  selectedTaskId.value = taskId;
  mode.value = "detail";
}

watch(
  [isExpanded, collapsedHeight],
  ([expanded, height]) => {
    void setWindowMode(expanded, height);
  },
  { immediate: true },
);
</script>

<template>
  <main class="app-shell" :class="{ 'app-shell--expanded': isExpanded }">
    <section class="island-window" :style="{ '--collapsed-height': `${collapsedHeight}px` }">
      <div class="island-trigger">
        <IslandCollapsed
          :completed-tasks="completedAlertTasks"
          :tasks="collapsedTasks"
          :active-count="taskStore.activeTasks.length"
          :waiting-count="taskStore.waitingCount"
          :loading="taskStore.loading"
          @acknowledge-completed="taskStore.acknowledgeCompletedTasks"
          @expand="toggleIsland"
        />
      </div>

      <Transition name="island-drop">
        <div v-if="isExpanded" class="panel panel--island">
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
