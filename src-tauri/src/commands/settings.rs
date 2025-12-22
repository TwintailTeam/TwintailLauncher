use std::fs;
use std::path::Path;
use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;
use crate::utils::{block_telemetry, download_or_update_fps_unlock, download_or_update_jadeite, download_or_update_xxmi, get_mi_path_from_game, send_notification, PathResolve};
use crate::utils::db_manager::{get_install_info_by_id, get_installed_runner_info_by_version, get_manifest_info_by_id, get_settings, update_settings_default_dxvk_location, update_settings_default_fps_unlock_location, update_settings_default_game_location, update_settings_default_jadeite_location, update_settings_default_mangohud_config_location, update_settings_default_prefix_location, update_settings_default_runner_location, update_settings_default_xxmi_location, update_settings_hide_manifests, update_settings_launch_action, update_settings_third_party_repo_update};
use crate::utils::repo_manager::get_manifest;
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
    let p = Path::new(&path).follow_symlink().unwrap();

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
    let p = Path::new(&path).follow_symlink().unwrap();

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
    let p = Path::new(&path).follow_symlink().unwrap();

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
    let p = Path::new(&path).follow_symlink().unwrap();

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
    let p = Path::new(&path).follow_symlink().unwrap();

    if !p.exists() && p.is_dir() {
        fs::create_dir_all(&p).unwrap();
        update_settings_default_prefix_location(&app, p.to_str().unwrap().parse().unwrap());
    } else {
        update_settings_default_prefix_location(&app, p.to_str().unwrap().parse().unwrap());
    }
    Some(true)
}

#[tauri::command]
pub fn update_settings_default_runner_path(app: AppHandle, path: String) -> Option<bool> {
    let p = Path::new(&path).follow_symlink().unwrap();

    if !p.exists() && p.is_dir() {
        fs::create_dir_all(&p).unwrap();
        update_settings_default_runner_location(&app, p.to_str().unwrap().parse().unwrap());
    } else {
        update_settings_default_runner_location(&app, p.to_str().unwrap().parse().unwrap());
    }
    Some(true)
}

#[tauri::command]
pub fn update_settings_default_dxvk_path(app: AppHandle, path: String) -> Option<bool> {
    let p = Path::new(&path).follow_symlink().unwrap();

    if !p.exists() && p.is_dir() {
        fs::create_dir_all(&p).unwrap();
        update_settings_default_dxvk_location(&app, p.to_str().unwrap().parse().unwrap());
    } else {
        update_settings_default_dxvk_location(&app, p.to_str().unwrap().parse().unwrap());
    }
    Some(true)
}

#[tauri::command]
pub fn update_settings_default_mangohud_config_path(app: AppHandle, path: String) -> Option<bool> {
    let p = Path::new(&path).follow_symlink().unwrap();

    if !p.exists() && p.is_file() {
        fs::create_dir_all(&p).unwrap();
        update_settings_default_mangohud_config_location(&app, p.to_str().unwrap().parse().unwrap());
    } else {
        update_settings_default_mangohud_config_location(&app, p.to_str().unwrap().parse().unwrap());
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
        block_telemetry(&app);
        app.notification().builder().icon("dialog-information").title("TwintailLauncher").body("Updated and fixed telemetry server block.").show().unwrap();
        None
    }
}

#[tauri::command]
pub fn update_extras(app: AppHandle, show_notify: bool) -> bool {
    let settings = get_settings(&app);
    if settings.is_some() {
        let s = settings.unwrap();
        let xxmi = Path::new(&s.xxmi_path).follow_symlink().unwrap().to_path_buf();
        let jadeite = Path::new(&s.jadeite_path).follow_symlink().unwrap().to_path_buf();
        let fpsu = Path::new(&s.fps_unlock_path).follow_symlink().unwrap().to_path_buf();

        download_or_update_jadeite(jadeite, true);
        download_or_update_fps_unlock(fpsu, true);
        download_or_update_xxmi(&app, xxmi, None,true);
        if show_notify { send_notification(&app, "Successfully repaired extras.", None); }
        true
    } else {
        false
    }
}

#[tauri::command]
pub fn open_folder(app: AppHandle, manifest_id: String, install_id: String, runner_version: String, path_type: String) {
    match path_type.as_str() {
        "mods" => {
            let settings = get_settings(&app);
            if settings.is_some() {
                let s = settings.unwrap();
                let m = get_manifest_info_by_id(&app, manifest_id).unwrap();
                let mm = get_manifest(&app, m.filename).unwrap();
                let fm = get_mi_path_from_game(mm.paths.exe_filename).unwrap();

                let xxmi = Path::new(&s.xxmi_path).follow_symlink().unwrap().to_path_buf();
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
                let fp = Path::new(&i.directory).join("game.log").follow_symlink().unwrap().to_path_buf();
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
        "runner" => {
            let install = get_install_info_by_id(&app, install_id);
            if install.is_some() {
                let i = install.unwrap();
                let fp = Path::new(&i.runner_path).join("proton").follow_symlink().unwrap().to_path_buf();
                if fp.exists() {
                    match app.opener().reveal_item_in_dir(fp.as_path()) {
                        Ok(_) => {}
                        Err(_e) => { send_notification(&app, "Directory opening failed, try again later!", None); }
                    }
                } else {
                    send_notification(&app, "Can not open runner directory, Is runner downloaded properly?", None);
                };
            }
        }
        "runner_global" => {
            let runner = get_installed_runner_info_by_version(&app, runner_version);
            if runner.is_some() {
                let i = runner.unwrap();
                let fp = Path::new(&i.runner_path).join("proton").follow_symlink().unwrap().to_path_buf();
                if fp.exists() {
                    match app.opener().reveal_item_in_dir(fp.as_path()) {
                        Ok(_) => {}
                        Err(_e) => { send_notification(&app, "Directory opening failed, try again later!", None); }
                    }
                } else {
                    send_notification(&app, "Can not open runner directory, Is runner downloaded properly?", None);
                }
            }
        }
        "runner_prefix" => {
            let install = get_install_info_by_id(&app, install_id);
            if install.is_some() {
                let i = install.unwrap();
                let fp = Path::new(&i.runner_prefix).join("version").follow_symlink().unwrap().to_path_buf();
                if fp.exists() {
                    match app.opener().reveal_item_in_dir(fp.as_path()) {
                        Ok(_) => {}
                        Err(_e) => { send_notification(&app, "Directory opening failed, try again later!", None); }
                    }
                } else {
                    send_notification(&app, "Can not open runner prefix directory, Is runner prefix initialized?", None);
                };
            }
        }
        _ => {}
    }
}

#[tauri::command]
pub fn open_uri(app: AppHandle, uri: String) {
    match app.opener().open_url(uri, None::<&str>) {
        Ok(_) => {},
        Err(_e) => { send_notification(&app, "Opening URL in browser failed!", None); }
    }
}