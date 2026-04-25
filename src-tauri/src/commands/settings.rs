use crate::utils::db_manager::{get_install_info_by_id, get_installed_runner_info_by_version, get_manifest_info_by_id, get_settings, update_settings_default_dxvk_location, update_settings_default_fps_unlock_location, update_settings_default_game_location, update_settings_default_jadeite_location, update_settings_default_mangohud_config_location, update_settings_default_prefix_location, update_settings_default_runner_location, update_settings_default_xxmi_location, update_settings_download_speed_limit, update_settings_hide_app_to_tray, update_settings_hide_manifests, update_settings_launch_action, update_settings_third_party_repo_update};
use crate::utils::repo_manager::get_manifest;
use crate::utils::{get_mi_path_from_game, show_dialog_with_callback};
use std::fs;
use std::path::Path;
use tauri::{AppHandle,Manager};
use tauri_plugin_opener::OpenerExt;

#[cfg(target_os = "linux")]
use std::os::unix::process::CommandExt;

#[tauri::command]
pub async fn list_settings(app: AppHandle) -> Option<String> {
    let settings = get_settings(&app);

    if settings.is_some() {
        let s = settings.unwrap();
        // Ensure fischl's global limiter is synced with persisted settings (value in KB/s).
        fischl::utils::downloader::set_global_download_speed_limit_kb(s.download_speed_limit.max(0) as u64);
        let stringified = serde_json::to_string(&s).unwrap();
        Some(stringified)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_settings_download_speed_limit_cmd(app: AppHandle, speed_limit: i64) -> Option<bool> {
    let clamped = speed_limit.max(0);
    update_settings_download_speed_limit(&app, clamped);
    fischl::utils::downloader::set_global_download_speed_limit_kb(clamped as u64);
    Some(true)
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
pub fn update_settings_default_runner_path(app: AppHandle, path: String) -> Option<bool> {
    let p = Path::new(&path);

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
    let p = Path::new(&path);

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
    let p = Path::new(&path);

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
pub fn update_settings_hide_app_tray(app: AppHandle, enabled: bool) -> Option<bool> {
    update_settings_hide_app_to_tray(&app, enabled);
    Some(true)
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

                let xxmi = Path::new(&s.xxmi_path);
                let fp = xxmi.join(&fm).join("d3dx.ini");
                if fp.exists() {
                    match app.opener().reveal_item_in_dir(fp.as_path()) {
                        Ok(_) => {}
                        Err(_e) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Directory opening failed, try again later!", None, None); }
                    }
                } else { show_dialog_with_callback(&app, "error", "TwintailLauncher", "XXMI is not downloaded or folder structure is corrupt! Can not open the folder.", None, None); };
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
                        Err(_e) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Directory opening failed, try again later!", None, None); }
                    }
                } else { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Can not open game directory, Please try again later!", None, None); };
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
                        Err(_e) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Directory opening failed, try again later!", None, None); }
                    }
                } else { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Can not open runner directory, Is runner downloaded properly?", None, None); };
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
                        Err(_e) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Directory opening failed, try again later!", None, None); }
                    }
                } else { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Can not open runner directory, Is runner downloaded properly?", None, None); }
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
                        Err(_e) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Directory opening failed, try again later!", None, None); }
                    }
                } else { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Can not open runner prefix directory, Is runner prefix initialized?", None, None); };
            }
        }
        "engine_log" => {
            let install = get_install_info_by_id(&app, install_id);
            if install.is_some() {
                let i = install.unwrap();
                #[cfg(target_os = "linux")]
                {
                    let prefix = Path::new(&i.runner_prefix).to_path_buf();
                    let prefix_exists = prefix.join("pfx/").exists();
                    if prefix_exists {
                        let base = prefix.join("pfx/drive_c/users/steamuser/AppData/LocalLow/");
                        let engine_log = base.join(crate::utils::get_engine_log_from_game(base.to_str().unwrap().to_string(), i.name, i.region_code));
                        if engine_log.exists() {
                            match app.opener().reveal_item_in_dir(engine_log.as_path()) {
                                Ok(_) => {}
                                Err(_e) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Directory opening failed, try again later!", None, None); }
                            }
                        } else { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Can not open game engine log directory, Is runner prefix initialized?", None, None); }
                    } else { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Can not open runner prefix directory, Is runner prefix initialized?", None, None); }
                }

                #[cfg(target_os = "windows")]
                {
                    let base = app.path().home_dir().unwrap().join("AppData/LocalLow/");
                    let engine_log = base.join(crate::utils::get_engine_log_from_game(base.to_str().unwrap().to_string(), i.name, i.region_code));
                    if engine_log.exists() {
                        match app.opener().reveal_item_in_dir(engine_log.as_path()) {
                            Ok(_) => {}
                            Err(_e) => { crate::utils::show_dialog_with_callback(&app, "error", "TwintailLauncher", "Directory opening failed, try again later!", None, None); }
                        }
                    } else { crate::utils::show_dialog_with_callback(&app, "error", "TwintailLauncher", "Can not open game engine log directory, Is runner prefix initialized?", None, None); }
                }
            }
        }
        _ => {}
    }
}

