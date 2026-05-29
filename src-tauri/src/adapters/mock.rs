use crate::adapters::types::{now_iso, AgentEvent, AgentEventType, AgentSource, AgentTask, TaskStatus, WindowHint};

pub fn mock_tasks() -> Vec<AgentTask> {
    let now = now_iso();

    vec![
        AgentTask {
            id: "codex-agent-island".to_string(),
            source: AgentSource::Codex,
            title: "实现 Agent Island MVP".to_string(),
            cwd: Some("/Users/spf/project/agent-island".to_string()),
            status: TaskStatus::ToolRunning,
            started_at: Some(now.clone()),
            updated_at: now.clone(),
            duration_seconds: None,
            last_action: Some("读取 mock adapter 数据".to_string()),
            waiting_reason: None,
            error_summary: None,
            window_hint: Some(WindowHint {
                app_name: Some("Codex".to_string()),
                process_id: None,
                window_title: Some("agent-island".to_string()),
            }),
            events: vec![
                event("codex-agent-island", AgentEventType::SessionStarted, "启动 Codex 会话"),
                event("codex-agent-island", AgentEventType::ToolStarted, "加载 Tauri command"),
            ],
        },
        AgentTask {
            id: "claude-api-server".to_string(),
            source: AgentSource::ClaudeCode,
            title: "检查 API server 测试失败".to_string(),
            cwd: Some("/Users/spf/project/api-server".to_string()),
            status: TaskStatus::WaitingUser,
            started_at: Some(now.clone()),
            updated_at: now.clone(),
            duration_seconds: None,
            last_action: Some("等待命令确认".to_string()),
            waiting_reason: Some("需要批准运行数据库迁移测试".to_string()),
            error_summary: None,
            window_hint: Some(WindowHint {
                app_name: Some("Terminal".to_string()),
                process_id: None,
                window_title: Some("claude api-server".to_string()),
            }),
            events: vec![
                event("claude-api-server", AgentEventType::SessionStarted, "启动 Claude Code 会话"),
                event("claude-api-server", AgentEventType::WaitingForUser, "等待用户确认命令"),
            ],
        },
        AgentTask {
            id: "manual-release-check".to_string(),
            source: AgentSource::Manual,
            title: "打包前检查清单".to_string(),
            cwd: Some("/Users/spf/project/agent-island".to_string()),
            status: TaskStatus::Completed,
            started_at: Some(now.clone()),
            updated_at: now,
            duration_seconds: None,
            last_action: Some("mock 数据验证完成".to_string()),
            waiting_reason: None,
            error_summary: None,
            window_hint: None,
            events: vec![event(
                "manual-release-check",
                AgentEventType::SessionCompleted,
                "检查完成",
            )],
        },
    ]
}

fn event(task_id: &str, r#type: AgentEventType, summary: &str) -> AgentEvent {
    AgentEvent {
        id: format!("{task_id}-{}", summary.replace(' ', "-")),
        task_id: task_id.to_string(),
        r#type,
        timestamp: now_iso(),
        summary: summary.to_string(),
        metadata: None,
    }
}
