<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { storeToRefs } from "pinia";
import { RefreshCw } from "@lucide/vue";
import { sourceLabel } from "@/domain/privacy";
import type { AgentSource, HookOperationError } from "@/domain/taskTypes";
import { usePreferencesStore } from "@/stores/preferencesStore";
import { useTaskStore } from "@/stores/taskStore";

const preferencesStore = usePreferencesStore();
const taskStore = useTaskStore();
const { settings } = storeToRefs(preferencesStore);

type HookUiSource = Extract<AgentSource, "codex" | "claude-code">;

const hookSources: HookUiSource[] = ["claude-code", "codex"];
const busySource = ref<HookUiSource>();

const diagnosticsBySource = computed(() => {
  return Object.fromEntries(taskStore.diagnostics.map((diagnostic) => [diagnostic.source, diagnostic]));
});

onMounted(() => {
  if (!taskStore.diagnostics.length) {
    void taskStore.refreshDiagnostics();
  }
});

function isInstalled(source: HookUiSource) {
  const diagnostic = diagnosticsBySource.value[source];
  return Boolean(diagnostic && diagnostic.status !== "unavailable" && diagnostic.status !== "error");
}

function isEnabled(source: HookUiSource) {
  return source === "codex" ? settings.value.hookSource.codex : settings.value.hookSource.claudeCode;
}

function errorForSource(source: HookUiSource): HookOperationError | undefined {
  return source === "codex" ? settings.value.hookSource.lastErrors.codex : settings.value.hookSource.lastErrors.claudeCode;
}

function statusText(source: HookUiSource) {
  const error = errorForSource(source);
  if (error) {
    return error.operation === "uninstall" ? "卸载失败" : error.operation === "self-test" ? "自检失败" : "安装失败";
  }
  if (busySource.value === source) {
    return "接入中";
  }
  return isEnabled(source) ? "已接入" : "未接入";
}

async function setSource(source: HookUiSource, enabled: boolean) {
  if (enabled) {
    const confirmed = window.confirm(
      `将为 ${sourceLabel(source)} 安装 Agent Island hook command。此操作会先备份并只追加 Agent Island 自己的配置。`,
    );
    if (!confirmed) {
      return;
    }
  }

  busySource.value = source;
  try {
    await preferencesStore.setHookSource(source, enabled);
  } finally {
    busySource.value = undefined;
  }
}

function handleSourceToggle(source: HookUiSource, event: Event) {
  const input = event.target as HTMLInputElement;
  const enabled = input.checked;
  void setSource(source, enabled).finally(() => {
    input.checked = isEnabled(source);
  });
}

async function retrySource(source: HookUiSource, error: HookOperationError) {
  busySource.value = source;
  try {
    await preferencesStore.retryHookSource(source, error.retryAction);
  } finally {
    busySource.value = undefined;
  }
}

async function selfTest(source: HookUiSource) {
  busySource.value = source;
  try {
    await preferencesStore.selfTestHookSource(source);
  } finally {
    busySource.value = undefined;
  }
}
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

    <section class="settings-section">
      <header class="settings-section__header">
        <div>
          <strong>状态采集</strong>
          <small>按来源独立接入；关闭开关会卸载 Agent Island 自己的 hook command。</small>
        </div>
        <button class="secondary-button" type="button" @click="taskStore.refreshDiagnostics()">
          <RefreshCw :size="14" />
          刷新
        </button>
      </header>

      <article v-for="source in hookSources" :key="source" class="hook-source-card">
        <template v-if="isInstalled(source)">
          <div class="hook-source-card__main">
            <div>
              <strong>{{ sourceLabel(source) }}</strong>
              <small>{{ source === "claude-code" ? "此开关只影响 Claude Code" : "此开关只影响 Codex" }}</small>
            </div>
            <label class="switch-control">
              <input
                type="checkbox"
                :checked="isEnabled(source)"
                :disabled="busySource === source"
                @change="handleSourceToggle(source, $event)"
              />
              <span>{{ isEnabled(source) ? "接入" : "未接入" }}</span>
            </label>
          </div>

          <div class="hook-source-card__status">
            <span :class="{ 'hook-source-card__badge--error': errorForSource(source) }">
              {{ statusText(source) }}
            </span>
            <small v-if="errorForSource(source)">
              {{ errorForSource(source)?.message }} · {{ new Date(errorForSource(source)!.occurredAt).toLocaleString() }}
            </small>
            <small v-else>
              {{ diagnosticsBySource[source]?.summary }}
            </small>
          </div>

          <div class="hook-source-card__actions">
            <button
              class="secondary-button"
              type="button"
              :disabled="busySource === source"
              @click="setSource(source, true)"
            >
              修复接入
            </button>
            <button
              v-if="errorForSource(source)"
              class="secondary-button"
              type="button"
              :disabled="busySource === source"
              @click="retrySource(source, errorForSource(source)!)"
            >
              重试
            </button>
            <button
              class="secondary-button"
              type="button"
              :disabled="busySource === source || !isEnabled(source)"
              @click="selfTest(source)"
            >
              运行自检
            </button>
          </div>
        </template>
        <template v-else>
          <div class="hook-source-card__main">
            <div>
              <strong>{{ sourceLabel(source) }}</strong>
              <small>未发现 {{ sourceLabel(source) }} 安装</small>
            </div>
          </div>
        </template>
      </article>
    </section>
  </div>
</template>
