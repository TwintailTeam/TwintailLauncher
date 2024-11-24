use tauri::{AppHandle};
use crate::utils::db_manager::{get_install_info_by_id, get_installs, get_installs_by_manifest_id};

#[tauri::command]
pub async fn list_installs(app: AppHandle) -> Option<String> {
    let installs = get_installs(&app).await;

    if installs.is_some() {
        let install = installs.unwrap();
        let stringified = serde_json::to_string(&install).unwrap();
        Some(stringified)
    } else {
        None
    }
}

#[tauri::command]
pub async fn list_installs_by_manifest_id(app: AppHandle, manifest_id: String) -> Option<String> {
    let installs = get_installs_by_manifest_id(&app, manifest_id).await;

    if installs.is_some() {
        let install = installs.unwrap();
        let stringified = serde_json::to_string(&install).unwrap();
        Some(stringified)
    } else {
        None
    }
}

#[tauri::command]
pub async fn get_install_by_id(app: AppHandle, id: String) -> Option<String> {
    let inst = get_install_info_by_id(&app, id).await;

    if inst.is_some() {
        let install = inst.unwrap();
        let stringified = serde_json::to_string(&install).unwrap();
        Some(stringified)
    } else {
        None
    }
}

#[tauri::command]
pub async fn add_install(_app: AppHandle, manifest_id: String, version: String, name: String, directory: String, runner: String, dxvk: String) -> Option<bool> {
    if manifest_id.is_empty() || version.is_empty() || name.is_empty() || directory.is_empty() || runner.is_empty() || dxvk.is_empty() {
        None
    } else {
        // TODO: Write bullshit to download, unpack, setup the entire installation
        Some(true)
    }
}

#[tauri::command]
pub async fn remove_install(_app: AppHandle, id: String) -> Option<bool> {
    if id.is_empty() {
        None
    } else {
        // TODO: Write more bullshit to uninstall the installation and wipe its files
        Some(true)
    }
}