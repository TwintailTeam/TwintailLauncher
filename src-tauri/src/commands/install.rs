use crate::utils::db_manager::{create_installation, delete_installation_by_id, get_install_info_by_id, get_installs, get_installs_by_manifest_id, get_manifest_info_by_filename, get_manifest_info_by_id, get_settings, update_install_disable_system_idle_by_id, update_install_env_vars_by_id, update_install_fps_value_by_id, update_install_game_background_by_id, update_install_game_location_by_id, update_install_graphics_api_by_id, update_install_ignore_updates_by_id, update_install_launch_args_by_id, update_install_launch_cmd_by_id, update_install_mangohud_config_location_by_id, update_install_pre_launch_cmd_by_id, update_install_prefix_location_by_id, update_install_shortcut_location_by_id, update_install_show_drpc_by_id, update_install_skip_hash_check_by_id, update_install_use_fps_unlock_by_id, update_install_use_gamemode_by_id, update_install_use_jadeite_by_id, update_install_use_mangohud_by_id, update_install_use_xxmi_by_id, update_install_xxmi_config_by_id, update_installs_order};
use crate::utils::game_launch_manager::launch;
use crate::utils::repo_manager::get_manifest;
use crate::utils::shortcuts::remove_desktop_shortcut;
use crate::utils::{AddInstallRsp, DownloadSizesRsp, ResumeStatesRsp, apply_xxmi_tweaks, copy_dir_all, generate_cuid, get_mi_path_from_game, models::GameVersion, show_dialog, extract_authkey_from_content};
use fischl::utils::free_space::get_disk_space;
use fischl::utils::is_process_running;
use fischl::utils::prettify_bytes;
use std::collections::HashMap;
use std::fs;
use std::ops::Add;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use sqlx::types::Json;
use std::sync::atomic::Ordering;
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::DownloadState;
use crate::downloading::ExtrasDownloadPayload;
use crate::downloading::queue::QueueJobKind;
use crate::downloading::QueueJobPayload;
#[cfg(target_os = "linux")]
use crate::downloading::RunnerDownloadPayload;
#[cfg(target_os = "linux")]
use crate::utils::db_manager::{create_installed_runner, get_installed_runner_info_by_version, update_install_shortcut_is_steam_by_id, update_installed_runner_is_installed_by_version};
use crate::utils::models::XXMISettings;
#[cfg(target_os = "linux")]
use crate::utils::repo_manager::get_compatibility;
#[cfg(target_os = "linux")]
use crate::utils::{is_flatpak, run_async_command, runner_from_runner_version, shortcuts::{add_desktop_shortcut, add_steam_shortcut, remove_steam_shortcut}};
#[cfg(target_os = "linux")]
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(target_os = "linux")]
use steam_shortcuts_util::{Shortcut, app_id_generator::calculate_app_id};

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
pub fn set_installs_order(app: AppHandle, order: Vec<(String, i32)>) {
    update_installs_order(&app, order);
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

#[allow(unused_mut, unused_variables)]
#[tauri::command]
pub fn add_install(app: AppHandle, manifest_id: String, version: String, audio_lang: String, name: String, mut directory: String, mut runner_path: String, mut dxvk_path: String, runner_version: String, dxvk_version: String, game_icon: String, game_background: String, mut ignore_updates: bool, skip_hash_check: bool, mut use_jadeite: bool, use_xxmi: bool, use_fps_unlock: bool, env_vars: String, pre_launch_command: String, launch_command: String, fps_value: String, mut runner_prefix: String, launch_args: String, skip_game_dl: bool, region_code: String) -> Option<AddInstallRsp> {
    if manifest_id.is_empty() || version.is_empty() || name.is_empty() || directory.is_empty() || runner_path.is_empty() || dxvk_path.is_empty() || game_icon.is_empty() || game_background.is_empty() {
        None
    } else {
        let gs = get_settings(&app).unwrap();
        let cuid = generate_cuid();
        let m = manifest_id + ".json";
        let dbm = get_manifest_info_by_filename(&app, m.clone()).unwrap();
        let gm = get_manifest(&app, m.clone()).unwrap();
        let g = gm.game_versions.iter().find(|e| e.metadata.version == version).unwrap();

        // Prevent duplicate: check if any existing install for this manifest has an active/queued download job
        if !skip_game_dl {
            if let Some(existing_installs) = get_installs_by_manifest_id(&app, dbm.id.clone()) {
                let state = app.state::<DownloadState>();
                let q = state.queue.lock().unwrap().clone();
                if let Some(ref queue) = q {
                    for ei in &existing_installs {
                        if ei.version == version && queue.has_job_for_id(ei.id.clone()) {
                            show_dialog(&app, "warning", "TwintailLauncher", format!("{in} is already queued for download!", in = ei.name.clone()).as_str(), None);
                            return Some(AddInstallRsp { success: false, install_id: "".to_string(), background: "".to_string() });
                        }
                    }
                }
            }
        }

        let mut steam_import = false;
        let mut install_location = if skip_game_dl { Path::new(directory.as_str()).to_path_buf() } else { Path::new(directory.as_str()).join(cuid.clone()) };
        directory = install_location.to_str().unwrap().to_string();

        if gm.extra.steam_import_config.enabled && skip_game_dl {
            if !gm.extra.steam_import_config.steam_api_dll.is_empty() {
                let steamdll = install_location.join(gm.extra.steam_import_config.steam_api_dll);
                if steamdll.exists() { ignore_updates = true; steam_import = true; }
            }
            if !gm.extra.steam_import_config.steam_appid_txt.is_empty() {
                let steamappid = install_location.join(gm.extra.steam_import_config.steam_appid_txt);
                if steamappid.exists() { ignore_updates = true; steam_import = true; }
            }
        }

        #[cfg(target_os = "windows")]
        {
            dxvk_path = "".to_string();
            runner_path = "".to_string();
        }

        #[cfg(target_os = "linux")]
        {
            let wine = Path::new(gs.default_runner_path.as_str());
            let dxvk = Path::new(gs.default_dxvk_path.as_str());
            let prefix_loc = Path::new(&runner_prefix).join(cuid.clone());
            runner_prefix = prefix_loc.to_str().unwrap().to_string();

            // Remove prefix just in case
            if prefix_loc.exists() { fs::remove_dir_all(runner_prefix.clone()).unwrap(); }

            runner_path = wine.join(runner_version.clone()).to_str().unwrap().to_string();
            dxvk_path = dxvk.join(dxvk_version.clone()).to_str().unwrap().to_string();

            if !Path::exists(runner_path.as_ref()) { fs::create_dir_all(runner_path.clone()).unwrap(); }
            //if !Path::exists(dxvk_path.as_ref()) { fs::create_dir_all(dxvk_path.clone()).unwrap(); }
            if !prefix_loc.exists() { fs::create_dir_all(runner_prefix.clone()).unwrap(); }

            let archandle = Arc::new(app.clone());
            let mut runv = Arc::new(runner_version.clone());
            let mut runpp = Arc::new(runner_path.clone());
            let rpp = Arc::new(runner_prefix.clone());
            //let dxvkpp = Arc::new(dxvk_path.clone());
            //let dxvkv = Arc::new(dxvk_version.clone());

            // Apply compatibility overrides
            let co = gm.extra.compat_overrides;
            if co.install_to_prefix {
                install_location = prefix_loc.clone().join("pfx").join("drive_c").join("Program Files").join(cuid.clone());
                if !install_location.exists() { fs::create_dir_all(&install_location).unwrap(); }
                directory = install_location.to_str().unwrap().to_string();
            }
            if co.override_runner.linux.enabled {
                runner_path = wine.join(co.override_runner.linux.runner_version.clone()).to_str().unwrap().to_string();
                runpp = Arc::new(runner_path.clone());
                runv = Arc::new(co.override_runner.linux.runner_version.clone());
            }

            // Download runner via queue system (shows in downloads UI)
            let rm = get_compatibility(&app, &runner_from_runner_version(runv.as_str().to_string()).unwrap());
            if let Some(rm) = rm {
                let rv = rm.versions.into_iter().filter(|v| v.version.as_str() == runv.as_str()).collect::<Vec<_>>();
                if let Some(runnerp) = rv.get(0) {
                    let rp = Path::new(runpp.as_str()).to_path_buf();

                    // Only download if directory is empty
                    if fs::read_dir(rp.as_path()).map(|mut d| d.next().is_none()).unwrap_or(true) {
                        // Determine the download URL based on architecture
                        let mut dl_url = runnerp.url.clone();
                        if let Some(ref urls) = runnerp.urls {
                            #[cfg(target_arch = "x86_64")]
                            { dl_url = urls.x86_64.clone(); }
                            #[cfg(target_arch = "aarch64")]
                            { dl_url = if urls.aarch64.is_empty() { runnerp.url.clone() } else { urls.aarch64.clone() }; }
                        }

                        // Create runner directory if needed
                        if !rp.exists() { let _ = fs::create_dir_all(&rp); }

                        // Create/update database entry (will be marked as installed by download job on completion)
                        let ir = get_installed_runner_info_by_version(&app, runv.to_string());
                        if ir.is_some() { update_installed_runner_is_installed_by_version(&app, runv.to_string(), false); } else { let _ = create_installed_runner(&app, runv.to_string(), false, rp.to_str().unwrap().to_string()); }

                        // Enqueue the download job via the queue system
                        let state = app.state::<DownloadState>();
                        let q = state.queue.lock().unwrap().clone();
                        if let Some(queue) = q {
                            queue.enqueue(QueueJobKind::RunnerDownload, QueueJobPayload::Runner(RunnerDownloadPayload {
                                    runner_version: runv.to_string(),
                                    runner_url: dl_url,
                                    runner_path: rp.to_str().unwrap().to_string(),
                            }));
                        }
                    }
                }
            }
            // Patch wuwa if existing install
            if gm.biz == "wuwa_global" && skip_game_dl { crate::utils::apply_patch(&app, Path::new(&directory.clone()).to_str().unwrap().to_string(), "aki".to_string(), "add".to_string()); }
        }
        #[cfg(target_os = "linux")]
        let gbg = g.assets.game_background.clone();
        #[cfg(not(target_os = "linux"))]
        let gbg = if let Some(ref lbg) = g.assets.game_live_background { if !lbg.is_empty() { lbg.clone() } else { g.assets.game_background.clone() } } else { g.assets.game_background.clone() };
        if !install_location.exists() {
            if let Err(e) = fs::create_dir_all(&install_location) {
                show_dialog(&app, "error", "TwintailLauncher", &format!("Failed to start installation! {}", e), None);
                return Some(AddInstallRsp {
                    success: false,
                    install_id: "".to_string(),
                    background: "".to_string(),
                });
            }
        }
        if !skip_game_dl {
            let downloading_marker = install_location.join("downloading");
            if !downloading_marker.exists() { let _ = fs::create_dir(&downloading_marker); }
        }
        let default_graphics_api = gm.extra.graphics_api_options.default.clone();
        create_installation(&app, cuid.clone(), dbm.id, version, audio_lang, g.metadata.versioned_name.clone(), directory, runner_path, dxvk_path, runner_version, dxvk_version, g.assets.game_icon.clone(), gbg.clone(), ignore_updates, skip_hash_check, use_jadeite, use_xxmi, use_fps_unlock, env_vars, pre_launch_command, launch_command, fps_value, runner_prefix, launch_args, false, false, gs.default_mangohud_config_path.clone(), region_code, steam_import, default_graphics_api).unwrap();
        Some(AddInstallRsp {
            success: true,
            install_id: cuid.clone(),
            background: gbg,
        })
    }
}

#[tauri::command]
pub async fn remove_install(app: AppHandle, id: String, wipe_prefix: bool, keep_game_data: bool) -> Option<bool> {
    if id.is_empty() {
        None
    } else {
        // Cancel any active or queued downloads for this installation first
        cancel_download_for_install(&app, &id);

        let install = get_install_info_by_id(&app, id.clone());
        if install.is_some() {
            let i = install.unwrap();
            let lm = get_manifest_info_by_id(&app, i.manifest_id.clone()).unwrap();
            let gm = get_manifest(&app, lm.filename.clone()).unwrap();

            let installdir = i.directory;
            let prefixdir = i.runner_prefix;
            let idp = Path::new(&installdir);
            let pdp = Path::new(&prefixdir);
            let gexe = idp.join(gm.paths.exe_filename.clone());

            if wipe_prefix {
                if pdp.exists() { fs::remove_dir_all(prefixdir.clone()).unwrap(); }
            }
            if !keep_game_data && !i.steam_imported {
                if idp.exists() && gexe.exists() {
                    let r = fs::remove_dir_all(installdir.clone());
                    match r {
                        Ok(_) => {},
                        Err(e) => { show_dialog(&app, "error", "TwintailLauncher", format!("Failed to remove game installation directory. {} - Please remove the folder manually!", e.to_string()).as_str(), None) }
                    }
                } else { show_dialog(&app, "error", "TwintailLauncher", "Failed to remove game installation directory. Please remove the folder manually!", None); }
            }
            delete_installation_by_id(&app, id.clone()).unwrap();
            Some(true)
        } else {
            None
        }
    }
}

#[tauri::command]
pub fn update_install_game_path(app: AppHandle, id: String, path: String) -> Option<bool> {
    let install = get_install_info_by_id(&app, id);

    if install.is_some() {
        let m = install.unwrap();
        let np = path.clone();
        let app1 = app.clone();
        let oldpath = Arc::new(m.directory);
        let installation_id = m.id.clone();
        let install_name = m.name.clone();

        if !Path::exists(path.as_ref()) { fs::create_dir_all(path.clone()).unwrap(); }

        // Initialize move only IF old path has files AND new path is empty directory
        if Path::exists(oldpath.as_ref().to_string().as_ref()) {
            if fs::read_dir(oldpath.as_ref()).unwrap().next().is_some() && fs::read_dir(&path).unwrap().next().is_none() {
                let op = oldpath.clone();
                std::thread::spawn(move || {
                    let ap = Path::new(op.as_ref()).to_path_buf();
                    copy_dir_all(&app1, ap, &path.clone(), installation_id, install_name.clone(), "Game".to_string()).unwrap();

                    let mut payload = HashMap::new();
                    payload.insert("install_name", install_name.clone());
                    payload.insert("install_type", "Game".to_string());
                    payload.insert("progress", "0".to_string());
                    payload.insert("total", "1000".to_string());
                    app1.emit("move_complete", &payload).unwrap();
                });
            }
        }
        update_install_game_location_by_id(&app, m.id, np);
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_install_runner_path(app: AppHandle, id: String, path: String) -> Option<bool> {
    let install = get_install_info_by_id(&app, id);

    if install.is_some() {
        let m = install.unwrap();
        let np = path.clone();
        let app1 = app.clone();
        let oldpath = Arc::new(m.runner_path);
        let installation_id = m.id.clone();
        let install_name = m.name.clone();

        if !Path::exists(path.as_ref()) { fs::create_dir_all(path.clone()).unwrap(); }

        if Path::exists(oldpath.as_ref().to_string().as_ref()) {
            if fs::read_dir(oldpath.as_ref()).unwrap().next().is_some() && fs::read_dir(&path).unwrap().next().is_none() {
                let op = oldpath.clone();
                std::thread::spawn(move || {
                    let ap = Path::new(op.as_ref()).to_path_buf();
                    copy_dir_all(&app1, ap, &path.clone(), installation_id, install_name.clone(), "Runner".to_string()).unwrap();

                    let mut payload = HashMap::new();
                    payload.insert("install_name", install_name.clone());
                    payload.insert("install_type", "Runner".to_string());
                    payload.insert("progress", "0".to_string());
                    payload.insert("total", "1000".to_string());
                    app1.emit("move_complete", &payload).unwrap();
                });
            }
        }
        crate::utils::db_manager::update_install_runner_location_by_id(&app, m.id, np);
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_install_dxvk_path(app: AppHandle, id: String, path: String) -> Option<bool> {
    let install = get_install_info_by_id(&app, id);

    if install.is_some() {
        let m = install.unwrap();
        let np = path.clone();
        let app1 = app.clone();
        let oldpath = Arc::new(m.dxvk_path);
        let installation_id = m.id.clone();
        let install_name = m.name.clone();

        if !Path::exists(path.as_ref()) {
            fs::create_dir_all(path.clone()).unwrap();
        }

        if Path::exists(oldpath.as_ref().to_string().as_ref()) {
            if fs::read_dir(oldpath.as_ref()).unwrap().next().is_some() && fs::read_dir(&path).unwrap().next().is_none() {
                let op = oldpath.clone();
                std::thread::spawn(move || {
                    let ap = Path::new(op.as_ref()).to_path_buf();
                    copy_dir_all(&app1, ap, &path.clone(), installation_id, install_name.clone(), "DXVK".to_string()).unwrap();

                    let mut payload = HashMap::new();
                    payload.insert("install_name", install_name.clone());
                    payload.insert("install_type", "DXVK".to_string());
                    payload.insert("progress", "0".to_string());
                    payload.insert("total", "1000".to_string());
                    app1.emit("move_complete", &payload).unwrap();
                });
            }
        }
        crate::utils::db_manager::update_install_dxvk_location_by_id(&app, m.id, np);
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_install_skip_version_updates(app: AppHandle, id: String, enabled: bool) -> Option<bool> {
    let manifest = get_install_info_by_id(&app, id);

    if manifest.is_some() {
        let m = manifest.unwrap();
        update_install_ignore_updates_by_id(&app, m.id, enabled);
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_install_skip_hash_valid(app: AppHandle, id: String, enabled: bool) -> Option<bool> {
    let manifest = get_install_info_by_id(&app, id);

    if manifest.is_some() {
        let m = manifest.unwrap();
        update_install_skip_hash_check_by_id(&app, m.id, enabled);
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_install_use_jadeite(app: AppHandle, id: String, enabled: bool) -> Option<bool> {
    let manifest = get_install_info_by_id(&app, id);
    let settings = get_settings(&app).unwrap();

    if manifest.is_some() {
        let m = manifest.unwrap();
        let p = Path::new(&settings.jadeite_path).to_path_buf();
        update_install_use_jadeite_by_id(&app, m.id, enabled);
        if enabled { enqueue_extras_download(&app, p.to_str().unwrap().to_string(), "jadeite".to_string(), "v5.0.1-hotfix".to_string(), false); }
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_install_use_xxmi(app: AppHandle, id: String, enabled: bool) -> Option<bool> {
    let manifest = get_install_info_by_id(&app, id);
    let settings = get_settings(&app).unwrap();

    if manifest.is_some() {
        let m = manifest.unwrap();
        let p = Path::new(&settings.xxmi_path).to_path_buf();
        let ps = p.to_str().unwrap().to_string();
        update_install_use_xxmi_by_id(&app, m.id.clone(), enabled);
        if enabled {
            for pkg_type in ["xxmi", "gimi", "srmi", "zzmi", "himi", "wwmi"/*, "efmi"*/] { enqueue_extras_download(&app, ps.clone(), "xxmi".to_string(), pkg_type.to_string(), false); }
        }
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_install_use_fps_unlock(app: AppHandle, id: String, enabled: bool) -> Option<bool> {
    let manifest = get_install_info_by_id(&app, id);
    let settings = get_settings(&app).unwrap();

    if manifest.is_some() {
        let m = manifest.unwrap();
        let p = Path::new(&settings.fps_unlock_path).to_path_buf();
        update_install_use_fps_unlock_by_id(&app, m.id, enabled);
        if enabled { enqueue_extras_download(&app, p.to_str().unwrap().to_string(), "keqingunlock".to_string(), "keqing_unlock".to_string(), false); }
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_install_fps_value(app: AppHandle, id: String, fps: String) -> Option<bool> {
    let install = get_install_info_by_id(&app, id);
    let settings = get_settings(&app).unwrap();

    if install.is_some() {
        let m = install.unwrap();
        let p = Path::new(&settings.fps_unlock_path).to_path_buf();
        update_install_fps_value_by_id(&app, m.id, fps);
        if m.use_fps_unlock { enqueue_extras_download(&app, p.to_str().unwrap().to_string(), "keqingunlock".to_string(), "keqing_unlock".to_string(), false); }
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_install_graphics_api(app: AppHandle, id: String, api: String) -> Option<bool> {
    let install = get_install_info_by_id(&app, id);
    if install.is_some() { let m = install.unwrap(); update_install_graphics_api_by_id(&app, m.id, api); Some(true) } else { None }
}

#[tauri::command]
pub fn update_install_use_gamemode(app: AppHandle, id: String, enabled: bool) -> Option<bool> {
    let manifest = get_install_info_by_id(&app, id);

    if manifest.is_some() {
        let m = manifest.unwrap();
        update_install_use_gamemode_by_id(&app, m.id, enabled);
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_install_use_mangohud(app: AppHandle, id: String, enabled: bool) -> Option<bool> {
    let manifest = get_install_info_by_id(&app, id);

    if manifest.is_some() {
        let m = manifest.unwrap();
        update_install_use_mangohud_by_id(&app, m.id, enabled);
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_install_mangohud_config_path(app: AppHandle, id: String, path: String) -> Option<bool> {
    let install = get_install_info_by_id(&app, id);

    if install.is_some() {
        let m = install.unwrap();
        let np = path.clone();
        update_install_mangohud_config_location_by_id(&app, m.id, np);
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_install_xxmi_config(app: AppHandle, id: String, xxmi_hunting: Option<u64>, xxmi_sd: Option<bool>, xxmi_sw: Option<bool>, _engineini_tweaks: Option<bool>) -> Option<bool> {
    let install = get_install_info_by_id(&app, id);

    if install.is_some() {
        let m = install.unwrap();
        let gs = get_settings(&app).unwrap();
        let mut data = Json(XXMISettings {
            hunting_mode: m.xxmi_config.hunting_mode,
            require_admin: m.xxmi_config.require_admin,
            dll_init_delay: m.xxmi_config.dll_init_delay,
            close_delay: m.xxmi_config.close_delay,
            show_warnings: m.xxmi_config.show_warnings,
            dump_shaders: m.xxmi_config.dump_shaders,
        });
        if xxmi_hunting.is_some() { data.hunting_mode = xxmi_hunting?; }
        if xxmi_sd.is_some() { data.dump_shaders = xxmi_sd?; }
        if xxmi_sw.is_some() { data.show_warnings = if xxmi_sw? { 1 } else { 0 } }

        if let Some(x) = get_manifest_info_by_id(&app, m.manifest_id) {
            if let Some(g) = get_manifest(&app, x.filename) {
                let exe = g.paths.exe_filename.clone().split('/').last().unwrap().to_string();
                let mi = get_mi_path_from_game(exe).unwrap();
                let package = Path::new(&gs.xxmi_path).join(mi);
                data = apply_xxmi_tweaks(package, data.clone());
            }
        }
        update_install_xxmi_config_by_id(&app, m.id, data);
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_install_show_drpc(app: AppHandle, id: String, enabled: bool) -> Option<bool> {
    let manifest = get_install_info_by_id(&app, id);

    if manifest.is_some() {
        let m = manifest.unwrap();
        update_install_show_drpc_by_id(&app, m.id, enabled);
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_install_disable_system_idle(app: AppHandle, id: String, enabled: bool) -> Option<bool> {
    let manifest = get_install_info_by_id(&app, id);

    if manifest.is_some() {
        let m = manifest.unwrap();
        update_install_disable_system_idle_by_id(&app, m.id, enabled);
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_install_env_vars(app: AppHandle, id: String, env_vars: String) -> Option<bool> {
    let install = get_install_info_by_id(&app, id);

    if install.is_some() {
        let m = install.unwrap();
        update_install_env_vars_by_id(&app, m.id, env_vars);
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_install_pre_launch_cmd(app: AppHandle, id: String, cmd: String) -> Option<bool> {
    let install = get_install_info_by_id(&app, id);

    if install.is_some() {
        let m = install.unwrap();
        update_install_pre_launch_cmd_by_id(&app, m.id, cmd);
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_install_launch_cmd(app: AppHandle, id: String, cmd: String) -> Option<bool> {
    let install = get_install_info_by_id(&app, id);

    if install.is_some() {
        let m = install.unwrap();
        update_install_launch_cmd_by_id(&app, m.id, cmd);
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_install_game_background(app: AppHandle, id: String, background: String) -> Option<bool> {
    let install = get_install_info_by_id(&app, id);
    if install.is_some() {
        let m = install.unwrap();
        update_install_game_background_by_id(&app, m.id, background);
        Some(true)
    } else { None }
}

#[tauri::command]
pub fn update_install_prefix_path(app: AppHandle, id: String, path: String) -> Option<bool> {
    let install = get_install_info_by_id(&app, id);

    if install.is_some() {
        let m = install.unwrap();
        let np = path.clone();
        let app1 = app.clone();
        let oldpath = Arc::new(m.runner_prefix.clone());
        let installation_id = m.id.clone();
        let install_name = m.name.clone();

        if !Path::exists(path.as_ref()) { fs::create_dir_all(path.clone()).unwrap(); }

        if Path::exists(oldpath.as_ref().to_string().as_ref()) {
            if fs::read_dir(oldpath.as_ref()).unwrap().next().is_some() && fs::read_dir(&path).unwrap().next().is_none() {
                let op = oldpath.clone();
                std::thread::spawn(move || {
                    let ap = Path::new(op.as_ref());
                    copy_dir_all(&app1, ap, &path.clone(), installation_id, install_name.clone(), "Prefix".to_string()).unwrap();

                    let mut payload = HashMap::new();
                    payload.insert("install_name", install_name.clone());
                    payload.insert("install_type", "Prefix".to_string());
                    payload.insert("progress", "0".to_string());
                    payload.insert("total", "1000".to_string());
                    app1.emit("move_complete", &payload).unwrap();
                });
            }
        }
        update_install_prefix_location_by_id(&app, m.id, np);
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_install_launch_args(app: AppHandle, id: String, args: String) -> Option<bool> {
    let install = get_install_info_by_id(&app, id);

    if install.is_some() {
        let m = install.unwrap();
        update_install_launch_args_by_id(&app, m.id, args);
        Some(true)
    } else {
        None
    }
}

#[cfg(target_os = "linux")]
#[tauri::command]
pub fn update_install_runner_version(app: AppHandle, id: String, version: String) -> Option<bool> {
    let install = get_install_info_by_id(&app, id);

    if install.is_some() {
        let m = install.unwrap();
        let rp = m.runner_path.clone();
        let rpn = rp.replace(m.runner_version.as_str(), version.as_str());
        if !Path::exists(rpn.as_ref()) { fs::create_dir_all(rpn.clone()).unwrap(); }

        if fs::read_dir(rpn.as_str()).unwrap().next().is_none() {
            // Download runner via queue system (shows in downloads UI)
            let rm = get_compatibility(&app, &runner_from_runner_version(version.clone()).unwrap());
            if let Some(rm) = rm {
                let rv = rm.versions.into_iter().filter(|v| v.version.as_str() == version.as_str()).collect::<Vec<_>>();
                if let Some(runnerp) = rv.get(0) {
                    let rp = Path::new(rpn.as_str()).to_path_buf();

                    // Determine the download URL based on architecture
                    let mut dl_url = runnerp.url.clone();
                    if let Some(ref urls) = runnerp.urls {
                        #[cfg(target_arch = "x86_64")]
                        { dl_url = urls.x86_64.clone(); }
                        #[cfg(target_arch = "aarch64")]
                        { dl_url = if urls.aarch64.is_empty() { runnerp.url.clone() } else { urls.aarch64.clone() }; }
                    }

                    // Create/update database entry (will be marked as installed by download job on completion)
                    let ir = get_installed_runner_info_by_version(&app, version.clone());
                    if ir.is_some() { update_installed_runner_is_installed_by_version(&app, version.clone(), false); } else { let _ = create_installed_runner(&app, version.clone(), false, rp.to_str().unwrap().to_string()); }

                    // Enqueue the download job via the queue system
                    let state = app.state::<DownloadState>();
                    let q = state.queue.lock().unwrap().clone();
                    if let Some(queue) = q {
                        queue.enqueue(QueueJobKind::RunnerDownload, QueueJobPayload::Runner(RunnerDownloadPayload {
                                runner_version: version.clone(),
                                runner_url: dl_url,
                                runner_path: rp.to_str().unwrap().to_string(),
                        }));
                    }
                }
            }
        } else {}
        crate::utils::db_manager::update_install_runner_version_by_id(&app, m.id.clone(), version);
        crate::utils::db_manager::update_install_runner_location_by_id(&app, m.id, rpn);
        Some(true)
    } else {
        None
    }
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub fn update_install_runner_version(_app: AppHandle, _id: String, _version: String) -> Option<bool> {
    None
}

#[cfg(target_os = "linux")]
#[tauri::command]
pub fn update_install_dxvk_version(app: AppHandle, id: String, version: String) -> Option<bool> {
    let install = get_install_info_by_id(&app, id);

    if install.is_some() {
        let m = install.unwrap();
        let p = m.dxvk_path.clone();
        let pn = p.replace(m.dxvk_version.as_str(), version.as_str());
        if !Path::exists(pn.as_ref()) { fs::create_dir_all(pn.clone()).unwrap(); }

        let archandle = Arc::new(app.clone());
        let dxvkv = Arc::new(version.clone());
        let dxpp = Arc::new(pn.clone());
        //let rpp = Arc::new(m.runner_prefix.clone());
        let runv = Arc::new(m.runner_version.clone());
        //let runp = Arc::new(m.runner_path.clone());

        if fs::read_dir(pn.as_str()).unwrap().next().is_none() {
            std::thread::spawn(move || {
                let rm = get_compatibility(archandle.as_ref(), &runner_from_runner_version(runv.as_str().to_string()).unwrap()).unwrap();
                //let dm = get_compatibility(archandle.as_ref(), &runner_from_runner_version(dxvkv.as_str().to_string()).unwrap()).unwrap();
                //let dv = dm.versions.into_iter().filter(|v| v.version.as_str() == dxvkv.as_str()).collect::<Vec<_>>();
                //let dxp = dv.get(0).unwrap().to_owned();
                let dxpp = Path::new(dxpp.as_str()).to_path_buf();
                //let rp = Path::new(runp.as_str()).to_path_buf();

                let mut dlpayload = HashMap::new();
                let is_proton = rm.display_name.to_ascii_lowercase().contains("proton") && !rm.display_name.to_ascii_lowercase().contains("wine");

                if is_proton {} else {
                    dlpayload.insert("name", runv.to_string());
                    dlpayload.insert("progress", "0".to_string());
                    dlpayload.insert("total", "1000".to_string());
                    archandle.emit("download_progress", dlpayload.clone()).unwrap();

                    /*let mut dl_url = dxp.url.clone(); // Always x86_64
                    if let Some(urls) = dxp.urls {
                        #[cfg(target_arch = "x86_64")]
                        { dl_url = urls.x86_64; }
                        #[cfg(target_arch = "aarch64")]
                        { dl_url = if urls.aarch64.is_empty() { dxp.url.clone() } else { urls.aarch64 }; }
                    }*/

                    let r0 = run_async_command(async { true });
                    if r0 {
                        archandle.emit("download_complete", ()).unwrap();
                    } else {
                        show_dialog(&*archandle, "error", "TwintailLauncher", format!("Error occurred while trying to download {dxvn} DXVK! Please retry later.", dxvn = dxvkv.as_str().to_string()).as_str(), Some(vec!["Ok"]));
                        archandle.emit("download_complete", ()).unwrap();
                        if dxpp.exists() { fs::remove_dir_all(&dxpp).unwrap(); }
                    }
                }
            });
        } else {}
        crate::utils::db_manager::update_install_dxvk_version_by_id(&app, m.id.clone(), version);
        crate::utils::db_manager::update_install_dxvk_location_by_id(&app, m.id, pn);
        Some(true)
    } else {
        None
    }
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub fn update_install_dxvk_version(_app: AppHandle, _id: String, _version: String) -> Option<bool> {
    None
}

#[tauri::command]
pub fn game_launch(app: AppHandle, id: String) -> Option<bool> {
    let install = get_install_info_by_id(&app, id);
    let global_settings = get_settings(&app).unwrap();

    if install.is_some() {
        let m = install.unwrap();
        let gmm = get_manifest_info_by_id(&app, m.clone().manifest_id).unwrap();
        let gm = get_manifest(&app, gmm.filename).unwrap();

        let appc = app.clone();
        std::thread::spawn(move || { let app = appc.clone(); launch(&app, m.clone(), gm, global_settings).unwrap() });
        Some(true)
    } else {
        show_dialog(&app, "error", "TwintailLauncher", "Failed to find game installation!", None);
        None
    }
}

#[tauri::command]
pub fn check_game_running(app: AppHandle, id: String) -> Option<String> {
    let install = get_install_info_by_id(&app, id.clone());

    if let Some(m) = install {
        let gmm = get_manifest_info_by_id(&app, m.manifest_id);
        if let Some(manifest_info) = gmm {
            let gm = get_manifest(&app, manifest_info.filename);
            if let Some(manifest) = gm {
                if is_process_running("winetricks") || is_process_running("winetr") { return Some("preparing".to_string()); }
                let exe_name = manifest.paths.exe_filename.split('/').last().unwrap_or("");
                let exe_stem = exe_name.split('.').next().unwrap_or(exe_name);
                let exe_check = if exe_stem.len() > 15 { &exe_stem[..15] } else { exe_name };
                if is_process_running(exe_check) { return Some("running".to_string()); }
                return Some("idle".to_string());
            }
        }
    }
    None
}

#[tauri::command]
pub fn get_download_sizes(app: AppHandle, biz: String, version: String, lang: String, path: String, region: Option<String>) -> Option<String> {
    let manifest = get_manifest(&app, biz + ".json");

    if manifest.is_some() {
        let m = manifest.unwrap();

        let entry = m.game_versions.into_iter().filter(|e| e.metadata.version == version).collect::<Vec<GameVersion>>();
        let g = entry.get(0).unwrap();
        let gs = if m.biz == "bh3_global" { g.game.full.iter().filter(|e| e.region_code.clone() == region.clone().unwrap_or("glb_official".to_string())).into_iter().map(|x| x.decompressed_size.parse::<u64>().unwrap()).sum::<u64>() } else { g.game.full.iter().map(|x| x.decompressed_size.parse::<u64>().unwrap()).sum::<u64>() };
        let mut fss = gs;
        if !g.audio.full.is_empty() {
            let audios: Vec<_> = g.audio.full.iter().filter(|x| x.language == lang).collect();
            let audio = audios.get(0).unwrap().decompressed_size.parse::<u64>().unwrap();
            fss = gs.add(audio);
        }

        let p = PathBuf::from(&path);
        let (a, t) = get_disk_space(p);
        let stringified = serde_json::to_string(&DownloadSizesRsp {
            game_decompressed_size: prettify_bytes(fss),
            free_disk_space: prettify_bytes(a),
            total_disk_space: prettify_bytes(t),
            game_decompressed_size_raw: fss,
            free_disk_space_raw: a,
            total_disk_space_raw: t,
        }).unwrap();
        Some(stringified)
    } else {
        None
    }
}

#[tauri::command]
pub fn get_resume_states(app: AppHandle, install: String) -> Option<String> {
    let install = get_install_info_by_id(&app, install);

    if install.is_some() {
        let i = install.unwrap();

        let ip = Path::new(&i.directory);
        let dp = ip.join("downloading");
        let up = ip.join("patching");
        let pup = ip.join("patching").join(".preload");
        let rep = ip.join("repairing");

        let frsp: ResumeStatesRsp;
        if dp.exists() && !rep.exists() && !up.exists() && !pup.exists() {
            frsp = ResumeStatesRsp {
                downloading: true,
                updating: false,
                preloading: false,
                repairing: false,
            };
        } else if up.exists() && !rep.exists() && !pup.exists() && !dp.exists() {
            frsp = ResumeStatesRsp {
                downloading: false,
                updating: true,
                preloading: false,
                repairing: false,
            };
        } else if pup.exists() && !dp.exists() && !up.exists() && !rep.exists() {
            frsp = ResumeStatesRsp {
                downloading: false,
                updating: false,
                preloading: true,
                repairing: false,
            };
        } else if rep.exists() && !dp.exists() && !up.exists() && !pup.exists() {
            frsp = ResumeStatesRsp {
                downloading: false,
                updating: false,
                preloading: false,
                repairing: true,
            };
        } else {
            frsp = ResumeStatesRsp {
                downloading: false,
                updating: false,
                preloading: false,
                repairing: false,
            };
        }
        let stringified = serde_json::to_string(&frsp).unwrap();
        Some(stringified)
    } else {
        None
    }
}

#[tauri::command]
pub fn add_shortcut(app: AppHandle, install_id: String, shortcut_type: String) {
    let install = get_install_info_by_id(&app, install_id).unwrap();
    #[cfg(target_os = "linux")]
    {
        match shortcut_type.as_str() {
            "desktop" => {
                let base = app.path().home_dir().unwrap().join(".local/share/applications");
                let file = base.join(format!("{}.desktop", install.name.as_str()));
                let bin_name = if is_flatpak() { "flatpak run app.twintaillauncher.ttl" } else { "twintaillauncher" };
                let icon = if is_flatpak() { "app.twintaillauncher.ttl" } else { "twintaillauncher" };

                let content = format!(
                    r#"[Desktop Entry]
Categories=Game;
Comment=Launch this game using TwintailLauncher
Exec={} --install={}
Icon={}
Name={}
Terminal=false
Type=Application
"#,
                    bin_name,
                    install.id.as_str(),
                    icon,
                    install.name.as_str()
                );

                let status = add_desktop_shortcut(file.clone(), content);
                if status {
                    update_install_shortcut_location_by_id(&app, install.id.clone(), file.clone().to_str().unwrap().to_string(), );
                    show_dialog(&app, "info", "TwintailLauncher", format!("Successfully created {} desktop shortcut.", install.name.as_str()).as_str(), None);
                } else { show_dialog(&app, "error", "TwintailLauncher", format!("Failed to create {} desktop shortcut! If you use flatpak please make sure we have permission to access ~/.local/share/applications", install.name.as_str()).as_str(), None); }
            }
            "steam" => {
                let flatpak_steam = app.path().home_dir().unwrap().join(".var/app/com.valvesoftware.Steam/data/Steam/userdata");
                let normal_steam = crate::utils::shortcuts::resolve_normal_steam_userdata(app.path().home_dir().unwrap());

                let manifest = get_manifest_info_by_id(&app, install.manifest_id).unwrap();
                let m = get_manifest(&app, manifest.filename).unwrap();
                let launchargs = format!("--install={}", install.id.as_str());

                let shortcut = Shortcut {
                    order: "",
                    app_id: calculate_app_id(m.paths.exe_filename.as_str(), install.name.as_str()),
                    app_name: install.name.as_str(),
                    exe: if is_flatpak() { "flatpak run app.twintaillauncher.ttl" } else { "twintaillauncher" },
                    start_dir: install.directory.as_str(),
                    icon: install.game_icon.as_str(),
                    shortcut_path: "",
                    launch_options: launchargs.as_str(),
                    is_hidden: false,
                    allow_desktop_config: true,
                    allow_overlay: true,
                    open_vr: 0,
                    dev_kit: 0,
                    dev_kit_game_id: "",
                    dev_kit_overrite_app_id: 0,
                    last_play_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32,
                    tags: vec!["twintaillauncher", "ttl"],
                };

                if flatpak_steam.exists() {
                    let status = add_steam_shortcut(flatpak_steam, install.name.as_str(), shortcut.clone());
                    if status {
                        update_install_shortcut_is_steam_by_id(&app, install.id.clone(), true);
                        show_dialog(&app, "info", "TwintailLauncher", format!("Successfully added {} to Steam (Flatpak), please restart Steam to apply changes.", install.name.as_str()).as_str(), None);
                    } else { show_dialog(&app, "error", "TwintailLauncher", format!("Failed to add {} to Steam (Flatpak)!", install.name.as_str()).as_str(), None); }
                }

                if normal_steam.exists() {
                    let status = add_steam_shortcut(normal_steam, install.name.as_str(), shortcut);
                    if status {
                        update_install_shortcut_is_steam_by_id(&app, install.id.clone(), true);
                        show_dialog(&app, "info", "TwintailLauncher", format!("Successfully added {} to Steam, please restart Steam to apply changes.", install.name.as_str()).as_str(), None);
                    } else { show_dialog(&app, "error", "TwintailLauncher", format!("Failed to add {} to Steam!", install.name.as_str()).as_str(), None); }
                }
            }
            _ => {}
        }
    };

    #[cfg(target_os = "windows")]
    {
        match shortcut_type.as_str() {
            "desktop" => {
                let base = app.path().desktop_dir().unwrap();
                let bin_name = app.path().app_local_data_dir().unwrap().join("twintaillauncher.exe");
                let file = base.join(format!("{}.lnk", install.name.as_str()));
                let sl = shortcuts_rs::ShellLink::new(bin_name.as_path(), Some(format!("--install={}", install.id.as_str())), Some(install.name.clone()), None).unwrap();
                let r = sl.create_lnk(file.as_path());
                if r.is_ok() {
                    update_install_shortcut_location_by_id(&app, install.id.clone(), file.clone().to_str().unwrap().to_string());
                    show_dialog(&app, "info", "TwintailLauncher", "Successfully created desktop shortcut. Find it on your desktop.", None);
                } else { show_dialog(&app, "error", "TwintailLauncher", "Failed to create launch shortcut!", None); }
            }
            "steam" => { show_dialog(&app, "warning", "TwintailLauncher", "Steam shortcuts are currently not supported on Windows!", None); }
            _ => {}
        }
    }
}

#[tauri::command]
pub fn remove_shortcut(app: AppHandle, install_id: String, shortcut_type: String) {
    let install = get_install_info_by_id(&app, install_id).unwrap();
    #[cfg(target_os = "linux")]
    {
        match shortcut_type.as_str() {
            "desktop" => {
                let base = app.path().home_dir().unwrap().join(".local/share/applications");
                let file = base.join(format!("{}.desktop", install.name.as_str()));

                let status = remove_desktop_shortcut(file.clone());
                if status {
                    update_install_shortcut_location_by_id(&app, install.id.clone(), "".to_string());
                    show_dialog(&app, "info", "TwintailLauncher", format!("Successfully deleted {} desktop shortcut.", install.name.as_str()).as_str(), None);
                } else { show_dialog(&app, "error", "TwintailLauncher", format!("Desktop shortcut for {} does not exist!", install.name.as_str()).as_str(), None); }
            }
            "steam" => {
                let flatpak_steam = app.path().home_dir().unwrap().join(".var/app/com.valvesoftware.Steam/data/Steam/userdata");
                let normal_steam = crate::utils::shortcuts::resolve_normal_steam_userdata(app.path().home_dir().unwrap());

                if flatpak_steam.exists() {
                    let status = remove_steam_shortcut(flatpak_steam, install.name.as_str());
                    if status {
                        update_install_shortcut_is_steam_by_id(&app, install.id.clone(), false);
                        show_dialog(&app, "info", "TwintailLauncher", format!("Successfully removed {} from Steam (Flatpak), please restart Steam to apply changes.", install.name.as_str()).as_str(), None);
                    } else {
                        // If flatpak Steam somehow exists but has no shortcut this will trigger an edge case with DB state
                        update_install_shortcut_is_steam_by_id(&app, install.id.clone(), false);
                        show_dialog(&app, "error", "TwintailLauncher", format!("Failed to remove {} from Steam (Flatpak)! Shortcut was most likely manually deleted.", install.name.as_str()).as_str(), None);
                    }
                }

                if normal_steam.exists() {
                    let status = remove_steam_shortcut(normal_steam, install.name.as_str());
                    if status {
                        update_install_shortcut_is_steam_by_id(&app, install.id.clone(), false);
                        show_dialog(&app, "info", "TwintailLauncher", format!("Successfully removed {} from Steam, please restart Steam to apply changes.", install.name.as_str()).as_str(), None);
                    } else {
                        // If normal Steam somehow exists but has no shortcut this will trigger an edge case with DB state
                        update_install_shortcut_is_steam_by_id(&app, install.id.clone(), false);
                        show_dialog(&app, "error", "TwintailLauncher", format!("Failed to remove {} from Steam! Shortcut was most likely manually deleted.", install.name.as_str()).as_str(), None);
                    }
                }
            }
            _ => {}
        }
    };

    #[cfg(target_os = "windows")]
    {
        match shortcut_type.as_str() {
            "desktop" => {
                let base = app.path().desktop_dir().unwrap();
                let file = base.join(format!("{}.lnk", install.name.as_str()));

                let status = remove_desktop_shortcut(file.clone());
                if status {
                    update_install_shortcut_location_by_id(&app, install.id.clone(), "".to_string());
                    show_dialog(&app, "info", "TwintailLauncher", "Successfully deleted desktop shortcut.", None);
                } else { show_dialog(&app, "warning", "TwintailLauncher", "Desktop shortcut for this game does not exist!", None); }
            }
            "steam" => { show_dialog(&app, "warning", "TwintailLauncher", "Steam shortcuts are currently not supported on Windows!", None); }
            _ => {}
        }
    };
}


#[tauri::command]
pub fn copy_authkey(app: AppHandle, id: String) -> bool {
    let install = get_install_info_by_id(&app, id).unwrap();
    fn get_engine_log_from_game(game_name: String, region_code: String) -> String {
        if game_name.to_ascii_lowercase().contains("genshin") { return "miHoYo/Genshin Impact/output_log.txt".to_string() }
        if game_name.to_ascii_lowercase().contains("starrail") { return "Cognosphere/Star Rail/Player.log".to_string() }
        if game_name.to_ascii_lowercase().contains("zenless") { return "miHoYo/ZenlessZoneZero/Player.log".to_string() }
        if game_name.to_ascii_lowercase().contains("honkai") {
            if region_code.to_ascii_lowercase().contains("glb_official") { return "miHoYo/Honkai Impact 3rd/Player.log".to_string() }
            if region_code.to_ascii_lowercase().contains("overseas_official") { return "miHoYo/Honkai Impact 3/Player.log".to_string() }
            if region_code.to_ascii_lowercase().contains("kr_official") { return "miHoYo/붕괴3rd/Player.log".to_string() }
            if region_code.to_ascii_lowercase().contains("asia_offcial") { return "miHoYo/崩壊3rd/Player.log".to_string() }
            if region_code.to_ascii_lowercase().contains("jp_official") { return "miHoYo/崩壊3rd/Player.log".to_string() }
            return "miHoYo/Honkai Impact 3rd/Player.log".to_string()
        }
        "".to_string()
    }

    #[cfg(target_os = "linux")]
    {
        let prefix = Path::new(&install.runner_prefix).to_path_buf();
        let prefix_exists = prefix.join("pfx/").exists();
        if prefix_exists {
            let engine_log = prefix.join("pfx/drive_c/users/steamuser/AppData/LocalLow/").join(get_engine_log_from_game(install.name, install.region_code));
            if engine_log.exists() {
                let log_content = fs::read_to_string(engine_log);
                return match log_content {
                    Ok(content) => {
                        let authkey = extract_authkey_from_content(&content);
                        if let Some(authkey) = authkey { match app.clipboard().write_text(authkey) { Ok(_) => true, Err(_) => false } } else { false }
                    },
                    Err(_) => { false }
                }
            } else { return false }
        } else { return false }
    }

    #[cfg(target_os = "windows")]
    {
        let base = app.path().home_dir().unwrap().join("AppData/LocalLow/");
        let engine_log = base.join(get_engine_log_from_game(install.name, install.region_code));
        if engine_log.exists() {
            let log_content = fs::read_to_string(engine_log);
            match log_content {
                Ok(content) => {
                    let authkey = extract_authkey_from_content(&content);
                    if let Some(authkey) = authkey { match app.clipboard().write_text(authkey) { Ok(_) => true, Err(_) => false } } else { false }
                },
                Err(_) => { false }
            }
        } else { false }
    }
}

fn enqueue_extras_download(app: &AppHandle, path: String, package_id: String, package_type: String, update_mode: bool) {
    let state = app.state::<DownloadState>();
    let q = state.queue.lock().unwrap().clone();
    if let Some(queue) = q { if !queue.has_job_for_id(package_type.clone()) { queue.enqueue(QueueJobKind::ExtrasDownload, QueueJobPayload::Extras(ExtrasDownloadPayload { path, package_id, package_type, update_mode })); } }
}

pub fn cancel_download_for_install(app: &AppHandle, install_id: &str) {
    let state = app.state::<DownloadState>();
    // 1. Signal any running download to stop
    {
        let tokens = state.tokens.lock().unwrap();
        if let Some(token) = tokens.get(install_id) {
            token.store(true, Ordering::Relaxed);
        }
    }
    // 2. Remove any queued jobs for this install
    {
        let queue_guard = state.queue.lock().unwrap();
        if let Some(ref queue_handle) = *queue_guard {
            queue_handle.remove_by_install_id(install_id.to_string());
        }
    }
    // 3. Clean up verified files tracking
    {
        let mut verified = state.verified_files.lock().unwrap();
        verified.remove(install_id);
    }
}
