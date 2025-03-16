use tauri::{AppHandle};
use crate::utils::db_manager::{create_installation, get_install_info_by_id, get_installs, get_installs_by_manifest_id, get_manifest_info_by_filename};
use crate::utils::generate_cuid;
use crate::utils::repo_manager::get_manifest;

#[tauri::command]
pub async fn list_installs(app: AppHandle) -> Option<String> {
    let installs = get_installs(&app);

    if installs.is_some() {
        let install = installs.unwrap();
        let stringified = serde_json::to_string(&install).unwrap();
        Some(stringified)
    } else {
        None
    }
}

#[tauri::command]
pub fn list_installs_by_manifest_id(app: AppHandle, manifest_id: String) -> Option<String> {
    let installs = get_installs_by_manifest_id(&app, manifest_id);

    if installs.is_some() {
        let install = installs.unwrap();
        let stringified = serde_json::to_string(&install).unwrap();
        Some(stringified)
    } else {
        None
    }
}

#[tauri::command]
pub fn get_install_by_id(app: AppHandle, id: String) -> Option<String> {
    let inst = get_install_info_by_id(&app, id);

    if inst.is_some() {
        let install = inst.unwrap();
        let stringified = serde_json::to_string(&install).unwrap();
        Some(stringified)
    } else {
        None
    }
}

#[tauri::command]
pub async fn add_install(app: AppHandle, manifest_id: String, version: String, name: String, directory: String, runner: String, dxvk: String, game_icon: String, game_background: String, ignore_updates: bool, skip_hash_check: bool, use_jadeite: bool, use_xxmi: bool, use_fps_unlock: bool, env_vars: String, pre_launch_command: String, launch_command: String) -> Option<bool> {
    if manifest_id.is_empty() || version.is_empty() || name.is_empty() || directory.is_empty() || runner.is_empty() || dxvk.is_empty() || game_icon.is_empty() || game_background.is_empty() || launch_command.is_empty() {
        None
    } else {
        // TODO: Write bullshit to download, unpack, setup the entire installation
        let cuid = generate_cuid();
        let m = manifest_id + ".json";
        let dbm = get_manifest_info_by_filename(&app, m.clone()).unwrap();
        let gm = get_manifest(&app, &m.clone()).unwrap();
        let g = gm.game_versions.iter().find(|e| e.metadata.version == version).unwrap();

        create_installation(&app, cuid, dbm.id, version, g.metadata.versioned_name.clone(), directory, runner, dxvk, g.assets.game_icon.clone(), g.assets.game_background.clone(), ignore_updates, skip_hash_check, use_jadeite, use_xxmi, use_fps_unlock, env_vars, pre_launch_command, launch_command).unwrap();

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