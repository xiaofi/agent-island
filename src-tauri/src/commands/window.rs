use tauri::{AppHandle, LogicalSize, Manager, Size, WebviewUrl, WebviewWindowBuilder, Window};

use crate::services::island_window;

#[tauri::command]
pub async fn set_mouse_passthrough(app: AppHandle, enabled: bool) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "main window not found".to_string())?;

    window
        .set_ignore_cursor_events(enabled)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn set_window_mode(
    window: Window,
    expanded: bool,
    collapsed_height: Option<f64>,
) -> Result<(), String> {
    let size = if expanded {
        LogicalSize {
            width: 260.0,
            height: 460.0,
        }
    } else {
        LogicalSize {
            width: 260.0,
            height: collapsed_height.unwrap_or(44.0).clamp(44.0, 420.0),
        }
    };

    window
        .set_size(Size::Logical(size))
        .map_err(|error| error.to_string())?;

    if let Some(main_window) = window.app_handle().get_webview_window("main") {
        island_window::reapply_island_window_level(&main_window)?;
    }

    Ok(())
}

#[tauri::command]
pub async fn open_app_window(app: AppHandle, kind: String) -> Result<(), String> {
    let (label, title, width, height) = match kind.as_str() {
        "settings" => ("settings", "Agent Island Settings", 760.0, 620.0),
        "diagnostics" => ("diagnostics", "Agent Island Diagnostics", 840.0, 640.0),
        _ => return Err(format!("unsupported window kind: {kind}")),
    };

    if let Some(window) = app.get_webview_window(label) {
        window.show().map_err(|error| error.to_string())?;
        window.set_focus().map_err(|error| error.to_string())?;
        return Ok(());
    }

    WebviewWindowBuilder::new(
        &app,
        label,
        WebviewUrl::App(format!("index.html?window={label}").into()),
    )
    .title(title)
    .inner_size(width, height)
    .min_inner_size(560.0, 420.0)
    .resizable(true)
    .decorations(true)
    .transparent(false)
    .always_on_top(false)
    .center()
    .build()
    .map_err(|error| error.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn toggle_window_visibility(window: Window) -> Result<(), String> {
    if window.is_visible().map_err(|error| error.to_string())? {
        window.hide().map_err(|error| error.to_string())
    } else {
        window.show().map_err(|error| error.to_string())?;
        if let Some(main_window) = window.app_handle().get_webview_window("main") {
            island_window::reapply_island_window_level(&main_window)?;
        }
        window.set_focus().map_err(|error| error.to_string())
    }
}
