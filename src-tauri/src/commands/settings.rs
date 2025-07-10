use std::fs;
use std::path::Path;
use fischl::download::Extras;
use fischl::utils::extract_archive;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;
use crate::utils::{block_telemetry, get_mi_path_from_game, send_notification};
use crate::utils::db_manager::{get_install_info_by_id, get_manifest_info_by_id, get_settings, update_settings_default_fps_unlock_location, update_settings_default_game_location, update_settings_default_jadeite_location, update_settings_default_prefix_location, update_settings_default_xxmi_location, update_settings_hide_manifests, update_settings_launch_action, update_settings_third_party_repo_update};
use crate::utils::repo_manager::get_manifest;

#[cfg(target_os = "linux")]
use std::os::unix::fs::symlink;
use tauri_plugin_notification::NotificationExt;

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
pub fn update_settings_manifests_hide(app: AppHandle, enabled: bool) -> Option<bool> {
    update_settings_hide_manifests(&app, enabled);
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
        app.notification().builder().icon("dialog-information").title("TwintailLauncher").body("Telemetry servers already blocked.").show().unwrap();
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

        // Pull latest jadeite if installed
        if fs::read_dir(&jadeite).unwrap().next().is_some() {
            std::thread::spawn(move || {
                let dl = Extras::download_jadeite("MrLGamer/jadeite".parse().unwrap(), jadeite.as_path().to_str().unwrap().parse().unwrap());
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

        // Pull latest xxmi and its packages if xxmi is installed
        if fs::read_dir(&xxmi).unwrap().next().is_some() {
            std::thread::spawn(move || {
                let dl = Extras::download_xxmi("SpectrumQT/XXMI-Libs-Package".parse().unwrap(), xxmi.as_path().to_str().unwrap().parse().unwrap(), false);
                if dl {
                    extract_archive(xxmi.join("xxmi.zip").as_path().to_str().unwrap().parse().unwrap(), xxmi.as_path().to_str().unwrap().parse().unwrap(), false);
                    let gimi = String::from("SilentNightSound/GIMI-Package");
                    let srmi = String::from("SpectrumQT/SRMI-Package");
                    let zzmi = String::from("leotorrez/ZZMI-Package");
                    let wwmi = String::from("SpectrumQT/WWMI-Package");
                    let himi = String::from("leotorrez/HIMI-Package");

                    let dl1 = Extras::download_xxmi_packages(gimi, srmi, zzmi, wwmi, himi, xxmi.as_path().to_str().unwrap().parse().unwrap());
                    if dl1 {
                        for mi in ["gimi", "srmi", "zzmi", "wwmi", "himi"] {
                            extract_archive(xxmi.join(format!("{mi}.zip")).as_path().to_str().unwrap().parse().unwrap(), xxmi.join(mi).as_path().to_str().unwrap().parse().unwrap(), false);
                            for lib in ["d3d11.dll", "d3dcompiler_47.dll"] {
                                let linkedpath = xxmi.join(mi).join(lib);
                                if !linkedpath.exists() {
                                    #[cfg(target_os = "linux")]
                                    symlink(xxmi.join(lib), linkedpath).unwrap();
                                    #[cfg(target_os = "windows")]
                                    fs::copy(xxmi.join(lib), linkedpath).unwrap();
                                }
                            }
                        }
                    }
                }
            });
        }
        send_notification(&app, "Successfully updated extras.", None);
        true
    } else {
        false
    }
}

#[tauri::command]
pub fn open_folder(app: AppHandle, manifest_id: String, install_id: String, path_type: String) {
    match path_type.as_str() {
        "mods" => {
            let settings = get_settings(&app);
            if settings.is_some() {
                let s = settings.unwrap();
                let m = get_manifest_info_by_id(&app, manifest_id).unwrap();
                let mm = get_manifest(&app, m.filename).unwrap();
                let fm = get_mi_path_from_game(mm.paths.exe_filename).unwrap();

                let xxmi = Path::new(&s.xxmi_path).to_path_buf();
                let fp = xxmi.join(&fm).join("d3dx.ini");
                if fp.exists() {
                    match app.opener().reveal_item_in_dir(fp.as_path()) {
                        Ok(_) => {}
                        Err(_e) => { send_notification(&app, "Directory opening failed, try again later!", None); }
                    }
                } else {
                    send_notification(&app, "XXMI is not downloaded or folder structure is corrupt! Can not open the folder.", None);
                };
            }
        },
        "install" => {
            let install = get_install_info_by_id(&app, install_id);
            if install.is_some() {
                let i = install.unwrap();
                let fp = Path::new(&i.directory).join("game.log");
                if fp.exists() {
                    match app.opener().reveal_item_in_dir(fp.as_path()) {
                        Ok(_) => {}
                        Err(_e) => { send_notification(&app, "Directory opening failed, try again later!", None); }
                    }
                } else {
                    send_notification(&app, "Can not open game directory, Please run the game once!", None);
                };
            }
        },
        _ => {}
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
    pub launcher_action: String,
    pub hide_manifests: bool
}