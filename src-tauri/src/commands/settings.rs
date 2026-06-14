use crate::utils::db_manager::{get_install_info_by_id, get_installed_runner_info_by_version, get_manifest_info_by_id, get_settings, update_settings_app_lang, update_settings_default_dxvk_location, update_settings_default_fps_unlock_location, update_settings_default_game_location, update_settings_default_jadeite_location, update_settings_default_mangohud_config_location, update_settings_default_prefix_location, update_settings_default_runner_location, update_settings_default_xxmi_location, update_settings_download_speed_limit, update_settings_hide_app_to_tray, update_settings_hide_manifests, update_settings_launch_action, update_settings_third_party_repo_update};
use crate::utils::models::GlobalSettings;
use crate::utils::repo_manager::get_manifest;
use crate::utils::{compare_version, get_mi_path_from_game, show_dialog_with_callback};
use std::fs;
use std::path::Path;
use tauri::{AppHandle, Runtime,Manager};
use tauri_plugin_opener::OpenerExt;

#[cfg(target_os = "linux")]
use std::os::unix::process::CommandExt;

#[tauri::command]
pub async fn list_settings<R: Runtime>(app: AppHandle<R>) -> Option<GlobalSettings> {
    let settings = get_settings(&app);

    if settings.is_some() {
        let s = settings.unwrap();
        // Ensure fischl's global limiter is synced with persisted settings (value in KB/s).
        fischl::utils::downloader::set_global_download_speed_limit_kb(s.download_speed_limit.max(0) as u64);
        Some(s)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_settings_download_speed_limit_cmd<R: Runtime>(app: AppHandle<R>, speed_limit: i64) -> Option<bool> {
    let clamped = speed_limit.max(0);
    update_settings_download_speed_limit(&app, clamped);
    fischl::utils::downloader::set_global_download_speed_limit_kb(clamped as u64);
    Some(true)
}

#[tauri::command]
pub fn update_settings_third_party_repo_updates<R: Runtime>(app: AppHandle<R>, enabled: bool) -> Option<bool> {
    update_settings_third_party_repo_update(&app, enabled);
    Some(true)
}

#[tauri::command]
pub fn update_settings_default_game_path<R: Runtime>(app: AppHandle<R>, path: String) -> Option<bool> {
    let p = Path::new(&path);

    if !p.exists() && p.is_dir() {
        fs::create_dir_all(&p).unwrap();
        update_settings_default_game_location(&app, p.to_str().unwrap().parse().unwrap());
    } else {
        update_settings_default_game_location(&app, p.to_str().unwrap().parse().unwrap());
    }
    log::debug!("Updated default game path to {}", path);
    Some(true)
}

#[tauri::command]
pub fn update_settings_default_xxmi_path<R: Runtime>(app: AppHandle<R>, path: String) -> Option<bool> {
    let p = Path::new(&path);

    if !p.exists() && p.is_dir() {
        fs::create_dir_all(&p).unwrap();
        update_settings_default_xxmi_location(&app, p.to_str().unwrap().parse().unwrap());
    } else {
        update_settings_default_xxmi_location(&app, p.to_str().unwrap().parse().unwrap());
    }
    log::debug!("Updated default XXMI path to {}", path);
    Some(true)
}

#[tauri::command]
pub fn update_settings_default_fps_unlock_path<R: Runtime>(app: AppHandle<R>, path: String) -> Option<bool> {
    let p = Path::new(&path);

    if !p.exists() && p.is_dir() {
        fs::create_dir_all(&p).unwrap();
        update_settings_default_fps_unlock_location(&app, p.to_str().unwrap().parse().unwrap());
    } else {
        update_settings_default_fps_unlock_location(&app, p.to_str().unwrap().parse().unwrap());
    }
    log::debug!("Updated default FPS unlock path to {}", path);
    Some(true)
}

#[tauri::command]
pub fn update_settings_default_jadeite_path<R: Runtime>(app: AppHandle<R>, path: String) -> Option<bool> {
    let p = Path::new(&path);

    if !p.exists() && p.is_dir() {
        fs::create_dir_all(&p).unwrap();
        update_settings_default_jadeite_location(&app, p.to_str().unwrap().parse().unwrap());
    } else {
        update_settings_default_jadeite_location(&app, p.to_str().unwrap().parse().unwrap());
    }
    log::debug!("Updated default Jadeite path to {}", path);
    Some(true)
}

#[tauri::command]
pub fn update_settings_default_prefix_path<R: Runtime>(app: AppHandle<R>, path: String) -> Option<bool> {
    let p = Path::new(&path);

    if !p.exists() && p.is_dir() {
        fs::create_dir_all(&p).unwrap();
        update_settings_default_prefix_location(&app, p.to_str().unwrap().parse().unwrap());
    } else {
        update_settings_default_prefix_location(&app, p.to_str().unwrap().parse().unwrap());
    }
    log::debug!("Updated default prefix path to {}", path);
    Some(true)
}

#[tauri::command]
pub fn update_settings_default_runner_path<R: Runtime>(app: AppHandle<R>, path: String) -> Option<bool> {
    let p = Path::new(&path);

    if !p.exists() && p.is_dir() {
        fs::create_dir_all(&p).unwrap();
        update_settings_default_runner_location(&app, p.to_str().unwrap().parse().unwrap());
    } else {
        update_settings_default_runner_location(&app, p.to_str().unwrap().parse().unwrap());
    }
    log::debug!("Updated default runner path to {}", path);
    Some(true)
}

#[tauri::command]
pub fn update_settings_default_dxvk_path<R: Runtime>(app: AppHandle<R>, path: String) -> Option<bool> {
    let p = Path::new(&path);

    if !p.exists() && p.is_dir() {
        fs::create_dir_all(&p).unwrap();
        update_settings_default_dxvk_location(&app, p.to_str().unwrap().parse().unwrap());
    } else {
        update_settings_default_dxvk_location(&app, p.to_str().unwrap().parse().unwrap());
    }
    log::debug!("Updated default DXVK path to {}", path);
    Some(true)
}

#[tauri::command]
pub fn update_settings_default_mangohud_config_path<R: Runtime>(app: AppHandle<R>, path: String) -> Option<bool> {
    let p = Path::new(&path);

    if !p.exists() && p.is_file() {
        fs::create_dir_all(&p).unwrap();
        update_settings_default_mangohud_config_location(&app, p.to_str().unwrap().parse().unwrap());
    } else {
        update_settings_default_mangohud_config_location(&app, p.to_str().unwrap().parse().unwrap());
    }
    log::debug!("Updated default MangoHUD config path to {}", path);
    Some(true)
}

#[tauri::command]
pub fn update_settings_launcher_action<R: Runtime>(app: AppHandle<R>, action: String) -> Option<bool> {
    update_settings_launch_action(&app, action);
    Some(true)
}

#[tauri::command]
pub fn update_settings_manifests_hide<R: Runtime>(app: AppHandle<R>, enabled: bool) -> Option<bool> {
    update_settings_hide_manifests(&app, enabled);
    Some(true)
}

#[tauri::command]
pub fn update_settings_hide_app_tray<R: Runtime>(app: AppHandle<R>, enabled: bool) -> Option<bool> {
    update_settings_hide_app_to_tray(&app, enabled);
    Some(true)
}

#[tauri::command]
pub fn open_folder<R: Runtime>(app: AppHandle<R>, manifest_id: String, install_id: String, runner_version: String, path_type: String) {
    log::debug!("Opening {} folder for install {}", path_type, install_id);
    match path_type.as_str() {
        "mods" => {
            let settings = get_settings(&app);
            if settings.is_some() {
                let s = settings.unwrap();
                let m = get_manifest_info_by_id(&app, manifest_id).unwrap();
                let mm = get_manifest(&app, m.filename).unwrap();
                let fm = get_mi_path_from_game(mm.paths.exe_filename).unwrap();

                let xxmi = Path::new(&s.xxmi_path);
                let fp = xxmi.join(&fm).join("d3dx.ini");
                if fp.exists() {
                    match app.opener().reveal_item_in_dir(fp.as_path()) {
                        Ok(_) => {}
                        Err(_e) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.directory_open_failed", None, None, None); }
                    }
                } else { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.xxmi_folder_not_found", None, None, None); };
            }
        }
        "install" => {
            let install = get_install_info_by_id(&app, install_id);
            if install.is_some() {
                let i = install.unwrap();
                let dm = get_manifest_info_by_id(&app, i.manifest_id).unwrap();
                let gm = get_manifest(&app, dm.filename).unwrap();
                let fp = Path::new(&i.directory).join(gm.paths.exe_filename);
                if fp.exists() {
                    match app.opener().reveal_item_in_dir(fp.as_path()) {
                        Ok(_) => {}
                        Err(_e) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.directory_open_failed", None, None, None); }
                    }
                } else { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.game_dir_open_failed", None, None, None); };
            }
        }
        "runner" => {
            let install = get_install_info_by_id(&app, install_id);
            if install.is_some() {
                let i = install.unwrap();
                let fp = Path::new(&i.runner_path).join("proton");
                if fp.exists() {
                    match app.opener().reveal_item_in_dir(fp.as_path()) {
                        Ok(_) => {}
                        Err(_e) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.directory_open_failed", None, None, None); }
                    }
                } else { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.runner_dir_open_failed", None, None, None); };
            }
        }
        "runner_global" => {
            let runner = get_installed_runner_info_by_version(&app, runner_version);
            if runner.is_some() {
                let i = runner.unwrap();
                let fp = Path::new(&i.runner_path).join("proton");
                if fp.exists() {
                    match app.opener().reveal_item_in_dir(fp.as_path()) {
                        Ok(_) => {}
                        Err(_e) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.directory_open_failed", None, None, None); }
                    }
                } else { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.runner_dir_open_failed", None, None, None); }
            }
        }
        "runner_prefix" => {
            let install = get_install_info_by_id(&app, install_id);
            if install.is_some() {
                let i = install.unwrap();
                let fp = Path::new(&i.runner_prefix).join("version");
                if fp.exists() {
                    match app.opener().reveal_item_in_dir(fp.as_path()) {
                        Ok(_) => {}
                        Err(_e) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.directory_open_failed", None, None, None); }
                    }
                } else { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.prefix_dir_open_failed", None, None, None); };
            }
        }
        "engine_log" => {
            let install = get_install_info_by_id(&app, install_id);
            if install.is_some() {
                let i = install.unwrap();
                let manifest = get_manifest_info_by_id(&app, i.manifest_id).unwrap();
                let gm = get_manifest(&app, manifest.filename).unwrap();
                let game_dir = Path::new(&i.directory).to_path_buf();
                #[cfg(target_os = "linux")]
                {
                    let prefix = Path::new(&i.runner_prefix).to_path_buf();
                    let prefix_exists = prefix.join("pfx/").exists();
                    if prefix_exists {
                        let base = if gm.biz != "wuwa_global" { prefix.join("pfx/drive_c/users/steamuser/AppData/LocalLow/") } else { game_dir };
                        let engine_log = base.join(crate::utils::get_engine_log_from_game(base.to_str().unwrap().to_string(), gm.biz, i.region_code));
                        if engine_log.exists() {
                            match app.opener().reveal_item_in_dir(engine_log.as_path()) {
                                Ok(_) => {}
                                Err(_e) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.directory_open_failed", None, None, None); }
                            }
                        } else { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.engine_log_dir_open_failed", None, None, None); }
                    } else { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.prefix_dir_open_failed", None, None, None); }
                }

                #[cfg(target_os = "windows")]
                {
                    let base = if gm.biz != "wuwa_global" { app.path().home_dir().unwrap().join("AppData/LocalLow/") } else { game_dir };
                    let engine_log = base.join(crate::utils::get_engine_log_from_game(base.to_str().unwrap().to_string(), gm.biz, i.region_code));
                    if engine_log.exists() {
                        match app.opener().reveal_item_in_dir(engine_log.as_path()) {
                            Ok(_) => {}
                            Err(_e) => { crate::utils::show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.directory_open_failed", None, None, None); }
                        }
                    } else { crate::utils::show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.engine_log_dir_open_failed", None, None, None); }
                }
            }
        }
        _ => {}
    }
}

#[tauri::command]
pub fn empty_folder<R: Runtime>(app: AppHandle<R>, install_id: String, path_type: String) {
    match path_type.as_str() {
        "runner_prefix" => {
            let install = get_install_info_by_id(&app, install_id);
            if install.is_some() {
                let i = install.unwrap();
                let fp = Path::new(&i.runner_prefix);
                if fp.exists() {
                    match crate::utils::empty_dir(fp) {
                        Ok(_) => {
                            log::info!("Cleared runner prefix for installation {}", i.id);
                            show_dialog_with_callback(&app, "info", "TwintailLauncher", "dialogs.prefix_repair_queued", None, None, None);
                        }
                        Err(_) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.prefix_repair_failed", None, None, None); }
                    }
                } else { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.prefix_not_initialized", None, None, None); };
            }
        }
        "steamrt" => {
            let gs = get_settings(&app).unwrap();
            let steamrt3 = Path::new(&gs.default_runner_path).join("steamrt/steamrt3/");
            if steamrt3.exists() {
                match crate::utils::empty_dir(steamrt3) {
                    Ok(_) => { log::info!("Cleared SteamRT3 for repair"); }
                    Err(_) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.steamrt3_repair_failed", None, None, None); }
                }
            } else { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.steamrt3_not_downloaded", None, None, None); };

            let steamrt4 = Path::new(&gs.default_runner_path).join("steamrt/steamrt4/");
            if steamrt4.exists() {
                match crate::utils::empty_dir(steamrt4) {
                    Ok(_) => { log::info!("Cleared SteamRT4 for repair"); }
                    Err(_) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.steamrt4_repair_failed", None, None, None); }
                }
            } else { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.steamrt4_not_downloaded", None, None, None); };
            show_dialog_with_callback(&app, "info", "TwintailLauncher", "dialogs.steamrt_repair_restart", Some(vec!["dialogs.buttons.restart_now"]), Some("dialog_steamrt_repair"), None);
        }
        _ => {}
    }
}

#[allow(unused_variables)]
#[tauri::command]
pub fn open_in_prefix<R: Runtime>(app: AppHandle<R>, install_id: String, path_type: String) {
    log::info!("Spawning {} in prefix for installation {}", path_type, install_id);
    match path_type.as_str() {
        "regedit.exe" => {
            #[cfg(target_os = "linux")] {
                let install = get_install_info_by_id(&app, install_id);
                if install.is_some() {
                    let i = install.unwrap();
                    let fp = Path::new(&i.runner_path);
                    let rp = Path::new(&i.runner_prefix).join("pfx/");
                    if fp.exists() {
                        if !rp.exists() { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.regedit_prefix_not_initialized", None, None, None); return; }
                        let runnerparent = fp.parent().unwrap().to_path_buf();
                        let toolid = crate::utils::get_steam_tool_appid(fp.to_path_buf());
                        let steamrtpp = runnerparent.join("steamrt/").join(toolid.clone());
                        let steamrtp = steamrtpp.join("_v2-entry-point");
                        let steamrt = steamrtp.to_str().unwrap().to_string();
                        #[cfg(not(debug_assertions))]
                        let reaper = if crate::utils::is_flatpak() { app.path().resource_dir().unwrap().join("resources/reaper").to_str().unwrap().to_string().replace("/app/lib/", "/run/parent/app/lib/") } else { app.path().resource_dir().unwrap().join("resources/reaper").to_str().unwrap().to_string().replace("/usr/lib/", "/run/host/usr/lib/") };
                        #[cfg(debug_assertions)]
                        let reaper = app.path().resource_dir().unwrap().join("resources/reaper").to_str().unwrap().to_string();
                        let appid = crate::utils::get_steam_appid();

                        let dir = i.directory.clone();
                        let prefix = i.runner_prefix.clone();
                        let runner = fp.to_str().unwrap().to_string();
                        let command = format!("'{steamrt}' --verb=run -- '{reaper}' SteamLaunch AppId={appid} -- '{runner}/proton' run 'regedit.exe'");

                        let mut cmd = std::process::Command::new("bash");
                        cmd.arg("-c");
                        cmd.arg(&command);

                        cmd.env("WINEARCH", "win64");
                        cmd.env("WINEPREFIX", prefix.clone() + "/pfx");
                        cmd.env("STEAM_COMPAT_APP_ID", "0");
                        cmd.env("STEAM_COMPAT_DATA_PATH", prefix.clone());
                        cmd.env("STEAM_COMPAT_INSTALL_PATH", dir.clone());
                        cmd.env("STEAM_COMPAT_CLIENT_INSTALL_PATH", "");
                        cmd.env("STEAM_COMPAT_TOOL_PATHS", runner.clone());
                        cmd.env("STEAM_COMPAT_SHADER_PATH", prefix.clone() + "/shadercache");
                        cmd.env("WINEDLLOVERRIDES", "lsteamclient=d;KRSDKExternal.exe=d");
                        cmd.env("PROTONFIXES_DISABLE", "1");
                        cmd.env("PROTON_USE_XALIA", "0");

                        cmd.stdout(std::process::Stdio::null());
                        cmd.stderr(std::process::Stdio::null());
                        cmd.current_dir(dir.clone());
                        cmd.process_group(0);

                        match cmd.spawn() {
                            Ok(mut child) => match child.try_wait() {
                                Ok(Some(status)) => {
                                    if !status.success() { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.regedit_exec_failed", None, None, None); }
                                }
                                Ok(None) => {}
                                Err(_) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.regedit_exec_incorrect", None, None, None); }
                            },
                            Err(_) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.regedit_exec_critical", None, None, None); }
                        }
                    } else { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.regedit_runner_not_found", None, None, None); };
                }
            }
        }
        "control.exe" => {
            #[cfg(target_os = "linux")]
            {
                let install = get_install_info_by_id(&app, install_id);
                if install.is_some() {
                    let i = install.unwrap();
                    let fp = Path::new(&i.runner_path);
                    let rp = Path::new(&i.runner_prefix).join("pfx/");
                    if fp.exists() {
                        if !rp.exists() { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.control_prefix_not_initialized", None, None, None); return; }
                        let runnerparent = fp.parent().unwrap().to_path_buf();
                        let toolid = crate::utils::get_steam_tool_appid(fp.to_path_buf());
                        let steamrtpp = runnerparent.join("steamrt/").join(toolid.clone());
                        let steamrtp = steamrtpp.join("_v2-entry-point");
                        let steamrt = steamrtp.to_str().unwrap().to_string();
                        #[cfg(not(debug_assertions))]
                        let reaper = if crate::utils::is_flatpak() { app.path().resource_dir().unwrap().join("resources/reaper").to_str().unwrap().to_string().replace("/app/lib/", "/run/parent/app/lib/") } else { app.path().resource_dir().unwrap().join("resources/reaper").to_str().unwrap().to_string().replace("/usr/lib/", "/run/host/usr/lib/") };
                        #[cfg(debug_assertions)]
                        let reaper = app.path().resource_dir().unwrap().join("resources/reaper").to_str().unwrap().to_string();
                        let appid = crate::utils::get_steam_appid();

                        let dir = i.directory.clone();
                        let prefix = i.runner_prefix.clone();
                        let runner = fp.to_str().unwrap().to_string();
                        let command = format!("'{steamrt}' --verb=run -- '{reaper}' SteamLaunch AppId={appid} -- '{runner}/proton' run 'control.exe'");

                        let mut cmd = std::process::Command::new("bash");
                        cmd.arg("-c");
                        cmd.arg(&command);

                        cmd.env("WINEARCH", "win64");
                        cmd.env("WINEPREFIX", prefix.clone() + "/pfx");
                        cmd.env("STEAM_COMPAT_APP_ID", "0");
                        cmd.env("STEAM_COMPAT_DATA_PATH", prefix.clone());
                        cmd.env("STEAM_COMPAT_INSTALL_PATH", dir.clone());
                        cmd.env("STEAM_COMPAT_CLIENT_INSTALL_PATH", "");
                        cmd.env("STEAM_COMPAT_TOOL_PATHS", runner.clone());
                        cmd.env("STEAM_COMPAT_SHADER_PATH", prefix.clone() + "/shadercache");
                        cmd.env("WINEDLLOVERRIDES", "lsteamclient=d;KRSDKExternal.exe=d");
                        cmd.env("PROTONFIXES_DISABLE", "1");
                        cmd.env("PROTON_USE_XALIA", "0");

                        cmd.stdout(std::process::Stdio::null());
                        cmd.stderr(std::process::Stdio::null());
                        cmd.current_dir(dir.clone());
                        cmd.process_group(0);

                        match cmd.spawn() {
                            Ok(mut child) => match child.try_wait() {
                                Ok(Some(status)) => {
                                    if !status.success() { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.control_exec_failed", None, None, None); }
                                }
                                Ok(None) => {}
                                Err(_) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.control_exec_incorrect", None, None, None); }
                            },
                            Err(_) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.control_exec_critical", None, None, None); }
                        }
                    } else { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.control_runner_not_found", None, None, None); };
                }
            }
        }
        "cmd.exe" => {
            #[cfg(target_os = "linux")]
            {
                let install = get_install_info_by_id(&app, install_id);
                if install.is_some() {
                    let i = install.unwrap();
                    let fp = Path::new(&i.runner_path);
                    let rp = Path::new(&i.runner_prefix).join("pfx/");
                    if fp.exists() {
                        if !rp.exists() { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.cmd_prefix_not_initialized", None, None, None); return; }
                        let runnerparent = fp.parent().unwrap().to_path_buf();
                        let toolid = crate::utils::get_steam_tool_appid(fp.to_path_buf());
                        let steamrtpp = runnerparent.join("steamrt/").join(toolid.clone());
                        let steamrtp = steamrtpp.join("_v2-entry-point");
                        let steamrt = steamrtp.to_str().unwrap().to_string();
                        #[cfg(not(debug_assertions))]
                        let reaper = if crate::utils::is_flatpak() { app.path().resource_dir().unwrap().join("resources/reaper").to_str().unwrap().to_string().replace("/app/lib/", "/run/parent/app/lib/") } else { app.path().resource_dir().unwrap().join("resources/reaper").to_str().unwrap().to_string().replace("/usr/lib/", "/run/host/usr/lib/") };
                        #[cfg(debug_assertions)]
                        let reaper = app.path().resource_dir().unwrap().join("resources/reaper").to_str().unwrap().to_string();
                        let appid = crate::utils::get_steam_appid();

                        let dir = i.directory.clone();
                        let prefix = i.runner_prefix.clone();
                        let runner = fp.to_str().unwrap().to_string();
                        let command = format!("'{steamrt}' --verb=run -- '{reaper}' SteamLaunch AppId={appid} -- '{runner}/proton' run 'cmd.exe'");

                        let mut cmd = std::process::Command::new("bash");
                        cmd.arg("-c");
                        cmd.arg(&command);

                        cmd.env("WINEARCH", "win64");
                        cmd.env("WINEPREFIX", prefix.clone() + "/pfx");
                        cmd.env("STEAM_COMPAT_APP_ID", "0");
                        cmd.env("STEAM_COMPAT_DATA_PATH", prefix.clone());
                        cmd.env("STEAM_COMPAT_INSTALL_PATH", dir.clone());
                        cmd.env("STEAM_COMPAT_CLIENT_INSTALL_PATH", "");
                        cmd.env("STEAM_COMPAT_TOOL_PATHS", runner.clone());
                        cmd.env("STEAM_COMPAT_SHADER_PATH", prefix.clone() + "/shadercache");
                        cmd.env("WINEDLLOVERRIDES", "lsteamclient=d;KRSDKExternal.exe=d");
                        cmd.env("PROTONFIXES_DISABLE", "1");
                        cmd.env("PROTON_USE_XALIA", "0");

                        cmd.stdout(std::process::Stdio::null());
                        cmd.stderr(std::process::Stdio::null());
                        cmd.current_dir(dir.clone());
                        cmd.process_group(0);

                        match cmd.spawn() {
                            Ok(mut child) => match child.try_wait() {
                                Ok(Some(status)) => {
                                    if !status.success() { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.cmd_exec_failed", None, None, None); }
                                }
                                Ok(None) => {}
                                Err(_) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.cmd_exec_incorrect", None, None, None); }
                            },
                            Err(_) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.cmd_exec_critical", None, None, None); }
                        }
                    } else { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.cmd_runner_not_found", None, None, None); };
                }
            }
        }
        "winecfg.exe" => {
            #[cfg(target_os = "linux")]
            {
                let install = get_install_info_by_id(&app, install_id);
                if install.is_some() {
                    let i = install.unwrap();
                    let fp = Path::new(&i.runner_path);
                    let rp = Path::new(&i.runner_prefix).join("pfx/");
                    if fp.exists() {
                        if !rp.exists() { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.winecfg_prefix_not_initialized", None, None, None); return; }
                        let runnerparent = fp.parent().unwrap().to_path_buf();
                        let toolid = crate::utils::get_steam_tool_appid(fp.to_path_buf());
                        let steamrtpp = runnerparent.join("steamrt/").join(toolid.clone());
                        let steamrtp = steamrtpp.join("_v2-entry-point");
                        let steamrt = steamrtp.to_str().unwrap().to_string();
                        #[cfg(not(debug_assertions))]
                        let reaper = if crate::utils::is_flatpak() { app.path().resource_dir().unwrap().join("resources/reaper").to_str().unwrap().to_string().replace("/app/lib/", "/run/parent/app/lib/") } else { app.path().resource_dir().unwrap().join("resources/reaper").to_str().unwrap().to_string().replace("/usr/lib/", "/run/host/usr/lib/") };
                        #[cfg(debug_assertions)]
                        let reaper = app.path().resource_dir().unwrap().join("resources/reaper").to_str().unwrap().to_string();
                        let appid = crate::utils::get_steam_appid();

                        let dir = i.directory.clone();
                        let prefix = i.runner_prefix.clone();
                        let runner = fp.to_str().unwrap().to_string();
                        let command = format!("'{steamrt}' --verb=run -- '{reaper}' SteamLaunch AppId={appid} -- '{runner}/proton' run 'winecfg.exe'");

                        let mut cmd = std::process::Command::new("bash");
                        cmd.arg("-c");
                        cmd.arg(&command);

                        cmd.env("WINEARCH", "win64");
                        cmd.env("WINEPREFIX", prefix.clone() + "/pfx");
                        cmd.env("STEAM_COMPAT_APP_ID", "0");
                        cmd.env("STEAM_COMPAT_DATA_PATH", prefix.clone());
                        cmd.env("STEAM_COMPAT_INSTALL_PATH", dir.clone());
                        cmd.env("STEAM_COMPAT_CLIENT_INSTALL_PATH", "");
                        cmd.env("STEAM_COMPAT_TOOL_PATHS", runner.clone());
                        cmd.env("STEAM_COMPAT_SHADER_PATH", prefix.clone() + "/shadercache");
                        cmd.env("WINEDLLOVERRIDES", "lsteamclient=d;KRSDKExternal.exe=d");
                        cmd.env("PROTONFIXES_DISABLE", "1");
                        cmd.env("PROTON_USE_XALIA", "0");

                        cmd.stdout(std::process::Stdio::null());
                        cmd.stderr(std::process::Stdio::null());
                        cmd.current_dir(dir.clone());
                        cmd.process_group(0);

                        match cmd.spawn() {
                            Ok(mut child) => match child.try_wait() {
                                Ok(Some(status)) => {
                                    if !status.success() { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.winecfg_exec_failed", None, None, None); }
                                }
                                Ok(None) => {}
                                Err(_) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.winecfg_exec_incorrect", None, None, None); }
                            },
                            Err(_) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.winecfg_exec_critical", None, None, None); }
                        }
                    } else { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.winecfg_runner_not_found", None, None, None); };
                }
            }
        }
        // Diagnostics and logs
        "steamrt3" => {
            #[cfg(target_os = "linux")]
            {
                let gs = get_settings(&app).unwrap();
                let fp = Path::new(&gs.default_runner_path).join("steamrt/steamrt3/");
                if fp.exists() {
                    log::info!("Running SteamRT3 diagnostics");
                    let steamrtp = fp.join("run");
                    let steamrt = steamrtp.to_str().unwrap().to_string();

                    let log_path = app.path().app_log_dir().unwrap().join("custom/");
                    if !log_path.exists() { let _ = fs::create_dir_all(&log_path); }
                    let log_path_file = log_path.join("steamrt3_diagnostics.log");
                    if log_path_file.exists() { let _ = fs::remove_file(&log_path_file); }
                    let log_file = fs::File::create(&log_path_file).expect("Failed to create log file");
                    let log_file_stderr = log_file.try_clone().expect("Failed to clone log file handle");

                    let command = format!("'{steamrt}' steam-runtime-system-info --verbose");
                    let mut cmd = std::process::Command::new("bash");
                    cmd.arg("-c");
                    cmd.arg(&command);

                    cmd.stdout(std::process::Stdio::from(log_file));
                    cmd.stderr(std::process::Stdio::from(log_file_stderr));
                    cmd.current_dir(fp.clone());
                    cmd.process_group(0);

                    match cmd.spawn() {
                        Ok(mut child) => match child.try_wait() {
                            Ok(Some(status)) => {
                                if !status.success() { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.steamrt3_diagnostics_failed", None, None, None); }
                            }
                            Ok(None) => {}
                            Err(_) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.steamrt_diagnostics_exec_incorrect", None, None, None); }
                        },
                        Err(e) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.steamrt_diagnostics_exec_critical", None, None, None); }
                    }
                    match app.opener().reveal_item_in_dir(log_path_file.as_path()) {
                        Ok(_) => {}
                        Err(_e) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.steamrt3_log_open_failed", None, None, None); }
                    }
                } else { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.steamrt3_not_downloaded_diag", None, None, None); };
            }
        }
        "steamrt4" => {
            #[cfg(target_os = "linux")]
            {
                let gs = get_settings(&app).unwrap();
                let fp = Path::new(&gs.default_runner_path).join("steamrt/steamrt4/");
                if fp.exists() {
                    log::info!("Running SteamRT4 diagnostics");
                    let steamrtp = fp.join("run");
                    let steamrt = steamrtp.to_str().unwrap().to_string();

                    let log_path = app.path().app_log_dir().unwrap().join("custom/");
                    if !log_path.exists() { let _ = fs::create_dir_all(&log_path); }
                    let log_path_file = log_path.join("steamrt4_diagnostics.log");
                    if log_path_file.exists() { let _ = fs::remove_file(&log_path_file); }
                    let log_file = fs::File::create(&log_path_file).expect("Failed to create log file");
                    let log_file_stderr = log_file.try_clone().expect("Failed to clone log file handle");

                    let command = format!("'{steamrt}' steam-runtime-system-info --verbose");
                    let mut cmd = std::process::Command::new("bash");
                    cmd.arg("-c");
                    cmd.arg(&command);

                    cmd.stdout(std::process::Stdio::from(log_file));
                    cmd.stderr(std::process::Stdio::from(log_file_stderr));
                    cmd.current_dir(fp.clone());
                    cmd.process_group(0);

                    match cmd.spawn() {
                        Ok(mut child) => match child.try_wait() {
                            Ok(Some(status)) => {
                                if !status.success() { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.steamrt4_diagnostics_failed", None, None, None); }
                            }
                            Ok(None) => {}
                            Err(_) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.steamrt_diagnostics_exec_incorrect", None, None, None); }
                        },
                        Err(e) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.steamrt_diagnostics_exec_critical", None, None, None); }
                    }
                    match app.opener().reveal_item_in_dir(log_path_file.as_path()) {
                        Ok(_) => {}
                        Err(_e) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.steamrt4_log_open_failed", None, None, None); }
                    }
                } else { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.steamrt4_not_downloaded_diag", None, None, None); };
            }
        }
        _ => {}
    }
}

#[tauri::command]
pub async fn check_app_update<R: Runtime>(app: AppHandle<R>) -> bool {
    tokio::task::spawn_blocking(move || {
        let Some(r) = fischl::utils::get_github_release("TwintailTeam/TwintailLauncher".to_string()) else { return false; };
        let v = r.tag_name.unwrap_or_default().replace("ttl-v", "");
        let cfg = app.config();
        match compare_version(cfg.version.clone().unwrap().as_str(), v.as_str()) {
            std::cmp::Ordering::Less => { log::info!("You are running outdated version of TwintailLauncher!"); true }
            std::cmp::Ordering::Equal => { log::info!("You are running up to date version of TwintailLauncher!"); false }
            std::cmp::Ordering::Greater => { log::info!("You are running newer version of TwintailLauncher! Is it dev build?"); false }
        }
    }).await.unwrap_or(false)
}

#[tauri::command]
pub fn open_uri<R: Runtime>(app: AppHandle<R>, uri: String) {
    match app.opener().open_url(uri, None::<&str>) {
        Ok(_) => {}
        Err(_e) => {}
    }
}

#[tauri::command]
pub fn update_settings_app_lang_cmd<R: Runtime>(app: AppHandle<R>, lang: String) -> Option<bool> {
    update_settings_app_lang(&app, lang);
    Some(true)
}

#[tauri::command]
pub fn get_locale<R: Runtime>(app: AppHandle<R>, code: String) -> Result<serde_json::Value, String> {
    let path = app.path().resource_dir().unwrap().join("resources/locales").join(format!("{}.json", code));
    let raw = fs::read_to_string(&path).map_err(|e| format!("Failed to read locale {}: {}", code, e))?;
    serde_json::from_str(&raw).map_err(|e| format!("Failed to parse locale {}: {}", code, e))
}

#[tauri::command]
pub fn list_locales<R: Runtime>(app: AppHandle<R>) -> Vec<String> {
    let dir = app.path().resource_dir().unwrap().join("resources/locales");
    let entries = match fs::read_dir(&dir) { Ok(e) => e, Err(_) => return vec![] };
    let mut codes = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let s = name.to_string_lossy();
        if s.ends_with(".json") { codes.push(s.trim_end_matches(".json").to_string()); }
    }
    codes.sort();
    codes
}
