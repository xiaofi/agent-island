use crate::{
    adapters::{
        claude_code, codex,
        types::{
            AdapterDiagnostic, AgentEvent, AgentEventType, AgentSource, AgentTask, CandidatePath,
            TaskStatus,
        },
    },
    aggregator::sort_tasks_by_updated_at,
    services::{app_open, config_store, hook_ingest},
};

#[tauri::command]
pub async fn get_tasks() -> Result<Vec<AgentTask>, String> {
    let settings = config_store::load_settings();
    let mut tasks = hook_ingest::load_tasks(&settings);
    let has_codex_hook_task = tasks.iter().any(|task| task.source == AgentSource::Codex);
    let has_claude_hook_task = tasks
        .iter()
        .any(|task| task.source == AgentSource::ClaudeCode);

    if settings.enabled_adapters.contains(&AgentSource::Codex) && !has_codex_hook_task {
        tasks.extend(tasks_from_diagnostic(
            codex::discover(),
            settings.hook_source.codex,
        ));
    }

    if settings.enabled_adapters.contains(&AgentSource::ClaudeCode) && !has_claude_hook_task {
        tasks.extend(tasks_from_diagnostic(
            claude_code::discover(),
            settings.hook_source.claude_code,
        ));
    }

    Ok(sort_tasks_by_updated_at(tasks))
}

#[tauri::command]
pub async fn open_task(task_id: String) -> Result<(), String> {
    println!("open_task requested for {task_id}");
    Ok(())
}

#[tauri::command]
pub async fn open_workdir(path: String) -> Result<(), String> {
    app_open::open_path(&path)
}

#[tauri::command]
pub async fn copy_task_summary(task_id: String) -> Result<(), String> {
    println!("copy_task_summary requested for {task_id}");
    Ok(())
}

fn tasks_from_diagnostic(diagnostic: AdapterDiagnostic, hook_enabled: bool) -> Vec<AgentTask> {
    diagnostic
        .candidate_paths
        .iter()
        .find(|path| path.readable)
        .map(|candidate_path| {
            vec![task_from_candidate_path(
                &diagnostic,
                candidate_path,
                hook_enabled,
            )]
        })
        .unwrap_or_default()
}

fn task_from_candidate_path(
    diagnostic: &AdapterDiagnostic,
    candidate_path: &CandidatePath,
    hook_enabled: bool,
) -> AgentTask {
    let source_label = source_label(&diagnostic.source);
    let source_slug = source_slug(&diagnostic.source);
    let updated_at = candidate_path
        .updated_at
        .as_ref()
        .unwrap_or(&diagnostic.updated_at);
    let summary = if hook_enabled {
        format!("已接入 {source_label}，等待 hook 状态事件")
    } else {
        format!("发现 {source_label} 配置文件；打开状态采集后可接收 hook 状态")
    };
    let waiting_reason = if hook_enabled {
        "已安装 hook command；当前还没有收到可展示的 hook 状态事件。"
    } else {
        "真实会话状态需要在设置中打开对应来源的 hook 接入；当前仅根据配置文件展示可接入状态。"
    };

    AgentTask {
        id: format!("{source_slug}-candidate-source"),
        source: diagnostic.source.clone(),
        title: if hook_enabled {
            "等待 hook 状态事件".to_string()
        } else {
            "等待接入状态采集".to_string()
        },
        cwd: None,
        status: TaskStatus::Discovering,
        started_at: None,
        updated_at: updated_at.clone(),
        duration_seconds: None,
        last_action: Some(summary.clone()),
        waiting_reason: Some(waiting_reason.to_string()),
        error_summary: None,
        window_hint: None,
        events: vec![discovery_event(
            &format!("{source_slug}-candidate-source"),
            updated_at,
            summary,
        )],
    }
}

fn discovery_event(task_id: &str, timestamp: &str, summary: String) -> AgentEvent {
    AgentEvent {
        id: format!("{task_id}-discovery"),
        task_id: task_id.to_string(),
        r#type: AgentEventType::Heartbeat,
        timestamp: timestamp.to_string(),
        summary,
        metadata: None,
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
