<script setup lang="ts">
import { computed } from "vue";
import { Check } from "@lucide/vue";
import DurationText from "@/components/primitives/DurationText.vue";
import StatusDot from "@/components/primitives/StatusDot.vue";
import { projectNameFromPath, sourceLabel, statusLabel } from "@/domain/privacy";
import type { AgentTask } from "@/domain/taskTypes";
import { useTaskStore } from "@/stores/taskStore";

const props = defineProps<{
  task: AgentTask;
  selected?: boolean;
}>();

defineEmits<{
  select: [];
  acknowledgeCompleted: [];
}>();

const taskStore = useTaskStore();
const projectName = computed(() => projectNameFromPath(props.task.cwd));
const isCompleted = computed(() => props.task.status === "completed");
</script>

<template>
  <article
    class="task-card"
    :class="{ 'task-card--selected': selected }"
    role="button"
    tabindex="0"
    @click="$emit('select')"
    @keydown.enter.prevent="$emit('select')"
    @keydown.space.prevent="$emit('select')"
  >
    <span class="task-card__top">
      <span class="task-card__source">
        <StatusDot :status="task.status" />
        {{ sourceLabel(task.source) }}
      </span>
      <button
        v-if="isCompleted"
        class="task-card__ack"
        type="button"
        :aria-label="`确认已收到 ${task.title} 完成提醒`"
        title="确认完成提醒"
        @click.stop="$emit('acknowledgeCompleted')"
        @keydown.enter.stop
        @keydown.space.stop
      >
        <Check :size="15" />
      </button>
    </span>

    <span class="task-card__title">
      <span class="task-card__state-title">{{ statusLabel(task.status) }}</span>
      <span v-if="task.title" class="task-card__conversation-title">{{ task.title }}</span>
    </span>

    <span class="task-card__meta">
      <span>{{ projectName || "未知目录" }}</span>
      <span>{{ task.lastAction || "等待状态更新" }}</span>
      <DurationText :seconds="taskStore.elapsedSeconds(task)" />
    </span>
  </article>
</template>
