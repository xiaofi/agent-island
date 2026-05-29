use std::{fs, path::PathBuf};

use crate::{
    adapters::types::{now_iso, AdapterDiagnostic, AgentSource, CandidatePath, DiagnosticStatus},
    services::process_scan::scan_processes,
};

pub fn discover() -> AdapterDiagnostic {
    let processes = scan_processes(&["codex"]);
    let candidate_paths = vec![
        candidate_path("~/.codex"),
        candidate_path("~/Library/Application Support/Codex"),
        candidate_path("~/Library/Logs/Codex"),
    ];
    let readable_count = candidate_paths.iter().filter(|path| path.readable).count();
    let parsed_sessions = usize::from(!processes.is_empty() || readable_count > 0);
    let status = if parsed_sessions > 0 {
        DiagnosticStatus::Partial
    } else {
        DiagnosticStatus::Unavailable
    };

    AdapterDiagnostic {
        source: AgentSource::Codex,
        status,
        summary: if parsed_sessions > 0 {
            "发现 Codex 相关进程或候选数据源；MVP 先以降级任务展示，后续接入事件解析。".to_string()
        } else {
            "没有发现可用 Codex 进程或本地数据源。".to_string()
        },
        processes,
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
                reason: if readable { None } else { Some("路径存在但不可读".to_string()) },
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
