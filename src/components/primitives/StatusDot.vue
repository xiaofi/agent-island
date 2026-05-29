<script setup lang="ts">
import type { TaskStatus } from "@/domain/taskTypes";

defineProps<{
  status: TaskStatus;
}>();
</script>

<template>
  <span class="status-dot" :class="`status-dot--${status}`" aria-hidden="true" />
</template>
