<script setup lang="ts">
defineProps<{
  label: string;
  active?: boolean;
}>();

defineEmits<{
  click: [];
}>();
</script>

<template>
  <button
    class="icon-button"
    :class="{ 'icon-button--active': active }"
    type="button"
    :title="label"
    @pointerdown.stop
    @click="$emit('click')"
  >
    <slot />
  </button>
</template>
