use crate::{
    adapters::types::AgentTask,
    services::{app_open, config_store, hook_ingest},
};

#[tauri::command]
pub async fn get_tasks() -> Result<Vec<AgentTask>, String> {
    let settings = config_store::load_settings();
    Ok(hook_ingest::load_tasks_with_fallback(&settings))
}

#[tauri::command]
pub async fn open_task(task_id: String) -> Result<(), String> {
    println!("open_task requested for {task_id}");
    Ok(())
}

#[tauri::command]
pub async fn open_workdir(path: String) -> Result<(), String> {
    app_open::open_path(&path)
}

#[tauri::command]
pub async fn copy_task_summary(task_id: String) -> Result<(), String> {
    println!("copy_task_summary requested for {task_id}");
    Ok(())
}
