use std::{
    collections::BTreeMap,
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{
    adapters::types::{
        AgentEvent, AgentEventType, AgentSource, AgentTask, AppSettings, TaskStatus,
    },
    aggregator::sort_tasks_by_updated_at,
    services::config_store,
};

const MAX_EVENTS_PER_TASK: usize = 10;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpoolEvent {
    source: AgentSource,
    event: String,
    session_id: Option<String>,
    transcript_path: Option<String>,
    session_key: String,
    cwd: Option<String>,
    timestamp: String,
    tool_name: Option<String>,
    notification_type: Option<String>,
    action_summary: Option<String>,
}

#[derive(Debug, Clone)]
struct TaskStateOverride {
    status: TaskStatus,
    timestamp: String,
    summary: String,
}

pub fn load_tasks(settings: &AppSettings) -> Vec<AgentTask> {
    let mut events = Vec::new();

    if source_enabled(settings, &AgentSource::Codex) {
        events.extend(read_source_events("codex"));
    }

    if source_enabled(settings, &AgentSource::ClaudeCode) {
        events.extend(read_source_events("claude-code"));
    }

    tasks_from_events(events)
}

fn read_source_events(source_slug: &str) -> Vec<SpoolEvent> {
    let Some(path) = config_store::app_support_dir()
        .map(|dir| dir.join("events").join(format!("{source_slug}.jsonl")))
    else {
        return Vec::new();
    };

    let Ok(contents) = fs::read_to_string(path) else {
        return Vec::new();
    };

    contents
        .lines()
        .filter_map(|line| serde_json::from_str::<SpoolEvent>(line).ok())
        .collect()
}

fn tasks_from_events(events: Vec<SpoolEvent>) -> Vec<AgentTask> {
    let home = home_dir();
    tasks_from_events_with_home(events, home.as_deref())
}

fn tasks_from_events_with_home(events: Vec<SpoolEvent>, home: Option<&Path>) -> Vec<AgentTask> {
    let mut grouped: BTreeMap<(AgentSource, String), Vec<SpoolEvent>> = BTreeMap::new();

    for event in events
        .into_iter()
        .filter(|event| !is_unidentified_hook_event(event))
    {
        grouped
            .entry((event.source.clone(), event.session_key.clone()))
            .or_default()
            .push(event);
    }

    let tasks = grouped
        .into_iter()
        .filter_map(|((source, session_key), mut events)| {
            events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
            task_from_events(source, session_key, events, home)
        })
        .collect();

    sort_tasks_by_updated_at(tasks)
}

fn task_from_events(
    source: AgentSource,
    session_key: String,
    spool_events: Vec<SpoolEvent>,
    home: Option<&Path>,
) -> Option<AgentTask> {
    let first = spool_events.first()?;
    let last = spool_events.last()?;
    let state_event = latest_state_event(&spool_events).unwrap_or(last);
    let task_id = format!("hook-{}-{}", source_slug(&source), session_key);
    let status = status_for_event(state_event).unwrap_or(TaskStatus::Running);
    let cwd = latest_cwd(&spool_events);
    let session_id = latest_session_id(&spool_events);
    let transcript_path = latest_transcript_path(&spool_events);
    let codex_session_state = if source == AgentSource::Codex {
        transcript_path
            .as_deref()
            .and_then(codex_session_state_from_transcript)
    } else {
        None
    };
    let mut status = status;
    let mut updated_at = last.timestamp.clone();
    let mut last_action = summary_for_event(state_event);
    if let Some(state) = codex_session_state {
        if state.timestamp.as_str() > state_event.timestamp.as_str() {
            status = state.status;
            last_action = state.summary;
            if state.timestamp.as_str() > updated_at.as_str() {
                updated_at = state.timestamp;
            }
        }
    }
    let title = home
        .and_then(|home| {
            resolve_session_title_in_home(
                &source,
                session_id.as_deref(),
                transcript_path.as_deref(),
                &session_key,
                home,
            )
        })
        .unwrap_or_else(|| fallback_task_title(&source, cwd.as_deref()));
    let events = spool_events
        .iter()
        .enumerate()
        .rev()
        .take(MAX_EVENTS_PER_TASK)
        .map(|(index, event)| agent_event(&task_id, index, event))
        .collect::<Vec<_>>();

    Some(AgentTask {
        id: task_id,
        source: source.clone(),
        title,
        cwd,
        status,
        started_at: Some(first.timestamp.clone()),
        updated_at,
        duration_seconds: None,
        last_action: Some(last_action),
        waiting_reason: waiting_reason_for_event(state_event).map(str::to_string),
        error_summary: error_summary_for_event(&state_event.event).map(str::to_string),
        window_hint: None,
        events,
    })
}

fn agent_event(task_id: &str, index: usize, event: &SpoolEvent) -> AgentEvent {
    AgentEvent {
        id: format!("{task_id}-{index}"),
        task_id: task_id.to_string(),
        r#type: event_type_for_event(event),
        timestamp: event.timestamp.clone(),
        summary: summary_for_event(event),
        metadata: event_metadata(event),
    }
}

fn event_metadata(event: &SpoolEvent) -> Option<serde_json::Value> {
    if event.tool_name.is_none() && event.notification_type.is_none() {
        return None;
    }

    let mut metadata = serde_json::Map::new();
    metadata.insert("hookEvent".to_string(), serde_json::json!(event.event));
    if let Some(tool_name) = event.tool_name.as_ref() {
        metadata.insert("toolName".to_string(), serde_json::json!(tool_name));
    }
    if let Some(notification_type) = event.notification_type.as_ref() {
        metadata.insert(
            "notificationType".to_string(),
            serde_json::json!(notification_type),
        );
    }
    Some(serde_json::Value::Object(metadata))
}

fn latest_cwd(events: &[SpoolEvent]) -> Option<String> {
    events
        .iter()
        .rev()
        .find_map(|event| event.cwd.as_ref().filter(|cwd| !cwd.is_empty()).cloned())
}

fn latest_session_id(events: &[SpoolEvent]) -> Option<String> {
    events.iter().rev().find_map(|event| {
        event
            .session_id
            .as_ref()
            .filter(|id| !id.is_empty())
            .cloned()
    })
}

fn latest_transcript_path(events: &[SpoolEvent]) -> Option<String> {
    events.iter().rev().find_map(|event| {
        event
            .transcript_path
            .as_ref()
            .filter(|path| !path.is_empty())
            .cloned()
    })
}

fn is_unidentified_hook_event(event: &SpoolEvent) -> bool {
    event.event == "HookEvent"
        && event
            .session_id
            .as_ref()
            .is_none_or(|value| value.trim().is_empty())
        && event
            .transcript_path
            .as_ref()
            .is_none_or(|value| value.trim().is_empty())
        && event
            .cwd
            .as_ref()
            .is_none_or(|value| value.trim().is_empty())
        && event
            .tool_name
            .as_ref()
            .is_none_or(|value| value.trim().is_empty())
        && event
            .action_summary
            .as_ref()
            .is_none_or(|value| value.trim().is_empty())
}

fn event_type_for_event(event: &SpoolEvent) -> AgentEventType {
    match event.event.as_str() {
        "SessionStart" | "SubagentStart" | "TaskCreated" => AgentEventType::SessionStarted,
        "UserPromptSubmit" => AgentEventType::UserMessage,
        "PreToolUse" => AgentEventType::ToolStarted,
        "PostToolUse" => AgentEventType::ToolFinished,
        "PermissionRequest" => AgentEventType::WaitingForUser,
        "Notification" if is_permission_notification(event) => AgentEventType::WaitingForUser,
        "Stop" | "SessionEnd" | "SubagentStop" | "TaskCompleted" => {
            AgentEventType::SessionCompleted
        }
        "PostToolUseFailure" | "StopFailure" => AgentEventType::SessionFailed,
        _ => AgentEventType::Heartbeat,
    }
}

fn latest_state_event(events: &[SpoolEvent]) -> Option<&SpoolEvent> {
    events
        .iter()
        .rev()
        .find(|event| status_for_event(event).is_some())
}

fn status_for_event(event: &SpoolEvent) -> Option<TaskStatus> {
    match event.event.as_str() {
        "SessionStart" | "SubagentStart" | "TaskCreated" => TaskStatus::Running,
        "UserPromptSubmit" => TaskStatus::Thinking,
        "PreToolUse" => TaskStatus::ToolRunning,
        "PermissionRequest" => TaskStatus::WaitingUser,
        "Notification" if is_permission_notification(event) => TaskStatus::WaitingUser,
        "Notification" | "CwdChanged" => return None,
        "PostToolUse" => TaskStatus::Thinking,
        "Stop" | "SessionEnd" | "SubagentStop" | "TaskCompleted" => TaskStatus::Completed,
        "PostToolUseFailure" | "StopFailure" => TaskStatus::Failed,
        _ => TaskStatus::Running,
    }
    .into()
}

fn is_permission_notification(event: &SpoolEvent) -> bool {
    event
        .notification_type
        .as_ref()
        .is_some_and(|value| value.contains("permission"))
}

fn summary_for_event(event: &SpoolEvent) -> String {
    if let Some(summary) = event
        .action_summary
        .as_ref()
        .filter(|summary| !summary.is_empty())
    {
        return summary.clone();
    }

    let tool = event.tool_name.as_deref().unwrap_or("工具");
    match event.event.as_str() {
        "SessionStart" => "启动会话".to_string(),
        "UserPromptSubmit" => "收到新的用户输入".to_string(),
        "PreToolUse" => format!("正在运行 {tool}"),
        "PermissionRequest" => "等待用户确认".to_string(),
        "Notification" if is_permission_notification(event) => "等待用户确认".to_string(),
        "Notification" => "收到通知".to_string(),
        "PostToolUse" => format!("{tool} 完成"),
        "PostToolUseFailure" => format!("{tool} 执行失败"),
        "Stop" | "SessionEnd" => "会话本轮完成".to_string(),
        "SubagentStart" => "子任务启动".to_string(),
        "SubagentStop" => "子任务完成".to_string(),
        "TaskCreated" => "任务已创建".to_string(),
        "TaskCompleted" => "任务已完成".to_string(),
        "CwdChanged" => "工作目录已更新".to_string(),
        _ => "收到状态事件".to_string(),
    }
}

fn waiting_reason_for_event(event: &SpoolEvent) -> Option<&'static str> {
    match event.event.as_str() {
        "PermissionRequest" => Some("Codex / Claude Code 正在等待用户确认。"),
        "Notification" if is_permission_notification(event) => {
            Some("Codex / Claude Code 正在等待用户确认。")
        }
        _ => None,
    }
}

