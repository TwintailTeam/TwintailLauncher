use tauri::{AppHandle};
use crate::utils::db_manager::get_repositories;

#[tauri::command]
pub async fn list_repositories(app: AppHandle) -> Option<String> {
    let repos = get_repositories(&app).await;

    if repos.is_some() {
        let repository = repos.unwrap();
        let stringified = serde_json::to_string(&repository).unwrap();
        Some(stringified)
    } else {
        None
    }
}