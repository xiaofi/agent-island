use std::collections::{HashMap, HashSet};

#[cfg(target_os = "macos")]
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
};

use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

use crate::adapters::types::{
    AgentEvent, AgentEventType, AgentSource, AgentTask, AppSettings, TaskStatus,
};

#[derive(Default)]
pub struct TaskNotificationState {
    initialized: bool,
    notified_keys: HashSet<String>,
}

struct TaskNotificationPayload {
    key: String,
    title: String,
    body: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NotificationSoundPreference {
    Default,
    System(&'static str),
}

impl NotificationSoundPreference {
    #[cfg(not(target_os = "macos"))]
    fn notification_sound_name(self) -> &'static str {
        match self {
            Self::Default => "NSUserNotificationDefaultSoundName",
            Self::System(name) => name,
        }
    }
}

pub fn notify_task_updates(
    app_handle: &AppHandle,
    state: &mut TaskNotificationState,
    settings: &AppSettings,
    tasks: &[AgentTask],
    previous_tasks: &HashMap<String, AgentTask>,
) {
    if !state.initialized {
        for task in tasks {
            if let Some(key) = notification_key(task) {
                state.notified_keys.insert(key);
            }
        }
        state.initialized = true;
        return;
    }

    if !settings.notifications.enabled {
        for task in tasks {
            if let Some(key) = notification_key(task) {
                state.notified_keys.insert(key);
            }
        }
        return;
    }

    for task in tasks {
        let Some(payload) = notification_payload(task, previous_tasks.get(&task.id)) else {
            continue;
        };

        if state.notified_keys.contains(&payload.key) {
            continue;
        }

        let result = show_notification(
            app_handle,
            payload.title,
            payload.body,
            "agent-island-tasks",
            &settings.notifications.sound,
        );

        match result {
            Ok(()) => {
                state.notified_keys.insert(payload.key);
                prune_notified_keys(&mut state.notified_keys);
            }
            Err(error) => {
                eprintln!("[task_notifications] failed to send notification: {error}");
            }
        }
    }
}

pub fn send_test_notification(app_handle: &AppHandle, sound: &str) -> Result<(), String> {
    show_notification(
        app_handle,
        "Agent Island 测试通知".to_string(),
        "如果你看到这条通知，系统通知已生效。".to_string(),
        "agent-island-tests",
        sound,
    )
}

fn show_notification(
    app_handle: &AppHandle,
    title: String,
    body: String,
    group: &'static str,
    sound: &str,
) -> Result<(), String> {
    let sound = notification_sound(sound);
    let builder = app_handle
        .notification()
        .builder()
        .title(title)
        .body(body)
        .group(group);

    #[cfg(not(target_os = "macos"))]
    let builder = if let Some(sound) = sound {
        builder.sound(sound.notification_sound_name())
    } else {
        builder
    };

    builder
        .show()
        .map_err(|error| format!("failed to send notification: {error}"))?;

    if let Err(error) = play_notification_sound(sound) {
        eprintln!("[task_notifications] failed to play notification sound: {error}");
    }

    Ok(())
}

fn notification_payload(
    task: &AgentTask,
    previous: Option<&AgentTask>,
) -> Option<TaskNotificationPayload> {
    let status = notification_status_label(&task.status)?;

    if previous.is_some_and(|previous| previous.status == task.status) {
        return None;
    }

    let key = notification_key(task)?;
    let source = source_label(&task.source);
    let body = if task.title.trim().is_empty() {
        format!("{source} task")
    } else {
        task.title.clone()
    };

    Some(TaskNotificationPayload {
        key,
        title: format!("{source} {status}"),
        body,
    })
}

fn notification_key(task: &AgentTask) -> Option<String> {
    let event_type = notification_event_type(&task.status)?;
    let event = latest_event_of_type(task, &event_type);
    Some(format!(
        "{}::{}::{}::{}",
        task.id,
        status_slug(&task.status),
        event_type_slug(&event_type),
        event
            .map(|event| event.timestamp.as_str())
            .unwrap_or(task.updated_at.as_str())
    ))
}

fn latest_event_of_type<'a>(
    task: &'a AgentTask,
    event_type: &AgentEventType,
) -> Option<&'a AgentEvent> {
    task.events
        .iter()
        .filter(|event| event.r#type == *event_type)
        .max_by_key(|event| event.timestamp.as_str())
}