fn error_summary_for_event(event: &str) -> Option<&'static str> {
    match event {
        "PostToolUseFailure" => Some("工具执行失败"),
        "StopFailure" => Some("会话结束失败"),
        _ => None,
    }
}

fn codex_session_state_from_transcript(transcript_path: &str) -> Option<TaskStateOverride> {
    let path = PathBuf::from(transcript_path.trim());
    let file = fs::File::open(path).ok()?;
    let reader = BufReader::new(file);
    let mut state = None;

    for line in reader.lines().map_while(Result::ok) {
        if !line.contains(r#""type":"event_msg""#) || !line.contains("turn_aborted") {
            continue;
        }

        let Ok(value) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        let is_turn_aborted = value
            .get("payload")
            .and_then(|payload| find_string_by_keys(payload, &["type"], 0))
            .is_some_and(|event_type| event_type == "turn_aborted");
        if !is_turn_aborted {
            continue;
        }

        let Some(timestamp) = find_string_by_keys(&value, &["timestamp"], 0) else {
            continue;
        };
        let summary = match find_string_by_keys(&value, &["reason"], 0) {
            Some("interrupted") => "用户手动暂停".to_string(),
            Some(reason) if !reason.trim().is_empty() => format!("任务已暂停：{reason}"),
            _ => "任务已暂停".to_string(),
        };
        state = Some(TaskStateOverride {
            status: TaskStatus::Paused,
            timestamp: timestamp.to_string(),
            summary,
        });
    }

    state
}

