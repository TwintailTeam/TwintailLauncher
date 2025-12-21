use linked_hash_map::LinkedHashMap;
use tauri::{AppHandle};
use crate::utils::db_manager::{get_manifest_info_by_filename, get_manifest_info_by_id, get_manifests_by_repository_id, update_manifest_enabled_by_id};
use crate::utils::repo_manager::{get_manifest, get_manifests};
use crate::utils::models::{GameManifest};

#[cfg(target_os = "linux")]
use crate::utils::repo_manager::{get_compatibilities, get_compatibility};
#[cfg(target_os = "linux")]
use crate::utils::models::{RunnerManifest};

#[tauri::command]
pub fn get_manifest_by_id(app: AppHandle, id: String) -> Option<String> {
    let manifest = get_manifest_info_by_id(&app, id);

    if manifest.is_some() {
        let m = manifest.unwrap();
        let stringified = serde_json::to_string(&m).unwrap();
        Some(stringified)
    } else {
        None
    }
}

#[tauri::command]
pub fn get_manifest_by_filename(app: AppHandle, filename: String) -> Option<String> {
    let manifest = get_manifest_info_by_filename(&app, filename);

    if manifest.is_some() {
        let m = manifest.unwrap();
        let stringified = serde_json::to_string(&m).unwrap();
        Some(stringified)
    } else {
        None
    }
}

#[tauri::command]
pub fn list_manifests_by_repository_id(app: AppHandle, repository_id: String) -> Option<String> {
    let manifests = get_manifests_by_repository_id(&app, repository_id);

    if manifests.is_some() {
        let manifest = manifests.unwrap();
        let stringified = serde_json::to_string(&manifest).unwrap();
        Some(stringified)
    } else {
        None
    }
}

#[tauri::command]
pub fn list_game_manifests(app: AppHandle) -> Option<String> {
    let manifestss: LinkedHashMap<String, GameManifest> = get_manifests(&app);
    let mut manifests: Vec<GameManifest> = Vec::new();

    for value in manifestss.clone().into_iter().map(|(_, value)| value) { manifests.push(value); }

    if manifests.is_empty() {
        None
    } else {
        let stringified = serde_json::to_string(&manifests).unwrap();
        Some(stringified)
    }
}

#[tauri::command]
pub fn get_game_manifest_by_filename(app: AppHandle, filename: String) -> Option<String> {
    let manifest = get_manifest(&app, filename.clone());
    let db_manifest = get_manifest_info_by_filename(&app, filename.clone());

    if manifest.is_some() && db_manifest.is_some() {
        let dbm = db_manifest.unwrap();

        if dbm.enabled {
            let m = manifest.unwrap();
            let stringified = serde_json::to_string(&m).unwrap();
            Some(stringified)
        } else {
            None
        }
    } else {
        None
    }
}

#[tauri::command]
pub fn get_game_manifest_by_manifest_id(app: AppHandle, id: String) -> Option<String> {
    let db_manifest = get_manifest_info_by_id(&app, id.clone());

    if db_manifest.is_some() {
        let dbm = db_manifest.unwrap();
        let manifest = get_manifest(&app, dbm.filename);

        if dbm.enabled {
            let m = manifest.unwrap();
            let stringified = serde_json::to_string(&m).unwrap();
            Some(stringified)
        } else {
            None
        }
    } else {
        None
    }
}

#[tauri::command]
pub fn update_manifest_enabled(app: AppHandle, id: String, enabled: bool) -> Option<bool> {
    let manifest = get_manifest_info_by_id(&app, id);

    if manifest.is_some() {
        let m = manifest.unwrap();
        update_manifest_enabled_by_id(&app, m.id, enabled);
        Some(true)
    } else {
        None
    }
}

#[cfg(target_os = "linux")]
#[tauri::command]
pub fn list_compatibility_manifests(app: AppHandle) -> Option<String> {
    let manifestss: LinkedHashMap<String, RunnerManifest> = get_compatibilities(&app);
    let mut manifests: Vec<RunnerManifest> = Vec::new();

    for value in manifestss.clone().into_iter().map(|(_, value)| value) { manifests.push(value); }

    if manifests.is_empty() {
        None
    } else {
        let stringified = serde_json::to_string(&manifests).unwrap();
        Some(stringified)
    }
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub fn list_compatibility_manifests(_app: AppHandle) -> Option<String> { None }

#[cfg(target_os = "linux")]
#[tauri::command]
pub fn get_compatibility_manifest_by_manifest_id(app: AppHandle, id: String) -> Option<String> {
    let db_manifest = get_manifest_info_by_id(&app, id.clone());

    if db_manifest.is_some() {
        let dbm = db_manifest.unwrap();
        let manifest = get_compatibility(&app, &dbm.filename);

        if dbm.enabled {
            let m = manifest.unwrap();
            let stringified = serde_json::to_string(&m).unwrap();
            Some(stringified)
        } else {
            None
        }
    } else {
        None
    }
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub fn get_compatibility_manifest_by_manifest_id(_app: AppHandle, _id: String) -> Option<String> { None }