fn notification_event_type(status: &TaskStatus) -> Option<AgentEventType> {
    match status {
        TaskStatus::WaitingUser => Some(AgentEventType::WaitingForUser),
        TaskStatus::Failed => Some(AgentEventType::SessionFailed),
        TaskStatus::Completed => Some(AgentEventType::SessionCompleted),
        _ => None,
    }
}

fn notification_status_label(status: &TaskStatus) -> Option<&'static str> {
    match status {
        TaskStatus::WaitingUser => Some("等待处理"),
        TaskStatus::Failed => Some("任务失败"),
        TaskStatus::Completed => Some("任务已完成"),
        _ => None,
    }
}

fn status_slug(status: &TaskStatus) -> &'static str {
    match status {
        TaskStatus::Discovering => "discovering",
        TaskStatus::Running => "running",
        TaskStatus::Thinking => "thinking",
        TaskStatus::ToolRunning => "tool-running",
        TaskStatus::WaitingUser => "waiting-user",
        TaskStatus::Completed => "completed",
        TaskStatus::Failed => "failed",
        TaskStatus::Paused => "paused",
        TaskStatus::Stale => "stale",
    }
}

fn event_type_slug(event_type: &AgentEventType) -> &'static str {
    match event_type {
        AgentEventType::SessionStarted => "session-started",
        AgentEventType::UserMessage => "user-message",
        AgentEventType::AssistantThinking => "assistant-thinking",
        AgentEventType::ToolStarted => "tool-started",
        AgentEventType::ToolFinished => "tool-finished",
        AgentEventType::WaitingForUser => "waiting-for-user",
        AgentEventType::SessionCompleted => "session-completed",
        AgentEventType::SessionFailed => "session-failed",
        AgentEventType::Heartbeat => "heartbeat",
    }
}

fn source_label(source: &AgentSource) -> &'static str {
    match source {
        AgentSource::Codex => "Codex",
        AgentSource::ClaudeCode => "Claude Code",
        AgentSource::Manual => "Manual",
    }
}

fn notification_sound(sound: &str) -> Option<NotificationSoundPreference> {
    match sound.trim() {
        "none" => None,
        "Basso" => Some(NotificationSoundPreference::System("Basso")),
        "Glass" => Some(NotificationSoundPreference::System("Glass")),
        "Hero" => Some(NotificationSoundPreference::System("Hero")),
        "Ping" => Some(NotificationSoundPreference::System("Ping")),
        "Pop" => Some(NotificationSoundPreference::System("Pop")),
        "Sosumi" => Some(NotificationSoundPreference::System("Sosumi")),
        "Tink" => Some(NotificationSoundPreference::System("Tink")),
        "default" | "" => Some(NotificationSoundPreference::Default),
        _ => Some(NotificationSoundPreference::Default),
    }
}

#[cfg(target_os = "macos")]
fn play_notification_sound(sound: Option<NotificationSoundPreference>) -> Result<(), String> {
    let Some(sound) = sound else {
        return Ok(());
    };

    let path = match sound {
        NotificationSoundPreference::Default => {
            default_macos_alert_sound_path().unwrap_or_else(|| macos_system_sound_path("Ping"))
        }
        NotificationSoundPreference::System(name) => macos_system_sound_path(name),
    };

    spawn_afplay(&path)
}

