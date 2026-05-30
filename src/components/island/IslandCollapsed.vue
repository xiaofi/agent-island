<script setup lang="ts">
import { computed } from "vue";
import { Check } from "@lucide/vue";
import StatusDot from "@/components/primitives/StatusDot.vue";
import { startWindowDrag } from "@/bridge/tauriApi";
import { statusLabel } from "@/domain/privacy";
import type { AgentTask } from "@/domain/taskTypes";

const props = defineProps<{
  alertTasks: AgentTask[];
  runningCount: number;
  loading: boolean;
  expanded: boolean;
}>();

const emit = defineEmits<{
  acknowledgeCompleted: [taskIds: string[]];
  expand: [];
}>();

let pointerStart: { x: number; y: number } | undefined;
let didDrag = false;

const hasAlertTasks = computed(() => props.alertTasks.length > 0);
const isStacked = computed(() => props.alertTasks.length > 1);
const hasAlertOverflow = computed(() => props.alertTasks.length > 4);
const visibleAlertTasks = computed(() =>
  hasAlertOverflow.value ? props.alertTasks.slice(0, 3) : props.alertTasks.slice(0, 4),
);
const hiddenAlertCount = computed(() => props.alertTasks.length - visibleAlertTasks.value.length);
const primaryAlertTask = computed(() => props.alertTasks[0]);
const runningSummaryText = computed(() => {
  if (props.loading) {
    return "正在发现本地 agent";
  }

  return props.runningCount > 0 ? `${props.runningCount} 个任务进行中` : "暂无任务进行中";
});
const primaryRowText = computed(() => {
  const task = primaryAlertTask.value;
  return task ? taskText(task) : runningSummaryText.value;
});
const summaryDotStatus = computed(() => {
  if (isStacked.value || !primaryAlertTask.value) {
    return props.loading ? "discovering" : "running";
  }

  return primaryAlertTask.value.status;
});
const summaryActionText = computed(() => (props.expanded ? "收起列表" : "显示全部任务"));

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

function acknowledgeCompleted(taskId: string) {
  if (taskId) {
    emit("acknowledgeCompleted", [taskId]);
  }
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
</script>

<template>
  <div class="collapsed-island" :class="{ 'collapsed-island--stacked': isStacked }">
    <template v-if="isStacked">
      <span
        v-for="task in visibleAlertTasks"
        :key="task.id"
        class="collapsed-island__row collapsed-island__row--alert"
      >
        <span class="collapsed-island__left">
          <StatusDot :status="task.status" />
          <span class="collapsed-island__text">{{ taskText(task) }}</span>
        </span>
        <span class="collapsed-island__actions">
          <button
            v-if="task.status === 'completed'"
            class="collapsed-island__ack"
            type="button"
            :aria-label="`确认已收到 ${task.title} 完成提醒`"
            title="确认完成提醒"
            @click.stop="acknowledgeCompleted(task.id)"
            @pointerdown.stop
          >
            <Check :size="13" />
          </button>
        </span>
      </span>
      <span
        v-if="hasAlertOverflow"
        class="collapsed-island__row collapsed-island__row--alert collapsed-island__row--overflow"
      >
        <span class="collapsed-island__left">
          <span class="collapsed-island__text">另有 {{ hiddenAlertCount }} 条任务需要关注</span>
        </span>
      </span>
    </template>

    <span
      class="collapsed-island__row collapsed-island__row--summary"
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
      <span class="collapsed-island__left">
        <StatusDot v-if="primaryAlertTask || runningCount > 0 || loading" :status="summaryDotStatus" />
        <span v-else class="collapsed-island__idle-dot" />
        <span class="collapsed-island__text">{{ isStacked ? runningSummaryText : primaryRowText }}</span>
      </span>
      <span class="collapsed-island__actions">
        <span class="collapsed-island__meta">
          {{ summaryActionText }}
        </span>
        <button
          v-if="!isStacked && primaryAlertTask?.status === 'completed'"
          class="collapsed-island__ack"
          type="button"
          :aria-label="`确认已收到 ${primaryAlertTask.title} 完成提醒`"
          title="确认完成提醒"
          @click.stop="acknowledgeCompleted(primaryAlertTask.id)"
          @pointerdown.stop
        >
          <Check :size="12" />
        </button>
      </span>
    </span>
  </div>
</template>
