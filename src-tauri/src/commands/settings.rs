use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle};
use crate::utils::db_manager::{get_settings, update_settings_default_fps_unlock_location, update_settings_default_game_location, update_settings_default_jadeite_location, update_settings_default_prefix_location, update_settings_default_xxmi_location, update_settings_launch_action, update_settings_third_party_repo_update};

#[tauri::command]
pub async fn list_settings(app: AppHandle) -> Option<String> {
    let settings = get_settings(&app);

    if settings.is_some() {
        let s = settings.unwrap();
        let stringified = serde_json::to_string(&s).unwrap();
        Some(stringified)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_settings_third_party_repo_updates(app: AppHandle, enabled: bool) -> Option<bool> {
    update_settings_third_party_repo_update(&app, enabled);
    Some(true)
}

#[tauri::command]
pub fn update_settings_default_game_path(app: AppHandle, path: String) -> Option<bool> {
    let p = Path::new(&path);

    if !p.exists() && p.is_dir() {
        fs::create_dir_all(&p).unwrap();
        update_settings_default_game_location(&app, p.to_str().unwrap().parse().unwrap());
    } else {
        update_settings_default_game_location(&app, p.to_str().unwrap().parse().unwrap());
    }
    Some(true)
}

#[tauri::command]
pub fn update_settings_default_xxmi_path(app: AppHandle, path: String) -> Option<bool> {
    let p = Path::new(&path);

    if !p.exists() && p.is_dir() {
        fs::create_dir_all(&p).unwrap();
        update_settings_default_xxmi_location(&app, p.to_str().unwrap().parse().unwrap());
    } else {
        update_settings_default_xxmi_location(&app, p.to_str().unwrap().parse().unwrap());
    }
    Some(true)
}

#[tauri::command]
pub fn update_settings_default_fps_unlock_path(app: AppHandle, path: String) -> Option<bool> {
    let p = Path::new(&path);

    if !p.exists() && p.is_dir() {
        fs::create_dir_all(&p).unwrap();
        update_settings_default_fps_unlock_location(&app, p.to_str().unwrap().parse().unwrap());
    } else {
        update_settings_default_fps_unlock_location(&app, p.to_str().unwrap().parse().unwrap());
    }
    Some(true)
}

#[tauri::command]
pub fn update_settings_default_jadeite_path(app: AppHandle, path: String) -> Option<bool> {
    let p = Path::new(&path);

    if !p.exists() && p.is_dir() {
        fs::create_dir_all(&p).unwrap();
        update_settings_default_jadeite_location(&app, p.to_str().unwrap().parse().unwrap());
    } else {
        update_settings_default_jadeite_location(&app, p.to_str().unwrap().parse().unwrap());
    }
    Some(true)
}

#[tauri::command]
pub fn update_settings_default_prefix_path(app: AppHandle, path: String) -> Option<bool> {
    let p = Path::new(&path);

    if !p.exists() && p.is_dir() {
        fs::create_dir_all(&p).unwrap();
        update_settings_default_prefix_location(&app, p.to_str().unwrap().parse().unwrap());
    } else {
        update_settings_default_prefix_location(&app, p.to_str().unwrap().parse().unwrap());
    }
    Some(true)
}

#[tauri::command]
pub fn update_settings_launcher_action(app: AppHandle, action: String) -> Option<bool> {
    update_settings_launch_action(&app, action);
    Some(true)
}

// === STRUCTS ===

#[derive(Serialize, Deserialize, Debug)]
pub struct GlobalSettings {
    pub default_game_path: String,
    pub xxmi_path: String,
    pub fps_unlock_path: String,
    pub jadeite_path: String,
    pub third_party_repo_updates: i32,
    pub default_runner_prefix_path: String,
    pub launcher_action: String
}