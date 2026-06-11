<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { storeToRefs } from "pinia";
import { RefreshCw } from "@lucide/vue";
import { sourceLabel } from "@/domain/privacy";
import type { AgentSource, HookOperationError, NotificationSound } from "@/domain/taskTypes";
import { usePreferencesStore } from "@/stores/preferencesStore";
import { useTaskStore } from "@/stores/taskStore";

const preferencesStore = usePreferencesStore();
const taskStore = useTaskStore();
const { settings } = storeToRefs(preferencesStore);

type HookUiSource = Extract<AgentSource, "codex" | "claude-code">;

const hookSources: HookUiSource[] = ["claude-code", "codex"];
const opacityOptions = [0.55, 0.7, 0.85, 0.92, 1];
const autoAcknowledgeOptions = [
  { label: "5 分钟", value: 300 },
  { label: "15 分钟", value: 900 },
  { label: "30 分钟", value: 1800 },
  { label: "1 小时", value: 3600 },
];
const notificationSoundOptions: { label: string; value: NotificationSound }[] = [
  { label: "系统默认", value: "default" },
  { label: "Basso", value: "Basso" },
  { label: "Ping", value: "Ping" },
  { label: "Glass", value: "Glass" },
  { label: "Hero", value: "Hero" },
  { label: "Pop", value: "Pop" },
  { label: "Sosumi", value: "Sosumi" },
  { label: "Tink", value: "Tink" },
  { label: "无声", value: "none" },
];
const busySource = ref<HookUiSource>();
const pendingSource = ref<HookUiSource>();

const diagnosticsBySource = computed(() => {
  return Object.fromEntries(taskStore.diagnostics.map((diagnostic) => [diagnostic.source, diagnostic]));
});
const opacityPercent = computed(() => Math.round(settings.value.appearance.islandOpacity * 100));

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
  if (pendingSource.value === source) {
    return "等待确认";
  }
  const error = errorForSource(source);
  if (error) {
    return error.operation === "uninstall" ? "卸载失败" : error.operation === "self-test" ? "自检失败" : "安装失败";
  }
  if (busySource.value === source) {
    return "接入中";
  }
  return isEnabled(source) ? "已接入" : "未接入";
}

function hookTargetPath(source: HookUiSource) {
  return source === "codex" ? "~/.codex/hooks.json" : "~/.claude/settings.json";
}

function hookEventSummary(source: HookUiSource) {
  return source === "codex"
    ? "SessionStart、UserPromptSubmit、PreToolUse、PermissionRequest、PostToolUse、SubagentStart、SubagentStop、Stop"
    : "SessionStart、UserPromptSubmit、PreToolUse、PermissionRequest、PostToolUse、Notification、Stop、SessionEnd、CwdChanged";
}

async function setSource(source: HookUiSource, enabled: boolean, confirmed = false) {
  if (enabled && !confirmed) {
    pendingSource.value = source;
    return;
  }

  busySource.value = source;
  try {
    await preferencesStore.setHookSource(source, enabled);
    if (pendingSource.value === source) {
      pendingSource.value = undefined;
    }
  } finally {
    busySource.value = undefined;
  }
}

function handleSourceToggle(source: HookUiSource, event: Event) {
  const input = event.target as HTMLInputElement;
  const enabled = input.checked;
  if (enabled) {
    pendingSource.value = source;
    input.checked = isEnabled(source);
    return;
  }

  void setSource(source, enabled).finally(() => {
    input.checked = isEnabled(source);
  });
}

function cancelPendingSource(source: HookUiSource) {
  if (pendingSource.value === source) {
    pendingSource.value = undefined;
  }
}

