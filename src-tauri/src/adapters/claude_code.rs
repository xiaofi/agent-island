use std::{fs, path::PathBuf};

use crate::adapters::types::{
    now_iso, AdapterDiagnostic, AgentSource, CandidatePath, DiagnosticStatus,
};

pub fn discover() -> AdapterDiagnostic {
    let candidate_paths = vec![
        candidate_path("~/.claude/settings.json"),
        candidate_path("~/.claude/settings.local.json"),
    ];
    let readable_count = candidate_paths.iter().filter(|path| path.readable).count();
    let parsed_sessions = readable_count;
    let status = if parsed_sessions > 0 {
        DiagnosticStatus::Partial
    } else {
        DiagnosticStatus::Unavailable
    };

    AdapterDiagnostic {
        source: AgentSource::ClaudeCode,
        status,
        summary: if parsed_sessions > 0 {
            "发现 Claude Code 配置文件；可安装或验证 Agent Island hook 接入。".to_string()
        } else {
            "没有发现 Claude Code 配置文件。".to_string()
        },
        processes: Vec::new(),
        candidate_paths,
        parsed_sessions,
        updated_at: now_iso(),
    }
}

fn candidate_path(path: &str) -> CandidatePath {
    let expanded = expand_home(path);
    let metadata = fs::metadata(&expanded);

    match metadata {
        Ok(metadata) => {
            let readable = if metadata.is_dir() {
                fs::read_dir(&expanded).is_ok()
            } else {
                fs::File::open(&expanded).is_ok()
            };

            CandidatePath {
                path: path.to_string(),
                exists: true,
                readable,
                reason: if readable {
                    None
                } else {
                    Some("路径存在但不可读".to_string())
                },
                updated_at: metadata.modified().ok().map(|modified| {
                    let datetime: chrono::DateTime<chrono::Utc> = modified.into();
                    datetime.to_rfc3339()
                }),
            }
        }
        Err(error) => CandidatePath {
            path: path.to_string(),
            exists: false,
            readable: false,
            reason: Some(error.to_string()),
            updated_at: None,
        },
    }
}

fn expand_home(path: &str) -> PathBuf {
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(stripped);
        }
    }

    PathBuf::from(path)
}
