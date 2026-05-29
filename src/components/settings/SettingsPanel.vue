<script setup lang="ts">
import { storeToRefs } from "pinia";
import { usePreferencesStore } from "@/stores/preferencesStore";

const preferencesStore = usePreferencesStore();
const { settings } = storeToRefs(preferencesStore);
</script>

<template>
  <div class="settings-panel">
    <label class="toggle-row">
      <span>
        <strong>隐藏项目路径</strong>
        <small>只显示项目目录名，避免暴露完整本机路径。</small>
      </span>
      <input
        type="checkbox"
        :checked="settings.privacy.hideProjectPath"
        @change="preferencesStore.setPrivacy('hideProjectPath', ($event.target as HTMLInputElement).checked)"
      />
    </label>

    <label class="toggle-row">
      <span>
        <strong>隐藏任务标题</strong>
        <small>用来源工具替代任务标题。</small>
      </span>
      <input
        type="checkbox"
        :checked="settings.privacy.hideTaskTitle"
        @change="preferencesStore.setPrivacy('hideTaskTitle', ($event.target as HTMLInputElement).checked)"
      />
    </label>

    <label class="toggle-row">
      <span>
        <strong>压缩态隐私模式</strong>
        <small>压缩态只保留来源、状态和任务数量。</small>
      </span>
      <input
        type="checkbox"
        :checked="settings.privacy.compactOnly"
        @change="preferencesStore.setPrivacy('compactOnly', ($event.target as HTMLInputElement).checked)"
      />
    </label>

    <label class="toggle-row">
      <span>
        <strong>鼠标穿透</strong>
        <small>默认关闭；MVP 仅保留 Tauri command 接口。</small>
      </span>
      <input
        type="checkbox"
        :checked="settings.mousePassthrough"
        @change="preferencesStore.setMousePassthroughPreference(($event.target as HTMLInputElement).checked)"
      />
    </label>
  </div>
</template>
