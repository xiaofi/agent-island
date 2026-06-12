use crate::{
    adapters::types::{AppSettings, AppSettingsPatch},
    services::{config_store, task_notifications},
};
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub async fn get_settings() -> Result<AppSettings, String> {
    Ok(config_store::load_settings())
}

#[tauri::command]
pub async fn update_settings(
    app: AppHandle,
    patch: AppSettingsPatch,
) -> Result<AppSettings, String> {
    let mut settings = config_store::load_settings();

    if let Some(appearance) = patch.appearance {
        settings.appearance = appearance;
    }

    if let Some(notifications) = patch.notifications {
        settings.notifications = notifications;
    }

    if let Some(auto_acknowledge) = patch.auto_acknowledge {
        settings.auto_acknowledge = auto_acknowledge;
    }

    if let Some(quiet_mode) = patch.quiet_mode {
        settings.quiet_mode = quiet_mode;
    }

    if let Some(show_in_dock) = patch.show_in_dock {
        settings.show_in_dock = show_in_dock;
    }

    if let Some(enabled_adapters) = patch.enabled_adapters {
        settings.enabled_adapters = enabled_adapters;
    }

    if let Some(hook_source) = patch.hook_source {
        settings.hook_source = hook_source;
    }

    if let Some(island_window) = patch.island_window {
        settings.island_window = island_window;
    }

    config_store::save_settings(&settings)?;
    let _ = app.emit("settings-updated", settings.clone());
    Ok(settings)
}

#[tauri::command]
pub async fn send_test_notification(app: AppHandle, sound: String) -> Result<(), String> {
    task_notifications::send_test_notification(&app, &sound)
}
