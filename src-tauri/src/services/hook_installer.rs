use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{
    adapters::types::{now_iso, AgentSource, HookOperation, HookOperationError},
    services::config_store,
};

const CODEX_EVENTS: &[&str] = &[
    "SessionStart",
    "UserPromptSubmit",
    "PreToolUse",
    "PermissionRequest",
    "PostToolUse",
    "SubagentStart",
    "SubagentStop",
    "Stop",
];

const CLAUDE_EVENTS: &[&str] = &[
    "SessionStart",
    "UserPromptSubmit",
    "PreToolUse",
    "PermissionRequest",
    "PostToolUse",
    "PostToolUseFailure",
    "Notification",
    "Stop",
    "StopFailure",
    "SessionEnd",
    "CwdChanged",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HookManifest {
    version: u32,
    entries: BTreeMap<String, HookManifestEntry>,
    last_errors: BTreeMap<String, HookOperationError>,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HookManifestEntry {
    source: AgentSource,
    target_path: String,
    command: String,
    installed_at: String,
    updated_at: String,
}

impl Default for HookManifest {
    fn default() -> Self {
        Self {
            version: 1,
            entries: BTreeMap::new(),
            last_errors: BTreeMap::new(),
            updated_at: now_iso(),
        }
    }
}

pub fn install_source(source: &AgentSource) -> Result<(), HookInstallError> {
    let command = ensure_helper_command(source)?;
    let target_path = target_config_path(source)?;
    let events = source_events(source)?;
    let mut root = read_json_object_or_empty(&target_path)?;

    let hooks = root
        .as_object_mut()
        .and_then(|object| object.entry("hooks").or_insert_with(|| json!({})).as_object_mut())
        .ok_or_else(|| HookInstallError::new("invalid-hooks", "hooks 字段不是 JSON object"))?;

    for event_name in events {
        let event_value = hooks.entry((*event_name).to_string()).or_insert_with(|| json!([]));
        let groups = event_value
            .as_array_mut()
            .ok_or_else(|| HookInstallError::new("invalid-event-hooks", "hook event 字段不是 JSON array"))?;

        if groups.iter().any(|group| group_contains_command(group, &command)) {
            continue;
        }

        groups.push(json!({
            "matcher": matcher_for_event(event_name),
            "hooks": [
                {
                    "type": "command",
                    "command": command,
                    "timeout": 1
                }
            ]
        }));
    }

    backup_if_exists(&target_path)?;
    write_json_atomic(&target_path, &root)?;
    update_manifest_entry(source, &target_path, &command)?;
    Ok(())
}

pub fn uninstall_source(source: &AgentSource) -> Result<(), HookInstallError> {
    let command = helper_command(source)?;
    let target_path = target_config_path(source)?;

    if !target_path.exists() {
        remove_manifest_entry(source)?;
        return Ok(());
    }

    let mut root = read_json_object_or_empty(&target_path)?;
    let Some(hooks) = root.get_mut("hooks").and_then(Value::as_object_mut) else {
        remove_manifest_entry(source)?;
        return Ok(());
    };

    let event_names: Vec<String> = hooks.keys().cloned().collect();
    for event_name in event_names {
        let Some(groups) = hooks.get_mut(&event_name).and_then(Value::as_array_mut) else {
            continue;
        };

        for group in groups.iter_mut() {
            if let Some(hooks_array) = group.get_mut("hooks").and_then(Value::as_array_mut) {
                hooks_array.retain(|hook| !hook_matches_command(hook, &command));
            }
        }

        groups.retain(|group| {
            group
                .get("hooks")
                .and_then(Value::as_array)
                .map(|hooks| !hooks.is_empty())
                .unwrap_or(true)
        });

        if groups.is_empty() {
            hooks.remove(&event_name);
        }
    }

    backup_if_exists(&target_path)?;
    write_json_atomic(&target_path, &root)?;
    remove_manifest_entry(source)?;
    Ok(())
}

pub fn self_test_source(source: &AgentSource) -> Result<(), HookInstallError> {
    let helper_path = helper_path()?;

    if !helper_path.exists() {
        return Err(HookInstallError::new("helper-missing", "Agent Island hook helper 不存在"));
    }

    let metadata = fs::metadata(&helper_path)
        .map_err(|error| HookInstallError::new("helper-stat-failed", error.to_string()))?;

    if !metadata.is_file() {
        return Err(HookInstallError::new("helper-invalid", "Agent Island hook helper 不是文件"));
    }

    let target_path = target_config_path(source)?;
    if !target_path.exists() {
        return Err(HookInstallError::new("config-missing", "目标 hook 配置文件不存在"));
    }

    let command = helper_command(source)?;
    let root = read_json_object_or_empty(&target_path)?;
    let installed = root
        .get("hooks")
        .and_then(Value::as_object)
        .map(|hooks| hooks.values().any(|event| event.as_array().is_some_and(|groups| {
            groups.iter().any(|group| group_contains_command(group, &command))
        })))
        .unwrap_or(false);

    if !installed {
        return Err(HookInstallError::new(
            "command-missing",
            "目标配置中没有 Agent Island hook command",
        ));
    }

    Ok(())
}

pub fn persist_manifest_error(source: &AgentSource, error: &HookOperationError) -> Result<(), String> {
    let mut manifest = load_manifest();
    manifest.last_errors.insert(source_key(source).to_string(), error.clone());
    manifest.updated_at = now_iso();
    save_manifest(&manifest)
}

pub fn clear_manifest_error(source: &AgentSource) -> Result<(), String> {
    let mut manifest = load_manifest();
    manifest.last_errors.remove(source_key(source));
    manifest.updated_at = now_iso();
    save_manifest(&manifest)
}

pub fn operation_error(operation: HookOperation, error: HookInstallError) -> HookOperationError {
    HookOperationError {
        retry_action: operation.clone(),
        operation,
        code: error.code,
        message: error.message,
        occurred_at: now_iso(),
    }
}

fn ensure_helper_command(source: &AgentSource) -> Result<String, HookInstallError> {
    let path = helper_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| HookInstallError::new("helper-dir-failed", error.to_string()))?;
    }

    if !path.exists() {
        let script = "#!/bin/sh\ncat >/dev/null 2>/dev/null || true\nexit 0\n";
        fs::write(&path, script).map_err(|error| HookInstallError::new("helper-write-failed", error.to_string()))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&path)
                .map_err(|error| HookInstallError::new("helper-stat-failed", error.to_string()))?
                .permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&path, permissions)
                .map_err(|error| HookInstallError::new("helper-chmod-failed", error.to_string()))?;
        }
    }

    helper_command(source)
}

