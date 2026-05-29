<script setup lang="ts">
import { computed } from "vue";
import { Bug, Settings } from "@lucide/vue";
import DiagnosticsPanel from "@/components/settings/DiagnosticsPanel.vue";
import SettingsPanel from "@/components/settings/SettingsPanel.vue";
import { useTaskStore } from "@/stores/taskStore";

const props = defineProps<{
  kind: "settings" | "diagnostics";
}>();

const taskStore = useTaskStore();

const title = computed(() => (props.kind === "settings" ? "设置" : "诊断"));
const subtitle = computed(() =>
  props.kind === "settings" ? "隐私、窗口行为和 adapter 开关" : "本机 agent 数据源、权限和解析状态",
);
</script>

<template>
  <main class="full-window-shell">
    <header class="full-window-header">
      <div class="full-window-header__icon">
        <Settings v-if="kind === 'settings'" :size="19" />
        <Bug v-else :size="19" />
      </div>
      <div>
        <p>Agent Island</p>
        <h1>{{ title }}</h1>
        <span>{{ subtitle }}</span>
      </div>
    </header>

    <section class="full-window-body">
      <SettingsPanel v-if="kind === 'settings'" />
      <DiagnosticsPanel v-else :diagnostics="taskStore.diagnostics" />
    </section>
  </main>
</template>
