use std::collections::{HashMap, HashSet};

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
        let Some(payload) = notification_payload(task, previous_tasks.get(&task.id), settings)
        else {
            continue;
        };

        if state.notified_keys.contains(&payload.key) {
            continue;
        }

        let mut builder = app_handle
            .notification()
            .builder()
            .title(payload.title)
            .body(payload.body)
            .group("agent-island-tasks");

        if let Some(sound) = notification_sound_name(&settings.notifications.sound) {
            builder = builder.sound(sound);
        }

        let result = builder.show();

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

fn notification_payload(
    task: &AgentTask,
    previous: Option<&AgentTask>,
    settings: &AppSettings,
) -> Option<TaskNotificationPayload> {
    let status = notification_status_label(&task.status)?;

    if previous.is_some_and(|previous| previous.status == task.status) {
        return None;
    }

    let key = notification_key(task)?;
    let source = source_label(&task.source);
    let body = if settings.privacy.hide_task_title || task.title.trim().is_empty() {
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

fn notification_sound_name(sound: &str) -> Option<&'static str> {
    match sound.trim() {
        "none" => None,
        "Basso" => Some("Basso"),
        "Glass" => Some("Glass"),
        "Hero" => Some("Hero"),
        "Ping" => Some("Ping"),
        "Pop" => Some("Pop"),
        "Sosumi" => Some("Sosumi"),
        "Tink" => Some("Tink"),
        "default" | "" => Some("NSUserNotificationDefaultSoundName"),
        _ => Some("NSUserNotificationDefaultSoundName"),
    }
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
    use crate::adapters::types::{
        default_settings, AgentEvent, AgentEventType, AgentSource, AgentTask, TaskStatus,
    };

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
        let settings = default_settings();
        let previous = task(TaskStatus::Completed, "2026-06-11T01:00:00Z");
        let next = task(TaskStatus::Completed, "2026-06-11T01:00:02Z");

        assert!(notification_payload(&next, Some(&previous), &settings).is_none());
    }

    #[test]
    fn notifies_when_task_enters_completed() {
        let settings = default_settings();
        let previous = task(TaskStatus::Thinking, "2026-06-11T01:00:00Z");
        let next = task(TaskStatus::Completed, "2026-06-11T01:00:02Z");

        let payload = notification_payload(&next, Some(&previous), &settings).unwrap();

        assert_eq!(payload.title, "Codex 任务已完成");
        assert_eq!(payload.body, "agent-island");
    }

    #[test]
    fn masks_title_when_privacy_requires_it() {
        let mut settings = default_settings();
        settings.privacy.hide_task_title = true;
        let previous = task(TaskStatus::Thinking, "2026-06-11T01:00:00Z");
        let next = task(TaskStatus::Completed, "2026-06-11T01:00:02Z");

        let payload = notification_payload(&next, Some(&previous), &settings).unwrap();

        assert_eq!(payload.body, "Codex task");
    }

    #[test]
    fn maps_notification_sound_preferences() {
        assert_eq!(
            notification_sound_name("default"),
            Some("NSUserNotificationDefaultSoundName")
        );
        assert_eq!(notification_sound_name("Ping"), Some("Ping"));
        assert_eq!(notification_sound_name("none"), None);
        assert_eq!(
            notification_sound_name("unknown"),
            Some("NSUserNotificationDefaultSoundName")
        );
    }
}
