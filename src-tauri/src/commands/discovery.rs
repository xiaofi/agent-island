use crate::adapters::{
    claude_code,
    codex,
    types::{AdapterDiagnostic, AgentSource},
};

#[tauri::command]
pub async fn run_discovery(source: Option<AgentSource>) -> Result<Vec<AdapterDiagnostic>, String> {
    let diagnostics = match source {
        Some(AgentSource::Codex) => vec![codex::discover()],
        Some(AgentSource::ClaudeCode) => vec![claude_code::discover()],
        Some(AgentSource::Manual) => Vec::new(),
        None => vec![codex::discover(), claude_code::discover()],
    };

    Ok(diagnostics)
}
