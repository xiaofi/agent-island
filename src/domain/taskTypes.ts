export type AgentSource = "codex" | "claude-code" | "manual";

export type TaskStatus =
  | "discovering"
  | "running"
  | "thinking"
  | "tool-running"
  | "waiting-user"
  | "completed"
  | "failed"
  | "paused"
  | "stale";

export type AgentEventType =
  | "session-started"
  | "user-message"
  | "assistant-thinking"
  | "tool-started"
  | "tool-finished"
  | "waiting-for-user"
  | "session-completed"
  | "session-failed"
  | "heartbeat";

export interface WindowHint {
  appName?: string;
  processId?: number;
  windowTitle?: string;
}

export interface AgentEvent {
  id: string;
  taskId: string;
  type: AgentEventType;
  timestamp: string;
  summary: string;
  metadata?: Record<string, unknown>;
}

export interface AgentTask {
  id: string;
  source: AgentSource;
  title: string;
  cwd?: string;
  status: TaskStatus;
  startedAt?: string;
  updatedAt: string;
  durationSeconds?: number;
  lastAction?: string;
  waitingReason?: string;
  errorSummary?: string;
  windowHint?: WindowHint;
  events: AgentEvent[];
}

export type DiagnosticStatus = "ok" | "partial" | "unavailable" | "error";

export interface CandidatePath {
  path: string;
  exists: boolean;
  readable: boolean;
  reason?: string;
  updatedAt?: string;
}

export interface DiscoveredProcess {
  pid: number;
  name: string;
  command: string;
  cwd?: string;
}

export interface AdapterDiagnostic {
  source: AgentSource;
  status: DiagnosticStatus;
  summary: string;
  processes: DiscoveredProcess[];
  candidatePaths: CandidatePath[];
  parsedSessions: number;
  updatedAt: string;
}

export interface AppearanceSettings {
  islandOpacity: number;
}

export type NotificationSound = "default" | "none" | "Basso" | "Glass" | "Hero" | "Ping" | "Pop" | "Sosumi" | "Tink";

export interface NotificationSettings {
  enabled: boolean;
  sound: NotificationSound;
}

export interface AutoAcknowledgeSettings {
  enabled: boolean;
  delaySeconds: number;
}

export type HookOperation = "install" | "uninstall" | "repair" | "self-test";

export interface HookOperationError {
  operation: HookOperation;
  code: string;
  message: string;
  occurredAt: string;
  retryAction: HookOperation;
}

export interface HookSourceErrors {
  codex?: HookOperationError;
  claudeCode?: HookOperationError;
}

export interface HookSourceSettings {
  codex: boolean;
  claudeCode: boolean;
  lastErrors: HookSourceErrors;
}

export interface IslandWindowSettings {
  x?: number;
  y?: number;
}

export interface AppSettings {
  appearance: AppearanceSettings;
  notifications: NotificationSettings;
  autoAcknowledge: AutoAcknowledgeSettings;
  quietMode: boolean;
  showInDock: boolean;
  enabledAdapters: AgentSource[];
  hookSource: HookSourceSettings;
  islandWindow: IslandWindowSettings;
}

export interface AgentBridgeSubscriptions {
  onTaskUpdated: (task: AgentTask) => void;
  onTaskRemoved: (taskId: string) => void;
  onEventCreated: (event: AgentEvent) => void;
  onDiagnosticUpdated: (diagnostic: AdapterDiagnostic) => void;
  onSettingsUpdated: (settings: AppSettings) => void;
}