fn helper_command(source: &AgentSource) -> Result<String, HookInstallError> {
    let path = helper_path()?;
    let source_arg = match source {
        AgentSource::Codex => "codex",
        AgentSource::ClaudeCode => "claude-code",
        AgentSource::Manual => return Err(HookInstallError::new("unsupported-source", "manual 不支持 hook 接入")),
    };

    Ok(format!("\"{}\" --source {}", path.display(), source_arg))
}

fn helper_path() -> Result<PathBuf, HookInstallError> {
    let app_dir = config_store::app_support_dir()
        .ok_or_else(|| HookInstallError::new("home-missing", "无法定位用户 HOME"))?;
    Ok(app_dir.join("hooks/agent-island-hook"))
}

fn target_config_path(source: &AgentSource) -> Result<PathBuf, HookInstallError> {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| HookInstallError::new("home-missing", "无法定位用户 HOME"))?;

    match source {
        AgentSource::Codex => Ok(home.join(".codex/hooks.json")),
        AgentSource::ClaudeCode => Ok(home.join(".claude/settings.json")),
        AgentSource::Manual => Err(HookInstallError::new("unsupported-source", "manual 不支持 hook 接入")),
    }
}

fn source_events(source: &AgentSource) -> Result<&'static [&'static str], HookInstallError> {
    match source {
        AgentSource::Codex => Ok(CODEX_EVENTS),
        AgentSource::ClaudeCode => Ok(CLAUDE_EVENTS),
        AgentSource::Manual => Err(HookInstallError::new("unsupported-source", "manual 不支持 hook 接入")),
    }
}

fn matcher_for_event(event_name: &str) -> &str {
    if event_name == "SessionStart" {
        "startup|resume|clear|compact"
    } else {
        "*"
    }
}