fn fallback_task_title(source: &AgentSource, cwd: Option<&str>) -> String {
    let source_label = source_label(source);
    let project = cwd
        .and_then(|path| path.rsplit('/').find(|part| !part.is_empty()))
        .unwrap_or("会话");
    format!("{source_label} · {project}")
}

fn resolve_session_title_in_home(
    source: &AgentSource,
    session_id: Option<&str>,
    transcript_path: Option<&str>,
    session_key: &str,
    home: &Path,
) -> Option<String> {
    match source {
        AgentSource::Codex => lookup_codex_session_title(home, session_id, session_key),
        AgentSource::ClaudeCode => lookup_claude_session_title(home, session_id, transcript_path),
        AgentSource::Manual => None,
    }
}

fn lookup_codex_session_title(
    home: &Path,
    session_id: Option<&str>,
    session_key: &str,
) -> Option<String> {
    let path = home.join(".codex/session_index.jsonl");
    let file = fs::File::open(path).ok()?;
    let reader = BufReader::new(file);
    let session_id = session_id.map(str::trim).filter(|id| !id.is_empty());

    for line in reader.lines().map_while(Result::ok) {
        let Ok(value) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        let Some(index_session_id) = find_string_by_keys(&value, &["id"], 0) else {
            continue;
        };

        let matches_session = session_id
            .map(|session_id| session_id == index_session_id)
            .unwrap_or_else(|| codex_session_key_for(index_session_id) == session_key);
        if matches_session {
            if let Some(title) = title_from_value(&value) {
                return Some(title);
            }
        }
    }

    None
}

