use std::{
    collections::{HashMap, HashSet},
    sync::{mpsc::RecvTimeoutError, Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use chrono::{DateTime, Utc};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter};

use crate::{
    adapters::types::{AgentEventType, AgentTask, TaskStatus},
    services::{config_store, hook_ingest, task_notifications},
};

const PERIODIC_SYNC_INTERVAL: Duration = Duration::from_secs(300);
const EVENT_SPOOL_COMPACTION_INTERVAL: Duration = Duration::from_secs(60 * 60);
const TOOL_RUNNING_STALE_AFTER: chrono::Duration = chrono::Duration::minutes(10);

struct WatcherState {
    tasks: HashMap<String, AgentTask>,
    notifications: task_notifications::TaskNotificationState,
    spool_changed_since_compaction: bool,
    last_spool_compaction: Instant,
}

/// 持有 watcher 所有权，确保应用退出时 notify channel 被断开，
/// watcher 线程中的 rx.recv() 返回 Disconnected 后自然退出。
pub struct TaskWatcherHandle {
    #[allow(dead_code)]
    watcher: RecommendedWatcher,
}

pub fn start_task_watcher(app: &AppHandle) -> Option<TaskWatcherHandle> {
    let app_handle = app.clone();
    let state = Arc::new(Mutex::new(WatcherState {
        tasks: HashMap::new(),
        notifications: task_notifications::TaskNotificationState::default(),
        spool_changed_since_compaction: false,
        last_spool_compaction: Instant::now(),
    }));

    let events_dir = match config_store::app_support_dir() {
        Some(dir) => dir.join("events"),
        None => {
            eprintln!("[task_watcher] app support dir not found");
            return None;
        }
    };

    if let Err(e) = std::fs::create_dir_all(&events_dir) {
        eprintln!("[task_watcher] failed to create events dir: {e}");
        return None;
    }

    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = match notify::recommended_watcher(tx) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("[task_watcher] failed to create watcher: {e}");
            return None;
        }
    };

    if let Err(e) = watcher.watch(&events_dir, RecursiveMode::NonRecursive) {
        eprintln!("[task_watcher] failed to watch events dir: {e}");
        return None;
    }

    thread::spawn(move || {
        // 初始加载一次当前状态
        hook_ingest::compact_event_spool();
        mark_spool_compacted(&state);
        sync_and_emit(&app_handle, &state);

        loop {
            match rx.recv_timeout(PERIODIC_SYNC_INTERVAL) {
                Ok(Ok(event)) => {
                    // 只关心 events 目录下的 .jsonl 文件
                    let relevant = event.paths.iter().any(|p| {
                        p.extension()
                            .and_then(|e| e.to_str())
                            .map(|e| e == "jsonl")
                            .unwrap_or(false)
                    });
                    if !relevant {
                        continue;
                    }

                    // debounce：等待 80ms 让批量写入落盘，并合并队列中积压事件
                    thread::sleep(Duration::from_millis(80));
                    while rx.try_recv().is_ok() {
                        // 丢弃队列中所有待处理事件，只保留最后一次读取
                    }

                    mark_spool_changed(&state);
                    sync_and_emit(&app_handle, &state);
                }
                Ok(Err(error)) => {
                    eprintln!("[task_watcher] watch error: {error}");
                }
                Err(RecvTimeoutError::Timeout) => {
                    let compacted_spool = compact_event_spool_if_due(&state);
                    if compacted_spool || has_task_due_for_stale(&state, Utc::now()) {
                        sync_and_emit(&app_handle, &state);
                    }
                }
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
    });

    Some(TaskWatcherHandle { watcher })
}

fn mark_spool_changed(state: &Arc<Mutex<WatcherState>>) {
    if let Ok(mut guard) = state.lock() {
        guard.spool_changed_since_compaction = true;
    }
}

fn mark_spool_compacted(state: &Arc<Mutex<WatcherState>>) {
    if let Ok(mut guard) = state.lock() {
        guard.spool_changed_since_compaction = false;
        guard.last_spool_compaction = Instant::now();
    }
}

fn compact_event_spool_if_due(state: &Arc<Mutex<WatcherState>>) -> bool {
    let exceeds_size_threshold = hook_ingest::event_spool_exceeds_compaction_size_threshold();
    let should_compact = match state.lock() {
        Ok(guard) => should_compact_event_spool(
            guard.spool_changed_since_compaction,
            guard.last_spool_compaction.elapsed(),
            exceeds_size_threshold,
        ),
        Err(_) => return false,
    };

    if should_compact {
        hook_ingest::compact_event_spool();
        mark_spool_compacted(state);
        return true;
    }

    false
}

fn should_compact_event_spool(
    changed_since_last_compaction: bool,
    elapsed_since_last_compaction: Duration,
    exceeds_size_threshold: bool,
) -> bool {
    changed_since_last_compaction
        && (elapsed_since_last_compaction >= EVENT_SPOOL_COMPACTION_INTERVAL
            || exceeds_size_threshold)
}

fn has_task_due_for_stale(state: &Arc<Mutex<WatcherState>>, now: DateTime<Utc>) -> bool {
    match state.lock() {
        Ok(guard) => guard
            .tasks
            .values()
            .any(|task| task_is_due_for_stale(task, now)),
        Err(_) => false,
    }
}

fn task_is_due_for_stale(task: &AgentTask, now: DateTime<Utc>) -> bool {
    if task.status != TaskStatus::ToolRunning {
        return false;
    }

    let Some(started_at) = latest_tool_started_at(task) else {
        return true;
    };

    now >= started_at + TOOL_RUNNING_STALE_AFTER
}

fn latest_tool_started_at(task: &AgentTask) -> Option<DateTime<Utc>> {
    task.events
        .iter()
        .filter(|event| event.r#type == AgentEventType::ToolStarted)
        .filter_map(|event| parse_timestamp(&event.timestamp))
        .max()
}

fn parse_timestamp(timestamp: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(timestamp)
        .ok()
        .map(|value| value.with_timezone(&Utc))
}

fn sync_and_emit(app_handle: &AppHandle, state: &Arc<Mutex<WatcherState>>) {
    let settings = config_store::load_settings();
    let tasks = hook_ingest::load_tasks_with_fallback(&settings);

    let mut guard = match state.lock() {
        Ok(guard) => guard,
        Err(_) => return,
    };

    let new_ids: HashSet<_> = tasks.iter().map(|t| t.id.clone()).collect();
    let old_ids: Vec<_> = guard.tasks.keys().cloned().collect();
    let previous_tasks = guard.tasks.clone();

    for task in &tasks {
        let should_emit = match previous_tasks.get(&task.id) {
            Some(old) => old.updated_at != task.updated_at,
            None => true,
        };

        if should_emit {
            let _ = app_handle.emit("agent-task-updated", task.clone());
            if let Some(event) = task.events.first() {
                let _ = app_handle.emit("agent-event-created", event.clone());
            }
        }
    }

    task_notifications::notify_task_updates(
        app_handle,
        &mut guard.notifications,
        &settings,
        &tasks,
        &previous_tasks,
    );

    for old_id in old_ids {
        if !new_ids.contains(&old_id) {
            let _ = app_handle.emit("agent-task-removed", old_id);
        }
    }

    guard.tasks = tasks.into_iter().map(|t| (t.id.clone(), t)).collect();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::types::{AgentEvent, AgentSource};

    #[test]
    fn skips_spool_compaction_when_events_have_not_changed() {
        assert!(!should_compact_event_spool(
            false,
            EVENT_SPOOL_COMPACTION_INTERVAL + Duration::from_secs(1),
            true,
        ));
    }

    #[test]
    fn skips_changed_spool_before_interval_when_below_size_threshold() {
        assert!(!should_compact_event_spool(
            true,
            EVENT_SPOOL_COMPACTION_INTERVAL - Duration::from_secs(1),
            false,
        ));
    }

    #[test]
    fn compacts_changed_spool_after_interval() {
        assert!(should_compact_event_spool(
            true,
            EVENT_SPOOL_COMPACTION_INTERVAL,
            false,
        ));
    }

    #[test]
    fn compacts_changed_spool_when_size_threshold_is_reached() {
        assert!(should_compact_event_spool(
            true,
            Duration::from_secs(0),
            true,
        ));
    }

    #[test]
    fn tool_running_task_is_not_due_for_stale_before_timeout() {
        let task = task_with_status_and_events(
            TaskStatus::ToolRunning,
            vec![event(AgentEventType::ToolStarted, "2026-05-30T01:00:00Z")],
        );

        assert!(!task_is_due_for_stale(&task, utc("2026-05-30T01:09:59Z"),));
    }

    #[test]
    fn tool_running_task_is_due_for_stale_after_timeout() {
        let task = task_with_status_and_events(
            TaskStatus::ToolRunning,
            vec![
                event(AgentEventType::Heartbeat, "2026-05-30T01:05:00Z"),
                event(AgentEventType::ToolStarted, "2026-05-30T01:00:00Z"),
            ],
        );

        assert!(task_is_due_for_stale(&task, utc("2026-05-30T01:10:00Z")));
    }

    #[test]
    fn non_tool_running_task_is_not_due_for_stale() {
        let task = task_with_status_and_events(
            TaskStatus::Stale,
            vec![event(AgentEventType::ToolStarted, "2026-05-30T01:00:00Z")],
        );

        assert!(!task_is_due_for_stale(&task, utc("2026-05-30T01:20:00Z"),));
    }

    #[test]
    fn tool_running_task_without_parseable_start_time_is_due_for_stale_conservatively() {
        let task = task_with_status_and_events(
            TaskStatus::ToolRunning,
            vec![event(AgentEventType::ToolStarted, "not-a-timestamp")],
        );

        assert!(task_is_due_for_stale(&task, utc("2026-05-30T01:20:00Z")));
    }

    fn task_with_status_and_events(status: TaskStatus, events: Vec<AgentEvent>) -> AgentTask {
        AgentTask {
            id: "hook-codex-session".to_string(),
            source: AgentSource::Codex,
            title: "agent-island".to_string(),
            cwd: Some("/Users/spf/project/agent-island".to_string()),
            status,
            started_at: Some("2026-05-30T01:00:00Z".to_string()),
            updated_at: "2026-05-30T01:00:00Z".to_string(),
            duration_seconds: None,
            last_action: Some("正在运行 Bash".to_string()),
            waiting_reason: None,
            error_summary: None,
            window_hint: None,
            events,
        }
    }

    fn event(r#type: AgentEventType, timestamp: &str) -> AgentEvent {
        AgentEvent {
            id: format!("event-{timestamp}"),
            task_id: "hook-codex-session".to_string(),
            r#type,
            timestamp: timestamp.to_string(),
            summary: "event".to_string(),
            metadata: None,
        }
    }

    fn utc(timestamp: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(timestamp)
            .unwrap()
            .with_timezone(&Utc)
    }
}