async function confirmPendingSource(source: HookUiSource) {
  await setSource(source, true, true);
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

function handleOpacityChange(event: Event) {
  const value = Number((event.target as HTMLInputElement).value);
  void preferencesStore.setIslandOpacity(value);
}

function handleNotificationSoundChange(event: Event) {
  const value = (event.target as HTMLSelectElement).value as NotificationSound;
  void preferencesStore.setNotificationSound(value);
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
        <small>压缩态只保留来源、状态和任务数量；展开列表仍按路径和标题设置展示。</small>
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
        <small>开启后悬浮岛不接收鼠标事件，适合只通过快捷方式或菜单控制时使用。</small>
      </span>
      <input
        type="checkbox"
        :checked="settings.mousePassthrough"
        @change="preferencesStore.setMousePassthroughPreference(($event.target as HTMLInputElement).checked)"
      />
    </label>

    <label class="toggle-row">
      <span>
        <strong>显示在 Dock 栏</strong>
        <small>开启后应用显示 Dock 图标；关闭后保持后台悬浮工具形态。</small>
      </span>
      <input
        type="checkbox"
        :checked="settings.showInDock"
        @change="preferencesStore.setShowInDockPreference(($event.target as HTMLInputElement).checked)"
      />
    </label>

    <label class="toggle-row">
      <span>
        <strong>安静模式</strong>
        <small>悬浮岛只显示等待处理、失败和完成任务，隐藏普通运行状态。</small>
      </span>
      <input
        type="checkbox"
        :checked="settings.quietMode"
        @change="preferencesStore.setQuietMode(($event.target as HTMLInputElement).checked)"
      />
    </label>

    <section class="settings-section">
      <header class="settings-section__header">
        <div>
          <strong>外观与提醒</strong>
          <small>控制悬浮岛可见度、关键状态通知和完成提醒保留时间。</small>
        </div>
      </header>

      <label class="setting-field">
        <span>
          <strong>悬浮岛透明度</strong>
          <small>固定应用到压缩态和下拉任务面板。</small>
        </span>
        <div class="range-control">
          <input
            type="range"
            min="0.55"
            max="1"
            step="0.01"
            :value="settings.appearance.islandOpacity"
            @input="handleOpacityChange"
          />
          <output>{{ opacityPercent }}%</output>
        </div>
      </label>

      <div class="segmented-control" aria-label="透明度预设">
        <button
          v-for="option in opacityOptions"
          :key="option"
          type="button"
          :class="{ 'segmented-control__button--active': settings.appearance.islandOpacity === option }"
          @click="preferencesStore.setIslandOpacity(option)"
        >
          {{ Math.round(option * 100) }}%
        </button>
      </div>

      <label class="toggle-row">
        <span>
          <strong>关键状态通知</strong>
          <small>任务等待处理、失败或完成时发送系统通知。</small>
        </span>
        <input
          type="checkbox"
          :checked="settings.notifications.enabled"
          @change="preferencesStore.setNotificationsEnabled(($event.target as HTMLInputElement).checked)"
        />
      </label>

      <label class="setting-field">
        <span>
          <strong>通知声音</strong>
          <small>开启关键状态通知后使用；选择无声时只发送通知。</small>
        </span>
        <select :value="settings.notifications.sound" @change="handleNotificationSoundChange">
          <option v-for="option in notificationSoundOptions" :key="option.value" :value="option.value">
            {{ option.label }}
          </option>
        </select>
      </label>

      <label class="toggle-row">
        <span>
          <strong>自动确认完成任务</strong>
          <small>完成任务在指定时间后自动归档，不一直占用悬浮岛。</small>
        </span>
        <input
          type="checkbox"
          :checked="settings.autoAcknowledge.enabled"
          @change="preferencesStore.setAutoAcknowledgeEnabled(($event.target as HTMLInputElement).checked)"
        />
      </label>

      <label class="setting-field">
        <span>
          <strong>自动确认时间</strong>
          <small>仅在自动确认开启后生效。</small>
        </span>
        <select
          :value="settings.autoAcknowledge.delaySeconds"
          :disabled="!settings.autoAcknowledge.enabled"
          @change="preferencesStore.setAutoAcknowledgeDelay(Number(($event.target as HTMLSelectElement).value))"
        >
          <option v-for="option in autoAcknowledgeOptions" :key="option.value" :value="option.value">
            {{ option.label }}
          </option>
        </select>
      </label>
    </section>

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

          <div v-if="pendingSource === source" class="hook-confirm">
            <strong>确认接入 {{ sourceLabel(source) }} 状态</strong>
            <small>
              将备份并写入 {{ hookTargetPath(source) }}，只追加 Agent Island 自己的 command，覆盖事件：
              {{ hookEventSummary(source) }}。helper 只落盘最小状态，不保存 prompt、回复正文、完整工具输入输出或完整 shell command。
            </small>
            <div class="hook-confirm__actions">
              <button
                class="secondary-button"
                type="button"
                :disabled="busySource === source"
                @click="confirmPendingSource(source)"
              >
                确认接入
              </button>
              <button class="secondary-button" type="button" :disabled="busySource === source" @click="cancelPendingSource(source)">
                取消
              </button>
            </div>
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
