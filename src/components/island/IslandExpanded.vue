<script setup lang="ts">
import TaskCard from "@/components/island/TaskCard.vue";
import type { AgentTask } from "@/domain/taskTypes";

defineProps<{
  tasks: AgentTask[];
  selectedTaskId?: string;
}>();

defineEmits<{
  selectTask: [taskId: string];
}>();
</script>

<template>
  <div class="task-list">
    <TaskCard
      v-for="task in tasks"
      :key="task.id"
      :task="task"
      :selected="task.id === selectedTaskId"
      @select="$emit('selectTask', task.id)"
    />
    <div v-if="tasks.length === 0" class="empty-state">
      <p>没有发现活跃 agent 会话</p>
    </div>
  </div>
</template>
