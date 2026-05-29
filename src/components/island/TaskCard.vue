<script setup lang="ts">
import { computed } from "vue";
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
}>();

const taskStore = useTaskStore();
const projectName = computed(() => projectNameFromPath(props.task.cwd));
</script>

<template>
  <button class="task-card" :class="{ 'task-card--selected': selected }" type="button" @click="$emit('select')">
    <span class="task-card__top">
      <span class="task-card__source">
        <StatusDot :status="task.status" />
        {{ sourceLabel(task.source) }}
      </span>
      <span class="task-card__status">{{ statusLabel(task.status) }}</span>
    </span>

    <span class="task-card__title">{{ task.title }}</span>

    <span class="task-card__meta">
      <span>{{ projectName || "未知目录" }}</span>
      <span>{{ task.lastAction || "等待状态更新" }}</span>
      <DurationText :seconds="taskStore.elapsedSeconds(task)" />
    </span>
  </button>
</template>
