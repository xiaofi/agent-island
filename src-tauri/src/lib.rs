mod adapters;
mod aggregator;
mod commands;
mod services;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let _ = services::hook_installer::refresh_helper_script();

            if let Some(window) = app.get_webview_window("main") {
                services::island_window::configure_island_window(&window)
                    .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::task::get_tasks,
            commands::discovery::run_discovery,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::hook::set_hook_source_enabled,
            commands::hook::retry_hook_source_operation,
            commands::hook::run_hook_self_test,
            commands::task::open_task,
            commands::task::open_workdir,
            commands::task::copy_task_summary,
            commands::window::set_mouse_passthrough,
            commands::window::save_island_window_position,
            commands::window::set_window_mode,
            commands::window::open_app_window,
            commands::window::toggle_window_visibility,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Agent Island");
}