fn read_json_object_or_empty(path: &Path) -> Result<Value, HookInstallError> {
    if !path.exists() {
        return Ok(json!({}));
    }

    let contents = fs::read_to_string(path).map_err(|error| HookInstallError::new("config-read-failed", error.to_string()))?;
    if contents.trim().is_empty() {
        return Ok(json!({}));
    }

    let value: Value =
        serde_json::from_str(&contents).map_err(|error| HookInstallError::new("config-parse-failed", error.to_string()))?;
    if !value.is_object() {
        return Err(HookInstallError::new("config-not-object", "目标配置不是 JSON object"));
    }

    Ok(value)
}

fn write_json_atomic(path: &Path, value: &Value) -> Result<(), HookInstallError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| HookInstallError::new("config-dir-failed", error.to_string()))?;
    }

    let temp_path = path.with_extension("agent-island-tmp");
    let contents = serde_json::to_string_pretty(value)
        .map_err(|error| HookInstallError::new("config-serialize-failed", error.to_string()))?;
    fs::write(&temp_path, contents).map_err(|error| HookInstallError::new("config-write-failed", error.to_string()))?;
    fs::rename(&temp_path, path).map_err(|error| HookInstallError::new("config-rename-failed", error.to_string()))
}

fn backup_if_exists(path: &Path) -> Result<(), HookInstallError> {
    if !path.exists() {
        return Ok(());
    }

    let app_dir = config_store::app_support_dir()
        .ok_or_else(|| HookInstallError::new("home-missing", "无法定位用户 HOME"))?;
    let backup_dir = app_dir.join("backups");
    fs::create_dir_all(&backup_dir).map_err(|error| HookInstallError::new("backup-dir-failed", error.to_string()))?;

    let file_name = path.file_name().and_then(|name| name.to_str()).unwrap_or("hook-config");
    let backup_path = backup_dir.join(format!("{}-{}.json", file_name, chrono::Utc::now().format("%Y%m%dT%H%M%SZ")));
    fs::copy(path, backup_path).map_err(|error| HookInstallError::new("backup-copy-failed", error.to_string()))?;
    Ok(())
}

fn group_contains_command(group: &Value, command: &str) -> bool {
    group
        .get("hooks")
        .and_then(Value::as_array)
        .is_some_and(|hooks| hooks.iter().any(|hook| hook_matches_command(hook, command)))
}

fn hook_matches_command(hook: &Value, command: &str) -> bool {
    hook.get("command").and_then(Value::as_str) == Some(command)
}

fn update_manifest_entry(source: &AgentSource, target_path: &Path, command: &str) -> Result<(), HookInstallError> {
    let mut manifest = load_manifest();
    let now = now_iso();
    manifest.entries.insert(
        source_key(source).to_string(),
        HookManifestEntry {
            source: source.clone(),
            target_path: target_path.display().to_string(),
            command: command.to_string(),
            installed_at: now.clone(),
            updated_at: now.clone(),
        },
    );
    manifest.last_errors.remove(source_key(source));
    manifest.updated_at = now;
    save_manifest(&manifest).map_err(|error| HookInstallError::new("manifest-write-failed", error))
}

fn remove_manifest_entry(source: &AgentSource) -> Result<(), HookInstallError> {
    let mut manifest = load_manifest();
    manifest.entries.remove(source_key(source));
    manifest.last_errors.remove(source_key(source));
    manifest.updated_at = now_iso();
    save_manifest(&manifest).map_err(|error| HookInstallError::new("manifest-write-failed", error))
}

fn load_manifest() -> HookManifest {
    let Some(path) = manifest_path() else {
        return HookManifest::default();
    };
    let Ok(contents) = fs::read_to_string(path) else {
        return HookManifest::default();
    };
    serde_json::from_str(&contents).unwrap_or_default()
}

fn save_manifest(manifest: &HookManifest) -> Result<(), String> {
    let Some(path) = manifest_path() else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let contents = serde_json::to_string_pretty(manifest).map_err(|error| error.to_string())?;
    fs::write(path, contents).map_err(|error| error.to_string())
}

fn manifest_path() -> Option<PathBuf> {
    config_store::app_support_dir().map(|dir| dir.join("hooks/install-manifest.json"))
}

fn source_key(source: &AgentSource) -> &'static str {
    match source {
        AgentSource::Codex => "codex",
        AgentSource::ClaudeCode => "claudeCode",
        AgentSource::Manual => "manual",
    }
}

#[derive(Debug)]
pub struct HookInstallError {
    pub code: String,
    pub message: String,
}

impl HookInstallError {
    fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}