fn lookup_claude_session_title(
    home: &Path,
    session_id: Option<&str>,
    transcript_path: Option<&str>,
) -> Option<String> {
    let session_id = session_id.map(str::trim).filter(|id| !id.is_empty());

    if let Some(path) = transcript_path
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
    {
        if let Some(title) = read_title_from_jsonl(&path, session_id, true) {
            return Some(title);
        }
    }

    let session_id = session_id?;

    for path in [
        home.join(".claude/session_index.jsonl"),
        home.join(".claude/history.jsonl"),
    ] {
        if let Some(title) = read_title_from_jsonl(&path, Some(session_id), false) {
            return Some(title);
        }
    }

    let projects_dir = home.join(".claude/projects");
    let mut candidates = Vec::new();
    collect_json_session_files(&projects_dir, session_id, 0, &mut candidates);

    for path in candidates {
        if let Some(title) = read_title_from_jsonl(&path, Some(session_id), true) {
            return Some(title);
        }
    }

    None
}

fn read_title_from_jsonl(
    path: &Path,
    session_id: Option<&str>,
    allow_session_file: bool,
) -> Option<String> {
    let file = fs::File::open(path).ok()?;
    let reader = BufReader::new(file);
    let mut title = None;

    for line in reader.lines().map_while(Result::ok) {
        let Ok(value) = serde_json::from_str::<Value>(&line) else {
            continue;
        };

        if allow_session_file || session_id.is_some_and(|id| session_matches(&value, id)) {
            if let Some(value_title) = title_from_value(&value) {
                title = Some(value_title);
            }
        }
    }

    title
}

fn collect_json_session_files(
    dir: &Path,
    session_id: &str,
    depth: usize,
    candidates: &mut Vec<PathBuf>,
) {
    if depth > 5 || candidates.len() >= 32 {
        return;
    }

    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_json_session_files(&path, session_id, depth + 1, candidates);
            continue;
        }

        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let is_session_file = file_name.contains(session_id)
            && (file_name.ends_with(".jsonl") || file_name.ends_with(".json"));
        if is_session_file {
            candidates.push(path);
        }
    }
}

fn session_matches(value: &Value, session_id: &str) -> bool {
    find_string_by_keys(
        value,
        &[
            "id",
            "session_id",
            "sessionId",
            "conversation_id",
            "conversationId",
            "thread_id",
            "threadId",
        ],
        0,
    )
    .is_some_and(|value| value == session_id)
}

fn title_from_value(value: &Value) -> Option<String> {
    find_string_by_keys(
        value,
        &[
            "thread_name",
            "threadName",
            "ai_title",
            "aiTitle",
            "title",
            "task_subject",
            "taskSubject",
            "summary",
        ],
        0,
    )
    .map(str::trim)
    .filter(|title| !title.is_empty())
    .map(str::to_string)
}

