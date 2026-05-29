<script setup lang="ts">
import { computed } from "vue";
import StatusDot from "@/components/primitives/StatusDot.vue";
import { startWindowDrag } from "@/bridge/tauriApi";
import { sourceLabel, statusLabel } from "@/domain/privacy";
import type { AgentTask } from "@/domain/taskTypes";

const props = defineProps<{
  task?: AgentTask;
  activeCount: number;
  waitingCount: number;
  loading: boolean;
}>();

const emit = defineEmits<{
  expand: [];
}>();

let pointerStart: { x: number; y: number } | undefined;
let didDrag = false;

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

  if (!props.task) {
    return "暂无活跃任务";
  }

  return `${sourceLabel(props.task.source)} ${statusLabel(props.task.status)}`;
});
</script>

<template>
  <button
    class="collapsed-island"
    type="button"
    @click="handleClick"
    @pointerdown="handlePointerDown"
    @pointermove="handlePointerMove"
    @pointerup="handlePointerUp"
    @pointercancel="handlePointerUp"
  >
    <span class="collapsed-island__left">
      <StatusDot v-if="task" :status="task.status" />
      <span v-else class="collapsed-island__idle-dot" />
      <span class="collapsed-island__text">{{ statusText }}</span>
    </span>
    <span class="collapsed-island__meta" :class="{ 'collapsed-island__meta--waiting': waitingCount > 0 }">
      {{ waitingCount > 0 ? `${waitingCount} 待处理` : `${activeCount} 任务` }}
    </span>
  </button>
</template>
