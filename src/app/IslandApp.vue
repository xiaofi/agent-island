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

const isExpanded = computed(() => mode.value !== "collapsed");

function showList() {
  mode.value = "list";
}

function collapse() {
  mode.value = "collapsed";
  selectedTaskId.value = undefined;
}

function selectTask(taskId: string) {
  selectedTaskId.value = taskId;
  mode.value = "detail";
}

watch(isExpanded, (expanded) => {
  void setWindowMode(expanded);
});
</script>

<template>
  <main class="app-shell" :class="{ 'app-shell--expanded': isExpanded }">
    <section class="island-window" data-tauri-drag-region>
      <IslandCollapsed
        v-if="mode === 'collapsed'"
        :task="taskStore.visiblePrimaryTask"
        :active-count="taskStore.activeTasks.length"
        :waiting-count="taskStore.waitingCount"
        :loading="taskStore.loading"
        @expand="showList"
      />

      <div v-else class="panel panel--island">
        <header class="panel__header" data-tauri-drag-region @pointerdown="startWindowDrag">
          <div class="panel__title">
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
            <button class="panel__close" type="button" @pointerdown.stop @click="collapse">收起</button>
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
    </section>
  </main>
</template>