fn find_string_by_keys<'a>(value: &'a Value, keys: &[&str], depth: usize) -> Option<&'a str> {
    if depth > 3 {
        return None;
    }

    match value {
        Value::Object(object) => {
            for key in keys {
                if let Some(text) = object.get(*key).and_then(Value::as_str) {
                    return Some(text);
                }
            }

            for value in object.values() {
                if let Some(text) = find_string_by_keys(value, keys, depth + 1) {
                    return Some(text);
                }
            }

            None
        }
        Value::Array(items) => items
            .iter()
            .take(32)
            .find_map(|item| find_string_by_keys(item, keys, depth + 1)),
        _ => None,
    }
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn codex_session_key_for(session_id: &str) -> String {
    let digest = Sha256::digest(format!("codex:{session_id}"));
    format!("{digest:x}").chars().take(16).collect()
}

fn source_enabled(settings: &AppSettings, source: &AgentSource) -> bool {
    if !settings.enabled_adapters.contains(source) {
        return false;
    }

    match source {
        AgentSource::Codex => settings.hook_source.codex,
        AgentSource::ClaudeCode => settings.hook_source.claude_code,
        AgentSource::Manual => false,
    }
}

fn source_label(source: &AgentSource) -> &'static str {
    match source {
        AgentSource::Codex => "Codex",
        AgentSource::ClaudeCode => "Claude Code",
        AgentSource::Manual => "Manual",
    }
}

