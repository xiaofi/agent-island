<script setup lang="ts">
import { RefreshCw } from "@lucide/vue";
import IconButton from "@/components/primitives/IconButton.vue";
import { sourceLabel } from "@/domain/taskPresentation";
import type { AdapterDiagnostic, AgentSource, HookOperationError } from "@/domain/taskTypes";
import { usePreferencesStore } from "@/stores/preferencesStore";
import { useTaskStore } from "@/stores/taskStore";

defineProps<{
  diagnostics: AdapterDiagnostic[];
}>();

const taskStore = useTaskStore();
const preferencesStore = usePreferencesStore();

function hookError(source: AgentSource): HookOperationError | undefined {
  if (source === "codex") {
    return preferencesStore.settings.hookSource.lastErrors.codex;
  }
  if (source === "claude-code") {
    return preferencesStore.settings.hookSource.lastErrors.claudeCode;
  }
  return undefined;
}
</script>

<template>
  <div class="diagnostics-panel">
    <div class="diagnostics-panel__toolbar">
      <span>Adapter discovery</span>
      <IconButton label="刷新诊断" @click="taskStore.refreshDiagnostics()">
        <RefreshCw :size="16" />
      </IconButton>
    </div>

    <article v-for="diagnostic in diagnostics" :key="diagnostic.source" class="diagnostic-card">
      <header>
        <strong>{{ sourceLabel(diagnostic.source) }}</strong>
        <span :class="`diagnostic-card__status diagnostic-card__status--${diagnostic.status}`">
          {{ diagnostic.status }}
        </span>
      </header>
      <p>{{ diagnostic.summary }}</p>
      <p v-if="hookError(diagnostic.source)" class="diagnostic-card__error">
        {{ hookError(diagnostic.source)?.operation }} 失败：{{ hookError(diagnostic.source)?.message }} ·
        {{ new Date(hookError(diagnostic.source)!.occurredAt).toLocaleString() }}
      </p>
      <dl>
        <div>
          <dt>进程</dt>
          <dd>{{ diagnostic.processes.length }}</dd>
        </div>
        <div>
          <dt>候选路径</dt>
          <dd>{{ diagnostic.candidatePaths.length }}</dd>
        </div>
        <div>
          <dt>可解析会话</dt>
          <dd>{{ diagnostic.parsedSessions }}</dd>
        </div>
      </dl>
      <ul v-if="diagnostic.candidatePaths.length" class="candidate-list">
        <li v-for="candidate in diagnostic.candidatePaths" :key="candidate.path">
          <span>{{ candidate.path }}</span>
          <em>{{ candidate.readable ? "可读" : candidate.reason || "不可读" }}</em>
        </li>
      </ul>
    </article>
  </div>
</template>
