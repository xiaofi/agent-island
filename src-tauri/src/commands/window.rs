use tauri::{
    AppHandle, LogicalSize, Manager, PhysicalPosition, Position, Size, WebviewUrl,
    WebviewWindowBuilder, Window,
};

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
pub async fn set_dock_visibility(app: AppHandle, visible: bool) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        return app
            .set_dock_visibility(visible)
            .map_err(|error| error.to_string());
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = (app, visible);
        Ok(())
    }
}

#[tauri::command]
pub async fn save_island_window_position(app: AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "main window not found".to_string())?;

    island_window::save_current_position(&window)
}

#[tauri::command]
pub async fn set_window_mode(
    window: Window,
    expanded: bool,
    collapsed_height: Option<f64>,
    expansion_direction: Option<String>,
) -> Result<String, String> {
    let collapsed_height = collapsed_height.unwrap_or(44.0).clamp(44.0, 420.0);
    let direction = if expanded {
        resolve_expansion_direction(&window, collapsed_height)?
    } else {
        "down".to_string()
    };
    let size = if expanded {
        LogicalSize {
            width: 260.0,
            height: 460.0,
        }
    } else {
        LogicalSize {
            width: 260.0,
            height: collapsed_height,
        }
    };

    let target_y = if expanded && direction == "up" {
        anchored_top_for_height(&window, size.height)
    } else if !expanded && expansion_direction.as_deref() == Some("up") {
        anchored_top_for_height(&window, size.height)
    } else {
        None
    };

    window
        .set_size(Size::Logical(size))
        .map_err(|error| error.to_string())?;

    if let Some(y) = target_y {
        let current_position = window.outer_position().map_err(|error| error.to_string())?;
        window
            .set_position(Position::Physical(PhysicalPosition::new(
                current_position.x,
                y,
            )))
            .map_err(|error| error.to_string())?;
    }

    if let Some(main_window) = window.app_handle().get_webview_window("main") {
        island_window::reapply_island_window_level(&main_window)?;
    }

    Ok(direction)
}

fn resolve_expansion_direction(window: &Window, collapsed_height: f64) -> Result<String, String> {
    let Some(monitor) = window
        .current_monitor()
        .map_err(|error| error.to_string())?
    else {
        return Ok("down".to_string());
    };

    let position = window.outer_position().map_err(|error| error.to_string())?;
    let size = window.outer_size().map_err(|error| error.to_string())?;
    let work_area = monitor.work_area();
    let scale_factor = monitor.scale_factor();
    let expanded_height = (460.0 * scale_factor).round() as i32;
    let collapsed_height = (collapsed_height * scale_factor).round() as i32;
    let extra_height = (expanded_height - collapsed_height).max(0);
    let current_bottom = position.y + size.height as i32;
    let work_top = work_area.position.y;
    let work_bottom = work_top + work_area.size.height as i32;
    let space_above = (position.y - work_top).max(0);
    let space_below = (work_bottom - current_bottom).max(0);

    if space_below < extra_height && space_above > space_below {
        Ok("up".to_string())
    } else {
        Ok("down".to_string())
    }
}

fn anchored_top_for_height(window: &Window, height: f64) -> Option<i32> {
    let monitor = window.current_monitor().ok()??;
    let position = window.outer_position().ok()?;
    let size = window.outer_size().ok()?;
    let work_area = monitor.work_area();
    let scale_factor = monitor.scale_factor();
    let target_height = (height * scale_factor).round() as i32;
    let current_bottom = position.y + size.height as i32;
    let work_top = work_area.position.y;
    let work_bottom = work_top + work_area.size.height as i32;
    let min_top = work_top;
    let max_top = work_bottom - target_height;

    Some((current_bottom - target_height).clamp(min_top, max_top.max(min_top)))
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
