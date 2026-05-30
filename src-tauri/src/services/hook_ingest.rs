use std::{collections::BTreeMap, fs};

use serde::Deserialize;

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
    session_key: String,
    cwd: Option<String>,
    timestamp: String,
    tool_name: Option<String>,
    action_summary: Option<String>,
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
    let mut grouped: BTreeMap<(AgentSource, String), Vec<SpoolEvent>> = BTreeMap::new();

    for event in events {
        grouped
            .entry((event.source.clone(), event.session_key.clone()))
            .or_default()
            .push(event);
    }

    let tasks = grouped
        .into_iter()
        .filter_map(|((source, session_key), mut events)| {
            events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
            task_from_events(source, session_key, events)
        })
        .collect();

    sort_tasks_by_updated_at(tasks)
}

fn task_from_events(
    source: AgentSource,
    session_key: String,
    spool_events: Vec<SpoolEvent>,
) -> Option<AgentTask> {
    let first = spool_events.first()?;
    let last = spool_events.last()?;
    let task_id = format!("hook-{}-{}", source_slug(&source), session_key);
    let status = status_for_event(&last.event);
    let cwd = latest_cwd(&spool_events);
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
        title: task_title(&source, cwd.as_deref()),
        cwd,
        status,
        started_at: Some(first.timestamp.clone()),
        updated_at: last.timestamp.clone(),
        duration_seconds: None,
        last_action: Some(summary_for_event(last)),
        waiting_reason: waiting_reason_for_event(&last.event).map(str::to_string),
        error_summary: error_summary_for_event(&last.event).map(str::to_string),
        window_hint: None,
        events,
    })
}

fn agent_event(task_id: &str, index: usize, event: &SpoolEvent) -> AgentEvent {
    AgentEvent {
        id: format!("{task_id}-{index}"),
        task_id: task_id.to_string(),
        r#type: event_type_for_event(&event.event),
        timestamp: event.timestamp.clone(),
        summary: summary_for_event(event),
        metadata: event.tool_name.as_ref().map(|tool_name| {
            serde_json::json!({
                "toolName": tool_name,
                "hookEvent": event.event,
            })
        }),
    }
}

fn latest_cwd(events: &[SpoolEvent]) -> Option<String> {
    events
        .iter()
        .rev()
        .find_map(|event| event.cwd.as_ref().filter(|cwd| !cwd.is_empty()).cloned())
}

fn event_type_for_event(event: &str) -> AgentEventType {
    match event {
        "SessionStart" | "SubagentStart" => AgentEventType::SessionStarted,
        "UserPromptSubmit" => AgentEventType::UserMessage,
        "PreToolUse" => AgentEventType::ToolStarted,
        "PostToolUse" | "SubagentStop" => AgentEventType::ToolFinished,
        "PermissionRequest" => AgentEventType::WaitingForUser,
        "Stop" | "SessionEnd" => AgentEventType::SessionCompleted,
        "PostToolUseFailure" | "StopFailure" => AgentEventType::SessionFailed,
        _ => AgentEventType::Heartbeat,
    }
}

fn status_for_event(event: &str) -> TaskStatus {
    match event {
        "SessionStart" | "SubagentStart" => TaskStatus::Running,
        "UserPromptSubmit" => TaskStatus::Thinking,
        "PreToolUse" => TaskStatus::ToolRunning,
        "PermissionRequest" => TaskStatus::WaitingUser,
        "PostToolUse" | "SubagentStop" => TaskStatus::Thinking,
        "Stop" | "SessionEnd" => TaskStatus::Completed,
        "PostToolUseFailure" | "StopFailure" => TaskStatus::Failed,
        _ => TaskStatus::Running,
    }
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
        "PostToolUse" => format!("{tool} 完成"),
        "PostToolUseFailure" => format!("{tool} 执行失败"),
        "Stop" | "SessionEnd" => "会话本轮完成".to_string(),
        "SubagentStart" => "子任务启动".to_string(),
        "SubagentStop" => "子任务完成".to_string(),
        "CwdChanged" => "工作目录已更新".to_string(),
        _ => "收到状态事件".to_string(),
    }
}

fn waiting_reason_for_event(event: &str) -> Option<&'static str> {
    match event {
        "PermissionRequest" => Some("Codex / Claude Code 正在等待用户确认。"),
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

fn task_title(source: &AgentSource, cwd: Option<&str>) -> String {
    let source_label = source_label(source);
    let project = cwd
        .and_then(|path| path.rsplit('/').find(|part| !part.is_empty()))
        .unwrap_or("会话");
    format!("{source_label} · {project}")
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
            session_key: "abc123".to_string(),
            cwd: Some("/Users/spf/project/agent-island".to_string()),
            timestamp: timestamp.to_string(),
            tool_name: Some("Bash".to_string()),
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
    fn groups_events_by_session() {
        let mut other = event("PermissionRequest", "2026-05-30T01:00:05Z");
        other.session_key = "other".to_string();

        let tasks = tasks_from_events(vec![event("SessionStart", "2026-05-30T01:00:00Z"), other]);

        assert_eq!(tasks.len(), 2);
    }
}
