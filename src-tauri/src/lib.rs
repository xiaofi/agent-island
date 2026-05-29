mod adapters;
mod aggregator;
mod commands;
mod services;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::task::get_tasks,
            commands::discovery::run_discovery,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::task::open_task,
            commands::task::open_workdir,
            commands::task::copy_task_summary,
            commands::window::set_mouse_passthrough,
            commands::window::set_window_mode,
            commands::window::open_app_window,
            commands::window::toggle_window_visibility,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Agent Island");
}
