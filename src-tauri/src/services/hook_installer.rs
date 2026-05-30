use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
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
        .and_then(|object| {
            object
                .entry("hooks")
                .or_insert_with(|| json!({}))
                .as_object_mut()
        })
        .ok_or_else(|| HookInstallError::new("invalid-hooks", "hooks 字段不是 JSON object"))?;

    for event_name in events {
        let event_value = hooks
            .entry((*event_name).to_string())
            .or_insert_with(|| json!([]));
        let groups = event_value.as_array_mut().ok_or_else(|| {
            HookInstallError::new("invalid-event-hooks", "hook event 字段不是 JSON array")
        })?;

        if groups
            .iter()
            .any(|group| group_contains_command(group, &command))
        {
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
    refresh_helper_script()?;
    let helper_path = helper_path()?;

    if !helper_path.exists() {
        return Err(HookInstallError::new(
            "helper-missing",
            "Agent Island hook helper 不存在",
        ));
    }

    let metadata = fs::metadata(&helper_path)
        .map_err(|error| HookInstallError::new("helper-stat-failed", error.to_string()))?;

    if !metadata.is_file() {
        return Err(HookInstallError::new(
            "helper-invalid",
            "Agent Island hook helper 不是文件",
        ));
    }

    let target_path = target_config_path(source)?;
    if !target_path.exists() {
        return Err(HookInstallError::new(
            "config-missing",
            "目标 hook 配置文件不存在",
        ));
    }

    let command = helper_command(source)?;
    let root = read_json_object_or_empty(&target_path)?;
    let installed = root
        .get("hooks")
        .and_then(Value::as_object)
        .map(|hooks| {
            hooks.values().any(|event| {
                event.as_array().is_some_and(|groups| {
                    groups
                        .iter()
                        .any(|group| group_contains_command(group, &command))
                })
            })
        })
        .unwrap_or(false);

    if !installed {
        return Err(HookInstallError::new(
            "command-missing",
            "目标配置中没有 Agent Island hook command",
        ));
    }

    let status = Command::new(&helper_path)
        .arg("--source")
        .arg(source_arg(source)?)
        .arg("--self-test")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| HookInstallError::new("helper-self-test-failed", error.to_string()))?;

    if !status.success() {
        return Err(HookInstallError::new(
            "helper-self-test-failed",
            "Agent Island hook helper 自检失败",
        ));
    }

    Ok(())
}

pub fn persist_manifest_error(
    source: &AgentSource,
    error: &HookOperationError,
) -> Result<(), String> {
    let mut manifest = load_manifest();
    manifest
        .last_errors
        .insert(source_key(source).to_string(), error.clone());
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

pub fn refresh_helper_script() -> Result<(), HookInstallError> {
    let path = helper_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| HookInstallError::new("helper-dir-failed", error.to_string()))?;
    }

    let script = helper_script()?;
    let should_write = fs::read_to_string(&path)
        .map(|contents| contents != script)
        .unwrap_or(true);

    if should_write {
        fs::write(&path, script)
            .map_err(|error| HookInstallError::new("helper-write-failed", error.to_string()))?;
    }

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

    Ok(())
}

fn ensure_helper_command(source: &AgentSource) -> Result<String, HookInstallError> {
    refresh_helper_script()?;
    helper_command(source)
}

fn helper_command(source: &AgentSource) -> Result<String, HookInstallError> {
    let path = helper_path()?;
    Ok(format!(
        "\"{}\" --source {}",
        path.display(),
        source_arg(source)?
    ))
}

fn source_arg(source: &AgentSource) -> Result<&'static str, HookInstallError> {
    match source {
        AgentSource::Codex => Ok("codex"),
        AgentSource::ClaudeCode => Ok("claude-code"),
        AgentSource::Manual => Err(HookInstallError::new(
            "unsupported-source",
            "manual 不支持 hook 接入",
        )),
    }
}