fn source_slug(source: &AgentSource) -> &'static str {
    match source {
        AgentSource::Codex => "codex",
        AgentSource::ClaudeCode => "claude-code",
        AgentSource::Manual => "manual",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(name: &str, timestamp: &str) -> SpoolEvent {
        SpoolEvent {
            source: AgentSource::Codex,
            event: name.to_string(),
            session_id: None,
            transcript_path: None,
            session_key: "abc123".to_string(),
            cwd: Some("/Users/spf/project/agent-island".to_string()),
            timestamp: timestamp.to_string(),
            tool_name: Some("Bash".to_string()),
            notification_type: None,
            action_summary: None,
        }
    }

    #[test]
    fn maps_latest_hook_event_to_task_status() {
        let tasks = tasks_from_events(vec![
            event("SessionStart", "2026-05-30T01:00:00Z"),
            event("PreToolUse", "2026-05-30T01:00:03Z"),
        ]);

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].status, TaskStatus::ToolRunning);
        assert_eq!(tasks[0].last_action.as_deref(), Some("正在运行 Bash"));
    }

    #[test]
    fn maps_claude_completion_events_to_completed_status() {
        for event_name in ["Stop", "SessionEnd", "SubagentStop", "TaskCompleted"] {
            let mut hook_event = event(event_name, "2026-05-30T01:00:00Z");
            hook_event.source = AgentSource::ClaudeCode;

            let tasks = tasks_from_events(vec![hook_event]);

            assert_eq!(tasks.len(), 1, "{event_name}");
            assert_eq!(tasks[0].status, TaskStatus::Completed, "{event_name}");
            assert!(
                tasks[0]
                    .events
                    .iter()
                    .any(|event| event.r#type == AgentEventType::SessionCompleted),
                "{event_name}"
            );
        }
    }

    #[test]
    fn keeps_completed_status_after_later_neutral_notification() {
        let mut stopped = event("Stop", "2026-05-30T01:00:00Z");
        stopped.source = AgentSource::ClaudeCode;
        let mut notification = event("Notification", "2026-05-30T01:01:00Z");
        notification.source = AgentSource::ClaudeCode;
        notification.tool_name = None;

        let tasks = tasks_from_events(vec![stopped, notification]);

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].status, TaskStatus::Completed);
        assert_eq!(tasks[0].last_action.as_deref(), Some("会话本轮完成"));
    }

    #[test]
    fn maps_permission_notification_to_waiting_user() {
        let mut notification = event("Notification", "2026-05-30T01:00:00Z");
        notification.source = AgentSource::ClaudeCode;
        notification.tool_name = None;
        notification.notification_type = Some("permission_prompt".to_string());

        let tasks = tasks_from_events(vec![notification]);

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].status, TaskStatus::WaitingUser);
        assert_eq!(tasks[0].events[0].r#type, AgentEventType::WaitingForUser);
    }

    #[test]
    fn maps_codex_turn_aborted_transcript_to_paused_status() {
        let home = test_home("codex-paused");
        let transcript_path = home.join(".codex/sessions/paused.jsonl");
        fs::create_dir_all(transcript_path.parent().unwrap()).unwrap();
        fs::write(
            &transcript_path,
            [
                r#"{"timestamp":"2026-05-30T01:00:00Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"ignored"}]}}"#,
                r#"{"timestamp":"2026-05-30T01:00:05Z","type":"event_msg","payload":{"type":"turn_aborted","reason":"interrupted"}}"#,
            ]
            .join("\n"),
        )
        .unwrap();
        let mut hook_event = event("UserPromptSubmit", "2026-05-30T01:00:00Z");
        hook_event.transcript_path = Some(transcript_path.to_string_lossy().to_string());

        let tasks = tasks_from_events_with_home(vec![hook_event], Some(&home));

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].status, TaskStatus::Paused);
        assert_eq!(tasks[0].updated_at, "2026-05-30T01:00:05Z");
        assert_eq!(tasks[0].last_action.as_deref(), Some("用户手动暂停"));
    }

    #[test]
    fn newer_codex_hook_event_clears_older_paused_transcript_state() {
        let home = test_home("codex-paused-cleared");
        let transcript_path = home.join(".codex/sessions/paused-cleared.jsonl");
        fs::create_dir_all(transcript_path.parent().unwrap()).unwrap();
        fs::write(
            &transcript_path,
            r#"{"timestamp":"2026-05-30T01:00:05Z","type":"event_msg","payload":{"type":"turn_aborted","reason":"interrupted"}}"#,
        )
        .unwrap();
        let mut paused = event("UserPromptSubmit", "2026-05-30T01:00:00Z");
        paused.transcript_path = Some(transcript_path.to_string_lossy().to_string());
        let mut resumed = event("UserPromptSubmit", "2026-05-30T01:00:10Z");
        resumed.transcript_path = Some(transcript_path.to_string_lossy().to_string());

        let tasks = tasks_from_events_with_home(vec![paused, resumed], Some(&home));

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].status, TaskStatus::Thinking);
        assert_eq!(tasks[0].updated_at, "2026-05-30T01:00:10Z");
    }

    #[test]
    fn groups_events_by_session() {
        let mut other = event("PermissionRequest", "2026-05-30T01:00:05Z");
        other.session_key = "other".to_string();

        let tasks = tasks_from_events(vec![event("SessionStart", "2026-05-30T01:00:00Z"), other]);

        assert_eq!(tasks.len(), 2);
    }

    #[test]
    fn uses_codex_session_index_title_when_available() {
        let home = test_home("codex-title");
        let index_path = home.join(".codex/session_index.jsonl");
        fs::create_dir_all(index_path.parent().unwrap()).unwrap();
        fs::write(
            &index_path,
            r#"{"id":"session-1","thread_name":"梳理完成通知状态逻辑","updated_at":"2026-05-30T10:14:35Z"}"#,
        )
        .unwrap();

        let mut hook_event = event("UserPromptSubmit", "2026-05-30T01:00:00Z");
        hook_event.session_id = Some("session-1".to_string());
        let tasks = tasks_from_events_with_home(vec![hook_event], Some(&home));

        assert_eq!(tasks[0].title, "梳理完成通知状态逻辑");
    }

    #[test]
    fn uses_codex_session_key_for_legacy_spool_events() {
        let home = test_home("codex-legacy-title");
        let index_path = home.join(".codex/session_index.jsonl");
        fs::create_dir_all(index_path.parent().unwrap()).unwrap();
        fs::write(
            &index_path,
            r#"{"id":"legacy-session","thread_name":"旧事件标题","updated_at":"2026-05-30T10:14:35Z"}"#,
        )
        .unwrap();

        let mut hook_event = event("UserPromptSubmit", "2026-05-30T01:00:00Z");
        hook_event.session_key = codex_session_key_for("legacy-session");
        let tasks = tasks_from_events_with_home(vec![hook_event], Some(&home));

        assert_eq!(tasks[0].title, "旧事件标题");
    }

    #[test]
    fn uses_claude_session_history_title_when_available() {
        let home = test_home("claude-title");
        let transcript_path =
            home.join(".claude/projects/Users-spf-project-agent-island/claude-session.jsonl");
        fs::create_dir_all(transcript_path.parent().unwrap()).unwrap();
        fs::write(
            &transcript_path,
            r#"{"session_id":"claude-session","title":"检查 API server 测试失败"}"#,
        )
        .unwrap();

        let mut hook_event = event("UserPromptSubmit", "2026-05-30T01:00:00Z");
        hook_event.source = AgentSource::ClaudeCode;
        hook_event.session_id = Some("claude-session".to_string());
        let tasks = tasks_from_events_with_home(vec![hook_event], Some(&home));

        assert_eq!(tasks[0].title, "检查 API server 测试失败");
    }

    #[test]
    fn uses_claude_transcript_path_title_without_session_id() {
        let home = test_home("claude-transcript-title");
        let transcript_path =
            home.join(".claude/projects/Users-spf-project-agent-island/session-file.jsonl");
        fs::create_dir_all(transcript_path.parent().unwrap()).unwrap();
        fs::write(
            &transcript_path,
            r#"{"type":"summary","summary":"修复 Claude 任务标题展示"}"#,
        )
        .unwrap();

        let mut hook_event = event("UserPromptSubmit", "2026-05-30T01:00:00Z");
        hook_event.source = AgentSource::ClaudeCode;
        hook_event.session_id = None;
        hook_event.transcript_path = Some(transcript_path.to_string_lossy().to_string());
        let tasks = tasks_from_events_with_home(vec![hook_event], Some(&home));

        assert_eq!(tasks[0].title, "修复 Claude 任务标题展示");
    }

    #[test]
    fn uses_claude_ai_title_from_transcript_path() {
        let home = test_home("claude-ai-title");
        let transcript_path =
            home.join(".claude/projects/Users-spf-project-agent-island/session-file.jsonl");
        fs::create_dir_all(transcript_path.parent().unwrap()).unwrap();
        fs::write(
            &transcript_path,
            r#"{"type":"ai-title","aiTitle":"调研 Claude Code 任务标题"}"#,
        )
        .unwrap();

        let mut hook_event = event("UserPromptSubmit", "2026-05-30T01:00:00Z");
        hook_event.source = AgentSource::ClaudeCode;
        hook_event.session_id = Some("claude-session".to_string());
        hook_event.transcript_path = Some(transcript_path.to_string_lossy().to_string());
        let tasks = tasks_from_events_with_home(vec![hook_event], Some(&home));

        assert_eq!(tasks[0].title, "调研 Claude Code 任务标题");
    }

    #[test]
    fn uses_latest_claude_ai_title_from_transcript_path() {
        let home = test_home("claude-latest-ai-title");
        let transcript_path =
            home.join(".claude/projects/Users-spf-project-agent-island/session-file.jsonl");
        fs::create_dir_all(transcript_path.parent().unwrap()).unwrap();
        fs::write(
            &transcript_path,
            [
                r#"{"type":"ai-title","aiTitle":"早期标题"}"#,
                r#"{"type":"ai-title","aiTitle":"最终标题"}"#,
            ]
            .join("\n"),
        )
        .unwrap();

        let mut hook_event = event("UserPromptSubmit", "2026-05-30T01:00:00Z");
        hook_event.source = AgentSource::ClaudeCode;
        hook_event.transcript_path = Some(transcript_path.to_string_lossy().to_string());
        let tasks = tasks_from_events_with_home(vec![hook_event], Some(&home));

        assert_eq!(tasks[0].title, "最终标题");
    }

    #[test]
    fn falls_back_to_project_directory_without_session_title() {
        let tasks = tasks_from_events_with_home(
            vec![event("UserPromptSubmit", "2026-05-30T01:00:00Z")],
            Some(&test_home("fallback-title")),
        );

        assert_eq!(tasks[0].title, "Codex · agent-island");
    }

    #[test]
    fn ignores_unidentified_hook_events() {
        let mut hook_event = event("HookEvent", "2026-05-30T01:00:00Z");
        hook_event.session_key = "61e780e9b0bca7ff".to_string();
        hook_event.cwd = None;
        hook_event.tool_name = None;

        let tasks = tasks_from_events_with_home(vec![hook_event], Some(&test_home("empty-hook")));

        assert!(tasks.is_empty());
    }

    fn test_home(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "agent-island-hook-ingest-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }
}