#[cfg(not(target_os = "macos"))]
fn play_notification_sound(_sound: Option<NotificationSoundPreference>) -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "macos")]
fn default_macos_alert_sound_path() -> Option<PathBuf> {
    static DEFAULT_ALERT_SOUND_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();

    DEFAULT_ALERT_SOUND_PATH
        .get_or_init(|| {
            let output = Command::new("/usr/bin/defaults")
                .args(["read", "-g", "com.apple.sound.beep.sound"])
                .output()
                .ok()?;

            if !output.status.success() {
                return None;
            }

            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let path = PathBuf::from(path);
            path.exists().then_some(path)
        })
        .clone()
}

#[cfg(target_os = "macos")]
fn macos_system_sound_path(name: &str) -> PathBuf {
    PathBuf::from("/System/Library/Sounds").join(format!("{name}.aiff"))
}

#[cfg(target_os = "macos")]
fn spawn_afplay(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("sound file does not exist: {}", path.display()));
    }

    let mut child = Command::new("/usr/bin/afplay")
        .arg(path)
        .spawn()
        .map_err(|error| format!("failed to launch afplay: {error}"))?;

    std::thread::spawn(move || {
        let _ = child.wait();
    });

    Ok(())
}

fn prune_notified_keys(keys: &mut HashSet<String>) {
    if keys.len() <= 200 {
        return;
    }

    let keep: HashSet<_> = keys.iter().skip(keys.len() - 200).cloned().collect();
    *keys = keep;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::types::{AgentEvent, AgentEventType, AgentSource, AgentTask, TaskStatus};

    fn event(id: &str, event_type: AgentEventType, timestamp: &str) -> AgentEvent {
        AgentEvent {
            id: id.to_string(),
            task_id: "task-1".to_string(),
            r#type: event_type,
            timestamp: timestamp.to_string(),
            summary: "完成".to_string(),
            metadata: None,
        }
    }

    fn task(status: TaskStatus, timestamp: &str) -> AgentTask {
        AgentTask {
            id: "task-1".to_string(),
            source: AgentSource::Codex,
            title: "agent-island".to_string(),
            cwd: None,
            status,
            started_at: Some("2026-06-11T01:00:00Z".to_string()),
            updated_at: timestamp.to_string(),
            duration_seconds: None,
            last_action: None,
            waiting_reason: None,
            error_summary: None,
            window_hint: None,
            events: vec![event(
                "complete-1",
                AgentEventType::SessionCompleted,
                timestamp,
            )],
        }
    }

    #[test]
    fn does_not_notify_when_task_stays_completed() {
        let previous = task(TaskStatus::Completed, "2026-06-11T01:00:00Z");
        let next = task(TaskStatus::Completed, "2026-06-11T01:00:02Z");

        assert!(notification_payload(&next, Some(&previous)).is_none());
    }

    #[test]
    fn notifies_when_task_enters_completed() {
        let previous = task(TaskStatus::Thinking, "2026-06-11T01:00:00Z");
        let next = task(TaskStatus::Completed, "2026-06-11T01:00:02Z");

        let payload = notification_payload(&next, Some(&previous)).unwrap();

        assert_eq!(payload.title, "Codex 任务已完成");
        assert_eq!(payload.body, "agent-island");
    }

    #[test]
    fn uses_source_fallback_when_task_title_is_empty() {
        let previous = task(TaskStatus::Thinking, "2026-06-11T01:00:00Z");
        let mut next = task(TaskStatus::Completed, "2026-06-11T01:00:02Z");
        next.title = "".to_string();

        let payload = notification_payload(&next, Some(&previous)).unwrap();

        assert_eq!(payload.body, "Codex task");
    }

    #[test]
    fn maps_notification_sound_preferences() {
        assert_eq!(
            notification_sound("default"),
            Some(NotificationSoundPreference::Default)
        );
        assert_eq!(
            notification_sound("Ping"),
            Some(NotificationSoundPreference::System("Ping"))
        );
        assert_eq!(notification_sound("none"), None);
        assert_eq!(
            notification_sound("unknown"),
            Some(NotificationSoundPreference::Default)
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn resolves_macos_system_sound_file() {
        let path = macos_system_sound_path("Ping");

        assert_eq!(path, PathBuf::from("/System/Library/Sounds/Ping.aiff"));
    }
}
