use crate::{
    adapters::types::{AgentSource, AppSettings, HookOperation, HookOperationError},
    services::{config_store, hook_installer},
};
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub async fn set_hook_source_enabled(
    app: AppHandle,
    source: AgentSource,
    enabled: bool,
) -> Result<AppSettings, String> {
    let settings = if enabled {
        run_install(source, HookOperation::Install)
    } else {
        run_uninstall(source)
    }?;
    emit_settings_updated(&app, &settings);
    Ok(settings)
}

#[tauri::command]
pub async fn retry_hook_source_operation(
    app: AppHandle,
    source: AgentSource,
    operation: HookOperation,
) -> Result<AppSettings, String> {
    let settings = match operation {
        HookOperation::Install => run_install(source, HookOperation::Install),
        HookOperation::Repair => run_install(source, HookOperation::Repair),
        HookOperation::Uninstall => run_uninstall(source),
        HookOperation::SelfTest => run_self_test(source),
    }?;
    emit_settings_updated(&app, &settings);
    Ok(settings)
}

#[tauri::command]
pub async fn run_hook_self_test(
    app: AppHandle,
    source: AgentSource,
) -> Result<AppSettings, String> {
    let settings = run_self_test(source)?;
    emit_settings_updated(&app, &settings);
    Ok(settings)
}

fn run_install(source: AgentSource, operation: HookOperation) -> Result<AppSettings, String> {
    match hook_installer::install_source(&source) {
        Ok(()) => {
            let mut settings = config_store::load_settings();
            set_source_enabled(&mut settings, &source, true);
            config_store::save_settings(&settings)?;

            let self_test_result = hook_installer::self_test_source(&source);

            match self_test_result {
                Ok(()) => {
                    clear_source_error(&mut settings, &source);
                    hook_installer::clear_manifest_error(&source)?;
                }
                Err(error) => {
                    let operation_error =
                        hook_installer::operation_error(HookOperation::SelfTest, error);
                    set_source_error(&mut settings, &source, operation_error.clone());
                    hook_installer::persist_manifest_error(&source, &operation_error)?;
                }
            }

            config_store::save_settings(&settings)?;
            Ok(settings)
        }
        Err(error) => persist_error(source, operation, error),
    }
}

fn run_uninstall(source: AgentSource) -> Result<AppSettings, String> {
    match hook_installer::uninstall_source(&source) {
        Ok(()) => {
            let mut settings = config_store::load_settings();
            set_source_enabled(&mut settings, &source, false);
            clear_source_error(&mut settings, &source);
            hook_installer::clear_manifest_error(&source)?;
            config_store::save_settings(&settings)?;
            Ok(settings)
        }
        Err(error) => {
            let mut settings = config_store::load_settings();
            set_source_enabled(&mut settings, &source, true);
            let operation_error = hook_installer::operation_error(HookOperation::Uninstall, error);
            set_source_error(&mut settings, &source, operation_error.clone());
            hook_installer::persist_manifest_error(&source, &operation_error)?;
            config_store::save_settings(&settings)?;
            Ok(settings)
        }
    }
}

fn run_self_test(source: AgentSource) -> Result<AppSettings, String> {
    match hook_installer::self_test_source(&source) {
        Ok(()) => {
            let mut settings = config_store::load_settings();
            clear_source_error(&mut settings, &source);
            hook_installer::clear_manifest_error(&source)?;
            config_store::save_settings(&settings)?;
            Ok(settings)
        }
        Err(error) => persist_error(source, HookOperation::SelfTest, error),
    }
}

fn persist_error(
    source: AgentSource,
    operation: HookOperation,
    error: hook_installer::HookInstallError,
) -> Result<AppSettings, String> {
    let operation_error = hook_installer::operation_error(operation, error);
    let mut settings = config_store::load_settings();
    set_source_error(&mut settings, &source, operation_error.clone());
    hook_installer::persist_manifest_error(&source, &operation_error)?;
    config_store::save_settings(&settings)?;
    Ok(settings)
}

fn emit_settings_updated(app: &AppHandle, settings: &AppSettings) {
    let _ = app.emit("settings-updated", settings.clone());
}

fn set_source_enabled(settings: &mut AppSettings, source: &AgentSource, enabled: bool) {
    match source {
        AgentSource::Codex => settings.hook_source.codex = enabled,
        AgentSource::ClaudeCode => settings.hook_source.claude_code = enabled,
        AgentSource::Manual => {}
    }
}

fn set_source_error(settings: &mut AppSettings, source: &AgentSource, error: HookOperationError) {
    match source {
        AgentSource::Codex => settings.hook_source.last_errors.codex = Some(error),
        AgentSource::ClaudeCode => settings.hook_source.last_errors.claude_code = Some(error),
        AgentSource::Manual => {}
    }
}

fn clear_source_error(settings: &mut AppSettings, source: &AgentSource) {
    match source {
        AgentSource::Codex => settings.hook_source.last_errors.codex = None,
        AgentSource::ClaudeCode => settings.hook_source.last_errors.claude_code = None,
        AgentSource::Manual => {}
    }
}
