use crate::{
    adapters::{mock::mock_tasks, types::AgentTask},
    aggregator::sort_tasks_by_updated_at,
    services::app_open,
};

#[tauri::command]
pub async fn get_tasks() -> Result<Vec<AgentTask>, String> {
    Ok(sort_tasks_by_updated_at(mock_tasks()))
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
