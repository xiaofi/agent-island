#[cfg(target_os = "macos")]
use objc2_app_kit::{NSScreenSaverWindowLevel, NSWindow, NSWindowCollectionBehavior};
use tauri::{Monitor, PhysicalPosition, PhysicalSize, Position, WebviewWindow};

use crate::{adapters::types::IslandWindowSettings, services::config_store};

const DEFAULT_WINDOW_MARGIN: f64 = 18.0;

pub fn configure_island_window(window: &WebviewWindow) -> Result<(), String> {
    window
        .set_always_on_top(true)
        .map_err(|error| error.to_string())?;
    window
        .set_visible_on_all_workspaces(true)
        .map_err(|error| error.to_string())?;

    restore_saved_or_position_at_top_right(window)?;
    configure_platform_window(window)?;
    window.show().map_err(|error| error.to_string())
}

pub fn reapply_island_window_level(window: &WebviewWindow) -> Result<(), String> {
    let window = window.clone();
    window
        .clone()
        .run_on_main_thread(move || {
            let _ = configure_platform_window(&window);
        })
        .map_err(|error| error.to_string())
}

#[cfg(target_os = "macos")]
fn configure_platform_window(window: &WebviewWindow) -> Result<(), String> {
    let ns_window = window.ns_window().map_err(|error| error.to_string())?;

    // Full-screen apps live in separate macOS Spaces; Tauri's normal
    // always-on-top level is not enough for an overlay-style island there.
    unsafe {
        let ns_window: &NSWindow = &*ns_window.cast();
        let mut behavior = ns_window.collectionBehavior();
        behavior |= NSWindowCollectionBehavior::CanJoinAllSpaces;
        behavior |= NSWindowCollectionBehavior::CanJoinAllApplications;
        behavior |= NSWindowCollectionBehavior::FullScreenAuxiliary;
        behavior |= NSWindowCollectionBehavior::Stationary;
        behavior |= NSWindowCollectionBehavior::IgnoresCycle;
        ns_window.setCollectionBehavior(behavior);
        ns_window.setAcceptsMouseMovedEvents(true);
        ns_window.setMovableByWindowBackground(true);
        ns_window.setCanHide(false);
        ns_window.setHidesOnDeactivate(false);
        ns_window.setExcludedFromWindowsMenu(true);
        ns_window.setLevel(NSScreenSaverWindowLevel);
    }

    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn configure_platform_window(_window: &WebviewWindow) -> Result<(), String> {
    Ok(())
}

pub fn save_current_position(window: &WebviewWindow) -> Result<(), String> {
    let position = window.outer_position().map_err(|error| error.to_string())?;
    save_window_position(position.x, position.y)
}

fn restore_saved_or_position_at_top_right(window: &WebviewWindow) -> Result<(), String> {
    let settings = config_store::load_settings();
    if let Some(monitor) = monitor_for_saved_position(window, &settings.island_window)? {
        position_from_settings(window, &monitor, &settings.island_window)?;
        return Ok(());
    }

    position_at_top_right(window)
}

fn position_at_top_right(window: &WebviewWindow) -> Result<(), String> {
    let Some(monitor) = window
        .current_monitor()
        .map_err(|error| error.to_string())?
        .or(window
            .primary_monitor()
            .map_err(|error| error.to_string())?)
    else {
        return Ok(());
    };

    let size = window.outer_size().map_err(|error| error.to_string())?;
    let work_area = monitor.work_area();
    let margin = (DEFAULT_WINDOW_MARGIN * monitor.scale_factor()).round() as i32;
    let work_left = work_area.position.x;
    let work_top = work_area.position.y;
    let work_right = work_left + work_area.size.width as i32;
    let max_x = work_right - size.width as i32 - margin;
    let x = max_x.max(work_left + margin);
    let y = work_top + margin;

    window
        .set_position(Position::Physical(PhysicalPosition::new(x, y)))
        .map_err(|error| error.to_string())
}

fn position_from_settings(
    window: &WebviewWindow,
    monitor: &Monitor,
    settings: &IslandWindowSettings,
) -> Result<(), String> {
    let size = window.outer_size().map_err(|error| error.to_string())?;
    let work_area = WorkArea::from_monitor(monitor);
    let margin = margin_for(monitor);
    let fallback_x = work_area.right - size.width as i32 - margin;
    let fallback_y = work_area.top + margin;
    let saved_x = settings.x.unwrap_or(fallback_x);
    let saved_y = settings.y.unwrap_or(fallback_y);
    let (x, y) = clamped_position(saved_x, saved_y, size, work_area, margin);

    window
        .set_position(Position::Physical(PhysicalPosition::new(x, y)))
        .map_err(|error| error.to_string())
}

fn save_window_position(x: i32, y: i32) -> Result<(), String> {
    let mut settings = config_store::load_settings();
    settings.island_window = IslandWindowSettings {
        x: Some(x),
        y: Some(y),
    };
    config_store::save_settings(&settings)
}

fn current_or_primary_monitor(window: &WebviewWindow) -> Result<Option<Monitor>, String> {
    Ok(window
        .current_monitor()
        .map_err(|error| error.to_string())?
        .or(window
            .primary_monitor()
            .map_err(|error| error.to_string())?))
}

fn monitor_for_saved_position(
    window: &WebviewWindow,
    settings: &IslandWindowSettings,
) -> Result<Option<Monitor>, String> {
    if let (Some(x), Some(y)) = (settings.x, settings.y) {
        if let Some(monitor) = monitor_containing_position(window, x, y)? {
            return Ok(Some(monitor));
        }
    }

    current_or_primary_monitor(window)
}

fn monitor_containing_position(
    window: &WebviewWindow,
    x: i32,
    y: i32,
) -> Result<Option<Monitor>, String> {
    let monitors = window
        .available_monitors()
        .map_err(|error| error.to_string())?;

    Ok(monitors.into_iter().find(|monitor| {
        let work_area = WorkArea::from_monitor(monitor);
        x >= work_area.left && x <= work_area.right && y >= work_area.top && y <= work_area.bottom
    }))
}

fn clamped_position(
    x: i32,
    y: i32,
    size: PhysicalSize<u32>,
    work_area: WorkArea,
    margin: i32,
) -> (i32, i32) {
    let left_x = work_area.left + margin;
    let right_x = work_area.right - size.width as i32 - margin;
    let top_y = work_area.top + margin;
    let bottom_y = work_area.bottom - size.height as i32 - margin;
    let clamped_x = clamp_axis(x, left_x, right_x);
    let clamped_y = clamp_axis(y, top_y, bottom_y);

    (clamped_x, clamped_y)
}

fn clamp_axis(value: i32, min: i32, max: i32) -> i32 {
    value.clamp(min, max.max(min))
}

fn margin_for(monitor: &Monitor) -> i32 {
    (DEFAULT_WINDOW_MARGIN * monitor.scale_factor()).round() as i32
}

#[derive(Clone, Copy)]
struct WorkArea {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

impl WorkArea {
    fn from_monitor(monitor: &Monitor) -> Self {
        let work_area = monitor.work_area();
        let left = work_area.position.x;
        let top = work_area.position.y;
        Self {
            left,
            top,
            right: left + work_area.size.width as i32,
            bottom: top + work_area.size.height as i32,
        }
    }
}
