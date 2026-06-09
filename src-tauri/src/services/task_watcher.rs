use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use tauri::{AppHandle, Emitter};

use crate::{
    adapters::types::AgentTask,
    services::{config_store, hook_ingest},
};

struct WatcherState {
    tasks: HashMap<String, AgentTask>,
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
        sync_and_emit(&app_handle, &state);

        while let Ok(result) = rx.recv() {
            if let Ok(event) = result {
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

                sync_and_emit(&app_handle, &state);
            }
        }
    });

    Some(TaskWatcherHandle { watcher })
}

fn sync_and_emit(app_handle: &AppHandle, state: &Arc<Mutex<WatcherState>>) {
    let settings = config_store::load_settings();
    let tasks = hook_ingest::load_tasks(&settings);

    let mut guard = match state.lock() {
        Ok(guard) => guard,
        Err(_) => return,
    };

    let new_ids: HashSet<_> = tasks.iter().map(|t| t.id.clone()).collect();
    let old_ids: Vec<_> = guard.tasks.keys().cloned().collect();

    for task in &tasks {
        let should_emit = match guard.tasks.get(&task.id) {
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

    for old_id in old_ids {
        if !new_ids.contains(&old_id) {
            let _ = app_handle.emit("agent-task-removed", old_id);
        }
    }

    guard.tasks = tasks.into_iter().map(|t| (t.id.clone(), t)).collect();
}
