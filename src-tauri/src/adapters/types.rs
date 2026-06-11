use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum AgentSource {
    Codex,
    ClaudeCode,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TaskStatus {
    Discovering,
    Running,
    Thinking,
    ToolRunning,
    WaitingUser,
    Completed,
    Failed,
    Paused,
    Stale,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AgentEventType {
    SessionStarted,
    UserMessage,
    AssistantThinking,
    ToolStarted,
    ToolFinished,
    WaitingForUser,
    SessionCompleted,
    SessionFailed,
    Heartbeat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowHint {
    pub app_name: Option<String>,
    pub process_id: Option<u32>,
    pub window_title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentEvent {
    pub id: String,
    pub task_id: String,
    pub r#type: AgentEventType,
    pub timestamp: String,
    pub summary: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentTask {
    pub id: String,
    pub source: AgentSource,
    pub title: String,
    pub cwd: Option<String>,
    pub status: TaskStatus,
    pub started_at: Option<String>,
    pub updated_at: String,
    pub duration_seconds: Option<u64>,
    pub last_action: Option<String>,
    pub waiting_reason: Option<String>,
    pub error_summary: Option<String>,
    pub window_hint: Option<WindowHint>,
    pub events: Vec<AgentEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiagnosticStatus {
    Ok,
    Partial,
    Unavailable,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CandidatePath {
    pub path: String,
    pub exists: bool,
    pub readable: bool,
    pub reason: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredProcess {
    pub pid: u32,
    pub name: String,
    pub command: String,
    pub cwd: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdapterDiagnostic {
    pub source: AgentSource,
    pub status: DiagnosticStatus,
    pub summary: String,
    pub processes: Vec<DiscoveredProcess>,
    pub candidate_paths: Vec<CandidatePath>,
    pub parsed_sessions: usize,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivacySettings {
    pub hide_project_path: bool,
    pub hide_task_title: bool,
    pub compact_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppearanceSettings {
    pub island_opacity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationSettings {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoAcknowledgeSettings {
    pub enabled: bool,
    pub delay_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum HookOperation {
    Install,
    Uninstall,
    Repair,
    SelfTest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookOperationError {
    pub operation: HookOperation,
    pub code: String,
    pub message: String,
    pub occurred_at: String,
    pub retry_action: HookOperation,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct HookSourceErrors {
    pub codex: Option<HookOperationError>,
    pub claude_code: Option<HookOperationError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookSourceSettings {
    pub codex: bool,
    pub claude_code: bool,
    #[serde(default)]
    pub last_errors: HookSourceErrors,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct IslandWindowSettings {
    pub x: Option<i32>,
    pub y: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub privacy: PrivacySettings,
    #[serde(default = "default_appearance_settings")]
    pub appearance: AppearanceSettings,
    #[serde(default = "default_notification_settings")]
    pub notifications: NotificationSettings,
    #[serde(default = "default_auto_acknowledge_settings")]
    pub auto_acknowledge: AutoAcknowledgeSettings,
    #[serde(default)]
    pub quiet_mode: bool,
    pub mouse_passthrough: bool,
    #[serde(default)]
    pub show_in_dock: bool,
    pub enabled_adapters: Vec<AgentSource>,
    #[serde(default = "default_hook_source_settings")]
    pub hook_source: HookSourceSettings,
    #[serde(default)]
    pub island_window: IslandWindowSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppSettingsPatch {
    pub privacy: Option<PrivacySettings>,
    pub appearance: Option<AppearanceSettings>,
    pub notifications: Option<NotificationSettings>,
    pub auto_acknowledge: Option<AutoAcknowledgeSettings>,
    pub quiet_mode: Option<bool>,
    pub mouse_passthrough: Option<bool>,
    pub show_in_dock: Option<bool>,
    pub enabled_adapters: Option<Vec<AgentSource>>,
    pub hook_source: Option<HookSourceSettings>,
    pub island_window: Option<IslandWindowSettings>,
}

pub fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

pub fn default_settings() -> AppSettings {
    AppSettings {
        privacy: PrivacySettings {
            hide_project_path: false,
            hide_task_title: false,
            compact_only: false,
        },
        appearance: default_appearance_settings(),
        notifications: default_notification_settings(),
        auto_acknowledge: default_auto_acknowledge_settings(),
        quiet_mode: false,
        mouse_passthrough: false,
        show_in_dock: false,
        enabled_adapters: vec![
            AgentSource::Manual,
            AgentSource::Codex,
            AgentSource::ClaudeCode,
        ],
        hook_source: default_hook_source_settings(),
        island_window: IslandWindowSettings::default(),
    }
}

pub fn default_appearance_settings() -> AppearanceSettings {
    AppearanceSettings {
        island_opacity: 0.92,
    }
}

pub fn default_notification_settings() -> NotificationSettings {
    NotificationSettings { enabled: false }
}

pub fn default_auto_acknowledge_settings() -> AutoAcknowledgeSettings {
    AutoAcknowledgeSettings {
        enabled: false,
        delay_seconds: 900,
    }
}

pub fn default_hook_source_settings() -> HookSourceSettings {
    HookSourceSettings {
        codex: false,
        claude_code: false,
        last_errors: HookSourceErrors::default(),
    }
}
