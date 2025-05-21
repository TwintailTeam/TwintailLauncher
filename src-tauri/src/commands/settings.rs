use std::fs;
use std::path::Path;
use fischl::download::Extras;
use fischl::utils::extract_archive;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use crate::utils::block_telemetry;
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

#[tauri::command]
pub fn block_telemetry_cmd(app: AppHandle) -> Option<bool> {
    let path = app.path().app_data_dir().unwrap().join(".telemetry_blocked");
    if !path.exists() {
        fs::write(&path, ".").unwrap();
        block_telemetry(&app);
        Some(true)
    } else {
        app.emit("telemetry_block", 2).unwrap();
        None
    }
}

#[tauri::command]
pub fn update_extras(app: AppHandle) -> bool {
    let settings = get_settings(&app);
    if settings.is_some() {
        let s = settings.unwrap();
        let xxmi = Path::new(&s.xxmi_path).to_path_buf();
        let jadeite = Path::new(&s.jadeite_path).to_path_buf();
        let fpsu = Path::new(&s.fps_unlock_path).to_path_buf();

        // Pull latest xxmi and its packages if xxmi is installed
        if fs::read_dir(&xxmi).unwrap().next().is_some() {
            std::thread::spawn(move || {
                let dl = Extras::download_xxmi("SpectrumQT/XXMI-Libs-Package".parse().unwrap(), xxmi.as_path().to_str().unwrap().parse().unwrap(), true);
                if dl {
                    extract_archive(xxmi.join("xxmi.zip").as_path().to_str().unwrap().parse().unwrap(), xxmi.as_path().to_str().unwrap().parse().unwrap(), false);
                    let gimi = String::from("TTL-extras/GIMI-Package");
                    let srmi = String::from("TTL-extras/SRMI-Package");
                    let zzmi = String::from("TTL-extras/ZZMI-Package");
                    let wwmi = String::from("TTL-extras/WWMI-Package");

                    let dl1 = Extras::download_xxmi_packages(gimi, srmi, zzmi, wwmi, xxmi.as_path().to_str().unwrap().parse().unwrap(), true);
                    if dl1 {
                        extract_archive(xxmi.join("gimi.zip").as_path().to_str().unwrap().parse().unwrap(), xxmi.join("gimi").as_path().to_str().unwrap().parse().unwrap(), false);
                        extract_archive(xxmi.join("srmi.zip").as_path().to_str().unwrap().parse().unwrap(), xxmi.join("srmi").as_path().to_str().unwrap().parse().unwrap(), false);
                        extract_archive(xxmi.join("zzmi.zip").as_path().to_str().unwrap().parse().unwrap(), xxmi.join("zzmi").as_path().to_str().unwrap().parse().unwrap(), false);
                        extract_archive(xxmi.join("wwmi.zip").as_path().to_str().unwrap().parse().unwrap(), xxmi.join("wwmi").as_path().to_str().unwrap().parse().unwrap(), false);
                    }
                }
            });
        }

        // Pull latest jadeite if installed
        if fs::read_dir(&jadeite).unwrap().next().is_some() {
            std::thread::spawn(move || {
                let dl = Extras::download_jadeite("mkrsym1/jadeite".parse().unwrap(), jadeite.as_path().to_str().unwrap().parse().unwrap());
                if dl {
                    extract_archive(jadeite.join("jadeite.zip").as_path().to_str().unwrap().parse().unwrap(), jadeite.as_path().to_str().unwrap().parse().unwrap(), false);
                }
            });
        }

        // Pull latest fps unlock if installed
        if fs::read_dir(&fpsu).unwrap().next().is_some() {
            std::thread::spawn(move || {
                Extras::download_fps_unlock("mkrsym1/fpsunlock".parse().unwrap(), fpsu.as_path().to_str().unwrap().parse().unwrap());
            });
        }
        true
    } else {
        false
    }
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