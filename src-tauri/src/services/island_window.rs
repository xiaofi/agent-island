#[cfg(target_os = "macos")]
use objc2_app_kit::{NSScreenSaverWindowLevel, NSWindow, NSWindowCollectionBehavior};
use tauri::WebviewWindow;

pub fn configure_island_window(window: &WebviewWindow) -> Result<(), String> {
    window
        .set_always_on_top(true)
        .map_err(|error| error.to_string())?;
    window
        .set_visible_on_all_workspaces(true)
        .map_err(|error| error.to_string())?;

    configure_platform_window(window)
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
