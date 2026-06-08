<script setup lang="ts">
import { Copy, ExternalLink, FolderOpen, Trash2 } from "@lucide/vue";
import IconButton from "@/components/primitives/IconButton.vue";
import StatusDot from "@/components/primitives/StatusDot.vue";
import { copyTaskSummary, openTask, openWorkdir } from "@/bridge/tauriApi";
import { sourceLabel, statusLabel } from "@/domain/privacy";
import type { AgentTask } from "@/domain/taskTypes";

const props = defineProps<{
  task: AgentTask;
  rawTask?: AgentTask;
}>();

defineEmits<{
  back: [];
  completeAndAcknowledge: [taskId: string];
}>();

const actionTask = () => props.rawTask ?? props.task;
</script>

<template>
  <div class="task-detail">
    <section class="detail-summary">
      <div class="detail-summary__header">
        <span class="task-card__source">
          <StatusDot :status="task.status" />
          {{ sourceLabel(task.source) }}
        </span>
        <span class="task-card__status">{{ statusLabel(task.status) }}</span>
      </div>
      <h2>{{ task.title }}</h2>
      <p v-if="task.cwd" class="detail-summary__path">{{ task.cwd }}</p>
      <p v-if="task.waitingReason" class="detail-summary__notice">{{ task.waitingReason }}</p>
      <p v-if="task.errorSummary" class="detail-summary__error">{{ task.errorSummary }}</p>

      <div class="detail-actions">
        <IconButton label="打开任务" @click="openTask(actionTask().id)">
          <ExternalLink :size="16" />
        </IconButton>
        <IconButton label="打开目录" @click="openWorkdir(actionTask().cwd)">
          <FolderOpen :size="16" />
        </IconButton>
        <IconButton label="复制摘要" @click="copyTaskSummary(actionTask())">
          <Copy :size="16" />
        </IconButton>
        <IconButton label="清除任务状态" @click="$emit('completeAndAcknowledge', actionTask().id)">
          <Trash2 :size="16" />
        </IconButton>
      </div>
    </section>

    <section class="event-list">
      <h3>最近事件</h3>
      <ol>
        <li v-for="event in task.events.slice(0, 10)" :key="event.id">
          <time>{{ new Date(event.timestamp).toLocaleTimeString() }}</time>
          <span>{{ event.summary }}</span>
        </li>
      </ol>
    </section>
  </div>
</template>
