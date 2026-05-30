<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { Check } from "@lucide/vue";
import StatusDot from "@/components/primitives/StatusDot.vue";
import { startWindowDrag } from "@/bridge/tauriApi";
import { statusLabel } from "@/domain/privacy";
import type { AgentTask } from "@/domain/taskTypes";

const props = defineProps<{
  completedTasks: AgentTask[];
  tasks: AgentTask[];
  activeCount: number;
  waitingCount: number;
  loading: boolean;
}>();

const emit = defineEmits<{
  acknowledgeCompleted: [taskIds: string[]];
  expand: [];
}>();

let pointerStart: { x: number; y: number } | undefined;
let didDrag = false;
let rotationTimer: number | undefined;
const taskIndex = ref(0);

const currentTask = computed(() => props.tasks[taskIndex.value]);
const hasCompletedTasks = computed(() => props.completedTasks.length > 0);
const hasCompletedOverflow = computed(() => props.completedTasks.length > 5);
const visibleCompletedTasks = computed(() =>
  hasCompletedOverflow.value ? props.completedTasks.slice(0, 4) : props.completedTasks.slice(0, 5),
);
const metaText = computed(() => (props.waitingCount > 0 ? `${props.waitingCount} 待处理` : `${props.activeCount} 任务`));

function handlePointerDown(event: PointerEvent) {
  if (event.button !== 0) {
    return;
  }

  pointerStart = { x: event.clientX, y: event.clientY };
  didDrag = false;
}

async function handlePointerMove(event: PointerEvent) {
  if (!pointerStart || didDrag) {
    return;
  }

  const dx = event.clientX - pointerStart.x;
  const dy = event.clientY - pointerStart.y;

  if (Math.hypot(dx, dy) < 5) {
    return;
  }

  didDrag = true;
  pointerStart = undefined;
  event.preventDefault();
  await startWindowDrag();
}

function handlePointerUp() {
  pointerStart = undefined;
}

function handleClick() {
  if (didDrag) {
    didDrag = false;
    return;
  }

  emit("expand");
}

const statusText = computed(() => {
  if (props.loading) {
    return "正在发现本地 agent";
  }

  if (!currentTask.value) {
    return "暂无活跃任务";
  }

  return taskText(currentTask.value);
});

function acknowledgeCompleted(taskId: string) {
  if (taskId) {
    emit("acknowledgeCompleted", [taskId]);
  }
}

function showMetaForCompletedRow(index: number) {
  return !hasCompletedOverflow.value && visibleCompletedTasks.value.length > 1 && index === visibleCompletedTasks.value.length - 1;
}

function rotateTask() {
  if (props.tasks.length <= 1) {
    taskIndex.value = 0;
    return;
  }

  taskIndex.value = (taskIndex.value + 1) % props.tasks.length;
}

function collapsedSourceLabel(source: AgentTask["source"]) {
  switch (source) {
    case "codex":
      return "codex";
    case "claude-code":
      return "Claude";
    case "manual":
      return "Manual";
  }
}

function compactTaskTitle(title: string, source: string) {
  const escapedSource = source.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  return title.replace(new RegExp(`^${escapedSource}\\s*[·:：-]\\s*`, "i"), "").trim() || title;
}

function taskText(task: AgentTask) {
  const source = collapsedSourceLabel(task.source);
  return `${source} · ${statusLabel(task.status)} · ${compactTaskTitle(task.title, source)}`;
}

watch(
  () => props.tasks.length,
  (length) => {
    if (taskIndex.value >= length) {
      taskIndex.value = 0;
    }
  },
);

onMounted(() => {
  rotationTimer = window.setInterval(rotateTask, 5000);
});

onBeforeUnmount(() => {
  if (rotationTimer !== undefined) {
    window.clearInterval(rotationTimer);
  }
});
</script>

<template>
  <div
    class="collapsed-island"
    :class="{ 'collapsed-island--stacked': hasCompletedTasks }"
    role="button"
    tabindex="0"
    @click="handleClick"
    @keydown.enter.prevent="handleClick"
    @keydown.space.prevent="handleClick"
    @pointerdown="handlePointerDown"
    @pointermove="handlePointerMove"
    @pointerup="handlePointerUp"
    @pointercancel="handlePointerUp"
  >
    <template v-if="hasCompletedTasks">
      <span
        v-for="(task, index) in visibleCompletedTasks"
        :key="task.id"
        class="collapsed-island__row collapsed-island__row--confirmation"
      >
        <span class="collapsed-island__left">
          <StatusDot :status="task.status" />
          <span class="collapsed-island__text">{{ taskText(task) }}</span>
        </span>
        <span class="collapsed-island__actions">
          <span
            v-if="showMetaForCompletedRow(index)"
            class="collapsed-island__meta"
            :class="{ 'collapsed-island__meta--waiting': waitingCount > 0 }"
          >
            {{ metaText }}
          </span>
          <button
            class="collapsed-island__ack"
            type="button"
            :aria-label="`确认已收到 ${task.title} 完成提醒`"
            title="确认完成提醒"
            @click.stop="acknowledgeCompleted(task.id)"
            @pointerdown.stop
          >
            <Check :size="15" />
          </button>
        </span>
      </span>
      <span
        v-if="hasCompletedOverflow"
        class="collapsed-island__row collapsed-island__row--confirmation collapsed-island__row--overflow"
      >
        <span class="collapsed-island__left">
          <span class="collapsed-island__text">请点击展开列表查看</span>
        </span>
        <span class="collapsed-island__meta" :class="{ 'collapsed-island__meta--waiting': waitingCount > 0 }">
          {{ metaText }}
        </span>
      </span>
    </template>

    <span v-else class="collapsed-island__row">
      <Transition name="collapsed-carousel" mode="out-in">
        <span :key="currentTask?.id ?? statusText" class="collapsed-island__left collapsed-island__left--carousel">
          <StatusDot v-if="currentTask" :status="currentTask.status" />
          <span v-else class="collapsed-island__idle-dot" />
          <span class="collapsed-island__text">{{ statusText }}</span>
        </span>
      </Transition>
      <span class="collapsed-island__meta" :class="{ 'collapsed-island__meta--waiting': waitingCount > 0 }">
        {{ metaText }}
      </span>
    </span>
  </div>
</template>
