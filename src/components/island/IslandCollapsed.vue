<script setup lang="ts">
import { computed } from "vue";
import { Check } from "@lucide/vue";
import StatusDot from "@/components/primitives/StatusDot.vue";
import { saveIslandWindowPosition, startWindowDrag } from "@/bridge/tauriApi";
import { statusLabel } from "@/domain/privacy";
import type { AgentTask } from "@/domain/taskTypes";

const props = withDefaults(defineProps<{
  alertTasks: AgentTask[];
  runningCount: number;
  loading: boolean;
  expanded: boolean;
  busy?: boolean;
  showSummary?: boolean;
  emptyText?: string;
}>(), {
  busy: false,
  showSummary: true,
  emptyText: "暂无任务进行中",
});

const emit = defineEmits<{
  acknowledgeCompleted: [taskIds: string[]];
  expand: [];
}>();

let pointerStart: { x: number; y: number } | undefined;
let didDrag = false;

const hasAlertTasks = computed(() => props.alertTasks.length > 0);
const hasRunningSummary = computed(() => props.runningCount > 0 || props.loading);
const isStacked = computed(
  () => props.alertTasks.length > 1 || (props.alertTasks.length === 1 && hasRunningSummary.value),
);
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

  return props.runningCount > 0 ? `${props.runningCount} 个任务进行中` : props.emptyText;
});
const stackedSummaryText = computed(() => {
  if (hasRunningSummary.value) {
    return runningSummaryText.value;
  }

  return `${props.alertTasks.length} 条任务需要关注`;
});
const primaryRowText = computed(() => {
  const task = primaryAlertTask.value;
  return task ? taskText(task) : runningSummaryText.value;
});
const summaryDotStatus = computed(() => {
  if (isStacked.value && !hasRunningSummary.value && primaryAlertTask.value) {
    return primaryAlertTask.value.status;
  }

  if (isStacked.value || !primaryAlertTask.value) {
    return props.loading ? "discovering" : "running";
  }

  return primaryAlertTask.value.status;
});
const summaryActionText = computed(() => (props.expanded ? "收起全部任务" : "显示全部任务"));

function handlePointerDown(event: PointerEvent) {
  if (event.button !== 0) {
    return;
  }

  (event.currentTarget as HTMLElement).setPointerCapture?.(event.pointerId);
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
  try {
    await saveIslandWindowPosition();
  } catch (error) {
    console.warn("[agent-island] failed to save island window position", error);
  }
}

function handlePointerUp() {
  pointerStart = undefined;
}

function handleClick() {
  if (props.busy) {
    return;
  }

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
  const title = compactTaskTitle(task.title, source);
  return title ? `${source} · ${statusLabel(task.status)} · ${title}` : `${source} · ${statusLabel(task.status)}`;
}
</script>

<template>
  <div
    class="collapsed-island"
    :class="{ 'collapsed-island--stacked': isStacked }"
    @pointerdown="handlePointerDown"
    @pointermove="handlePointerMove"
    @pointerup="handlePointerUp"
    @pointercancel="handlePointerUp"
  >
    <template v-if="isStacked">
      <span
        v-for="task in visibleAlertTasks"
        :key="task.id"
        class="collapsed-island__row collapsed-island__row--alert"
      >
        <span class="collapsed-island__left" data-tauri-drag-region>
          <StatusDot :status="task.status" />
          <span class="collapsed-island__text" data-tauri-drag-region>{{ taskText(task) }}</span>
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
        <span class="collapsed-island__left" data-tauri-drag-region>
          <span class="collapsed-island__text" data-tauri-drag-region>另有 {{ hiddenAlertCount }} 条任务需要关注</span>
        </span>
      </span>
    </template>

    <span
      v-if="showSummary"
      class="collapsed-island__row collapsed-island__row--summary"
      role="button"
      tabindex="0"
      @click="handleClick"
      @keydown.enter.prevent="handleClick"
      @keydown.space.prevent="handleClick"
    >
      <span class="collapsed-island__left" data-tauri-drag-region>
        <StatusDot v-if="primaryAlertTask || runningCount > 0 || loading" :status="summaryDotStatus" />
        <span v-else class="collapsed-island__idle-dot" />
        <span class="collapsed-island__text" data-tauri-drag-region>{{ isStacked ? stackedSummaryText : primaryRowText }}</span>
      </span>
      <span class="collapsed-island__actions">
        <button
          class="collapsed-island__meta collapsed-island__meta-button"
          type="button"
          :disabled="busy"
          @click.stop="handleClick"
          @pointerdown.stop
        >
          {{ summaryActionText }}
        </button>
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
