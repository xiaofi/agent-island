<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  seconds: number;
}>();

const text = computed(() => {
  const safeSeconds = Math.max(0, props.seconds);
  const hours = Math.floor(safeSeconds / 3600);
  const minutes = Math.floor((safeSeconds % 3600) / 60);
  const seconds = safeSeconds % 60;

  if (hours > 0) {
    return `${hours}:${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
  }

  return `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
});
</script>

<template>
  <time class="duration-text">{{ text }}</time>
</template>