#[tauri::command]
pub fn empty_folder(app: AppHandle, install_id: String, path_type: String) {
    match path_type.as_str() {
        "runner_prefix" => {
            let install = get_install_info_by_id(&app, install_id);
            if install.is_some() {
                let i = install.unwrap();
                let fp = Path::new(&i.runner_prefix);
                if fp.exists() {
                    match crate::utils::empty_dir(fp) {
                        Ok(_) => {}
                        Err(_) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Runner prefix repair failed, try again later!", None, None); }
                    }
                } else { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Can not repair runner prefix directory, Is runner prefix initialized?", None, None); };
            }
        }
        "steamrt" => {
            let gs = get_settings(&app).unwrap();
            let steamrt3 = Path::new(&gs.default_runner_path).join("steamrt/steamrt3/");
            if steamrt3.exists() {
                match crate::utils::empty_dir(steamrt3) {
                    Ok(_) => {}
                    Err(_) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "SteamRT3 repair failed, try again later!", None, None); }
                }
            } else { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Can not repair SteamRT3, Is it properly downloaded?", None, None); };

            let steamrt4 = Path::new(&gs.default_runner_path).join("steamrt/steamrt4/");
            if steamrt4.exists() {
                match crate::utils::empty_dir(steamrt4) {
                    Ok(_) => {}
                    Err(_) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "SteamRT4 repair failed, try again later!", None, None); }
                }
            } else { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Can not repair SteamRT4, Is it properly downloaded?", None, None); };
            show_dialog_with_callback(&app, "info", "TwintailLauncher", "SteamRT has been set into repair state, please restart the application to redownload.", Some(vec!["Restart Now"]), Some("dialog_steamrt_repair"));
        }
        _ => {}
    }
}

#[allow(unused_variables)]
#[tauri::command]
pub fn open_in_prefix(app: AppHandle, install_id: String, path_type: String) {
    match path_type.as_str() {
        "regedit.exe" => {
            #[cfg(target_os = "linux")] {
                let install = get_install_info_by_id(&app, install_id);
                if install.is_some() {
                    let i = install.unwrap();
                    let fp = Path::new(&i.runner_path);
                    let rp = Path::new(&i.runner_prefix).join("pfx/");
                    if fp.exists() {
                        if !rp.exists() { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Can not execute regedit.exe, Please start game at least once!", None, None); return; }
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
                                    if !status.success() { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Failed to execute regedit helper command! Please try again.", None, None); }
                                }
                                Ok(None) => {}
                                Err(_) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Failed to execute regedit helper command! Please try again or check the command correctness.", None, None); }
                            },
                            Err(_) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Failed to execute regedit helper command! Something serious is wrong.", None, None); }
                        }
                    } else { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Can not execute regedit.exe, Is runner downloaded properly?", None, None); };
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
                        if !rp.exists() { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Can not execute control.exe, Please start game at least once!", None, None); return; }
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
                                    if !status.success() { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Failed to execute control helper command! Please try again.", None, None); }
                                }
                                Ok(None) => {}
                                Err(_) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Failed to execute control helper command! Please try again or check the command correctness.", None, None); }
                            },
                            Err(_) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Failed to execute control helper command! Something serious is wrong.", None, None); }
                        }
                    } else { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Can not execute control.exe, Is runner downloaded properly?", None, None); };
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
                        if !rp.exists() { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Can not execute cmd.exe, Please start game at least once!", None, None); return; }
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
                                    if !status.success() { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Failed to execute cmd helper command! Please try again.", None, None); }
                                }
                                Ok(None) => {}
                                Err(_) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Failed to execute cmd helper command! Please try again or check the command correctness.", None, None); }
                            },
                            Err(_) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Failed to execute cmd helper command! Something serious is wrong.", None, None); }
                        }
                    } else { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Can not execute cmd.exe, Is runner downloaded properly?", None, None); };
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
                        if !rp.exists() { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Can not execute cmd.exe, Please start game at least once!", None, None); return; }
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
                                    if !status.success() { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Failed to execute winecfg helper command! Please try again.", None, None); }
                                }
                                Ok(None) => {}
                                Err(_) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Failed to execute winecfg helper command! Please try again or check the command correctness.", None, None); }
                            },
                            Err(_) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Failed to execute winecfg helper command! Something serious is wrong.", None, None); }
                        }
                    } else { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Can not execute winecfg.exe, Is runner downloaded properly?", None, None); };
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
                                if !status.success() { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Failed to dump steamrt3 diagnostics! Please try again.", None, None); }
                            }
                            Ok(None) => {}
                            Err(_) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Failed to execute steam-runtime-system-info command! Please try again or check the command correctness.", None, None); }
                        },
                        Err(e) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Failed to execute steam-runtime-system-info command! Something serious is wrong.", None, None); }
                    }
                    match app.opener().reveal_item_in_dir(log_path_file.as_path()) {
                        Ok(_) => {}
                        Err(_e) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "SteamRT3 log directory opening failed, try again later!", None, None); }
                    }
                } else { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Can not execute steam-runtime-system-info, Is steamrt3 downloaded properly?", None, None); };
            }
        }
        "steamrt4" => {
            #[cfg(target_os = "linux")]
            {
                let gs = get_settings(&app).unwrap();
                let fp = Path::new(&gs.default_runner_path).join("steamrt/steamrt4/");
                if fp.exists() {
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
                                if !status.success() { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Failed to dump steamrt3 diagnostics! Please try again.", None, None); }
                            }
                            Ok(None) => {}
                            Err(_) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Failed to execute steam-runtime-system-info command! Please try again or check the command correctness.", None, None); }
                        },
                        Err(e) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Failed to execute steam-runtime-system-info command! Something serious is wrong.", None, None); }
                    }
                    match app.opener().reveal_item_in_dir(log_path_file.as_path()) {
                        Ok(_) => {}
                        Err(_e) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "SteamRT4 log directory opening failed, try again later!", None, None); }
                    }
                } else { show_dialog_with_callback(&app, "error", "TwintailLauncher", "Can not execute steam-runtime-system-info, Is steamrt3 downloaded properly?", None, None); };
            }
        }
        _ => {}
    }
}

#[tauri::command]
pub fn open_uri(app: AppHandle, uri: String) {
    match app.opener().open_url(uri, None::<&str>) {
        Ok(_) => {}
        Err(_e) => {}
    }
}
