use tauri::{AppHandle, Manager};
use crate::utils::db_manager::{delete_repository_by_id, get_repositories, get_repository_info_by_id};
use crate::utils::repo_manager::clone_new_repository;

#[tauri::command]
pub fn list_repositories(app: AppHandle) -> Option<String> {
    let repos = get_repositories(&app);

    if repos.is_some() {
        let repository = repos.unwrap();
        let stringified = serde_json::to_string(&repository).unwrap();
        Some(stringified)
    } else {
        None
    }
}

#[tauri::command]
pub fn get_repository(app: AppHandle, repository_id: String) -> Option<String> {
    let repo = get_repository_info_by_id(&app, repository_id);

    if repo.is_some() {
        let repository = repo.unwrap();
        let stringified = serde_json::to_string(&repository).unwrap();
        Some(stringified)
    } else {
        None
    }
}

#[tauri::command]
pub fn add_repository(app: AppHandle, url: String) -> Option<bool> {
    if url.is_empty() {
        None
    } else {
        let path = app.path().app_data_dir().unwrap().join("manifests");
        let rtn = clone_new_repository(&app, &path, url);

        if rtn.is_ok() {
            Some(rtn.unwrap())
        } else {
            None
        }
    }
}

#[tauri::command]
pub fn remove_repository(app: AppHandle, id: String) -> Option<bool> {
    if id.is_empty() {
        None
    } else {
        // TODO: Properly delete repository bullshit and disallow if installation with ANY manifest of a repo exists
        let rtn = delete_repository_by_id(&app, id);
        if rtn.is_ok() {
            Some(rtn.unwrap())
        } else {
            None
        }
    }
}