fn helper_script() -> Result<String, HookInstallError> {
    let app_dir = config_store::app_support_dir()
        .ok_or_else(|| HookInstallError::new("home-missing", "无法定位用户 HOME"))?;
    let app_dir = shell_single_quote(&app_dir.display().to_string());

    Ok(format!(
        r#"#!/bin/sh
# Agent Island hook helper v2
APP_DIR={app_dir}
PYTHON_BIN="$(command -v python3 || true)"
if [ -z "$PYTHON_BIN" ]; then
  case " $* " in *" --self-test "*) exit 1 ;; *) exit 0 ;; esac
fi

AGENT_ISLAND_APP_DIR="$APP_DIR" "$PYTHON_BIN" -c '
import datetime, hashlib, json, os, sys

args = sys.argv[1:]
self_test = "--self-test" in args
source = None
for index, arg in enumerate(args):
    if arg == "--source" and index + 1 < len(args):
        source = args[index + 1]

if source not in ("codex", "claude-code"):
    sys.exit(1 if self_test else 0)

app_dir = os.environ.get("AGENT_ISLAND_APP_DIR")
if not app_dir:
    sys.exit(1 if self_test else 0)

def load_json(path):
    try:
        with open(path, "r", encoding="utf-8") as handle:
            return json.load(handle)
    except Exception:
        return {{}}

settings = load_json(os.path.join(app_dir, "config.json"))
settings_key = {{"codex": "codex", "claude-code": "claudeCode"}}[source]
if settings.get("hookSource", {{}}).get(settings_key) is not True:
    sys.exit(1 if self_test else 0)

events_dir = os.path.join(app_dir, "events")
event_path = os.path.join(events_dir, source + ".jsonl")
os.makedirs(events_dir, exist_ok=True)

if self_test:
    with open(event_path, "a", encoding="utf-8"):
        pass
    sys.exit(0)

try:
    raw = sys.stdin.read(1048576)
    payload = json.loads(raw) if raw.strip() else {{}}
except Exception:
    payload = {{}}

def text(value):
    return value if isinstance(value, str) else None

def pick(*names):
    for name in names:
        value = text(payload.get(name))
        if value:
            return value
    return None

def has_key(obj, names, depth=0):
    if depth > 3:
        return False
    if isinstance(obj, dict):
        for key, value in obj.items():
            if key in names:
                return True
            if has_key(value, names, depth + 1):
                return True
    elif isinstance(obj, list):
        return any(has_key(item, names, depth + 1) for item in obj[:20])
    return False

def clean(value, limit=80):
    if not isinstance(value, str):
        return None
    value = "".join(ch for ch in value if ch.isprintable()).strip()
    return value[:limit] or None

event = clean(pick("hook_event_name", "hookEventName", "event", "event_name")) or "HookEvent"
session_id = clean(pick("session_id", "sessionId", "conversation_id", "thread_id"), 160)
transcript_path = pick("transcript_path", "transcriptPath")
cwd = clean(pick("cwd"), 512)
session_seed = session_id or transcript_path or cwd or "unknown"
session_key = hashlib.sha256((source + ":" + session_seed).encode("utf-8")).hexdigest()[:16]

tool_name = clean(pick("tool_name", "toolName"))
tool = payload.get("tool")
if tool_name is None and isinstance(tool, dict):
    tool_name = clean(text(tool.get("name")) or text(tool.get("tool_name")) or text(tool.get("toolName")))
tool_input = payload.get("tool_input") or payload.get("toolInput")
if tool_name is None and isinstance(tool_input, dict):
    tool_name = clean(text(tool_input.get("name")) or text(tool_input.get("tool_name")) or text(tool_input.get("toolName")))

timestamp = datetime.datetime.now(datetime.timezone.utc).isoformat().replace("+00:00", "Z")
record = {{
    "schemaVersion": 1,
    "source": source,
    "event": event,
    "sessionId": session_id,
    "sessionKey": session_key,
    "cwd": cwd,
    "timestamp": timestamp,
    "toolName": tool_name,
    "actionSummary": None,
    "permissionMode": clean(pick("permission_mode", "permissionMode")),
    "rawEventFields": {{
        "hasTranscriptPath": transcript_path is not None,
        "hasPrompt": has_key(payload, {{"prompt", "user_prompt", "userPrompt"}}),
        "hasToolInput": has_key(payload, {{"tool_input", "toolInput"}}),
        "hasToolResponse": has_key(payload, {{"tool_response", "toolResponse"}}),
    }},
}}

with open(event_path, "a", encoding="utf-8") as handle:
    handle.write(json.dumps(record, ensure_ascii=False, separators=(",", ":")) + "\n")
' "$@" >/dev/null 2>&1
STATUS=$?

case " $* " in
  *" --self-test "*) exit "$STATUS" ;;
  *) exit 0 ;;
esac
"#
    ))
}

fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
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
        AgentSource::Manual => Err(HookInstallError::new(
            "unsupported-source",
            "manual 不支持 hook 接入",
        )),
    }
}

fn source_events(source: &AgentSource) -> Result<&'static [&'static str], HookInstallError> {
    match source {
        AgentSource::Codex => Ok(CODEX_EVENTS),
        AgentSource::ClaudeCode => Ok(CLAUDE_EVENTS),
        AgentSource::Manual => Err(HookInstallError::new(
            "unsupported-source",
            "manual 不支持 hook 接入",
        )),
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

    let contents = fs::read_to_string(path)
        .map_err(|error| HookInstallError::new("config-read-failed", error.to_string()))?;
    if contents.trim().is_empty() {
        return Ok(json!({}));
    }

    let value: Value = serde_json::from_str(&contents)
        .map_err(|error| HookInstallError::new("config-parse-failed", error.to_string()))?;
    if !value.is_object() {
        return Err(HookInstallError::new(
            "config-not-object",
            "目标配置不是 JSON object",
        ));
    }

    Ok(value)
}

fn write_json_atomic(path: &Path, value: &Value) -> Result<(), HookInstallError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| HookInstallError::new("config-dir-failed", error.to_string()))?;
    }

    let temp_path = path.with_extension("agent-island-tmp");
    let contents = serde_json::to_string_pretty(value)
        .map_err(|error| HookInstallError::new("config-serialize-failed", error.to_string()))?;
    fs::write(&temp_path, contents)
        .map_err(|error| HookInstallError::new("config-write-failed", error.to_string()))?;
    fs::rename(&temp_path, path)
        .map_err(|error| HookInstallError::new("config-rename-failed", error.to_string()))
}

fn backup_if_exists(path: &Path) -> Result<(), HookInstallError> {
    if !path.exists() {
        return Ok(());
    }

    let app_dir = config_store::app_support_dir()
        .ok_or_else(|| HookInstallError::new("home-missing", "无法定位用户 HOME"))?;
    let backup_dir = app_dir.join("backups");
    fs::create_dir_all(&backup_dir)
        .map_err(|error| HookInstallError::new("backup-dir-failed", error.to_string()))?;

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("hook-config");
    let backup_path = backup_dir.join(format!(
        "{}-{}.json",
        file_name,
        chrono::Utc::now().format("%Y%m%dT%H%M%SZ")
    ));
    fs::copy(path, backup_path)
        .map_err(|error| HookInstallError::new("backup-copy-failed", error.to_string()))?;
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

fn update_manifest_entry(
    source: &AgentSource,
    target_path: &Path,
    command: &str,
) -> Result<(), HookInstallError> {
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
