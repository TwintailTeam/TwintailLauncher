use std::collections::HashMap;
use std::fs;
use std::ops::Add;
use std::path::Path;
use std::sync::Arc;
use fischl::compat::Compat;
use fischl::download::{Compatibility, Extras};
use fischl::utils::{extract_archive, prettify_bytes};
use fischl::utils::free_space::available;
use tauri::{AppHandle, Emitter};
use crate::utils::db_manager::{create_installation, delete_installation_by_id, get_install_info_by_id, get_installs, get_installs_by_manifest_id, get_manifest_info_by_filename, get_manifest_info_by_id, get_settings, update_install_dxvk_location_by_id, update_install_dxvk_version_by_id, update_install_env_vars_by_id, update_install_fps_value_by_id, update_install_game_location_by_id, update_install_ignore_updates_by_id, update_install_launch_args_by_id, update_install_launch_cmd_by_id, update_install_pre_launch_cmd_by_id, update_install_prefix_location_by_id, update_install_runner_location_by_id, update_install_runner_version_by_id, update_install_skip_hash_check_by_id, update_install_use_fps_unlock_by_id, update_install_use_jadeite_by_id, update_install_use_xxmi_by_id};
use crate::utils::game_launch_manager::launch;
use crate::utils::{copy_dir_all, generate_cuid, runner_from_runner_version, AddInstallRsp, DownloadSizesRsp};
use crate::utils::repo_manager::{get_compatibility, get_manifest, GameVersion};

#[cfg(target_os = "linux")]
use tauri::{Manager};
use tauri_plugin_notification::NotificationExt;

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
pub fn add_install(app: AppHandle, manifest_id: String, version: String, audio_lang: String, name: String, mut directory: String, mut runner_path: String, mut dxvk_path: String, runner_version: String, dxvk_version: String, game_icon: String, game_background: String, ignore_updates: bool, skip_hash_check: bool, use_jadeite: bool, use_xxmi: bool, use_fps_unlock: bool, env_vars: String, pre_launch_command: String, launch_command: String, fps_value: String, runner_prefix: String, launch_args: String, skip_game_dl: bool) -> Option<AddInstallRsp> {
    if manifest_id.is_empty() || version.is_empty() || name.is_empty() || directory.is_empty() || runner_path.is_empty() || dxvk_path.is_empty() || game_icon.is_empty() || game_background.is_empty() {
        None
    } else {
        let cuid = generate_cuid();
        let m = manifest_id + ".json";
        let dbm = get_manifest_info_by_filename(&app, m.clone()).unwrap();
        let gm = get_manifest(&app, m.clone()).unwrap();
        let g = gm.game_versions.iter().find(|e| e.metadata.version == version).unwrap();

        let install_location = Path::new(directory.as_str()).to_path_buf();
        if !install_location.exists() {
            fs::create_dir_all(&install_location).unwrap();
        }
        directory = install_location.to_str().unwrap().to_string();
        
        #[cfg(target_os = "windows")]
        {
            dxvk_path = "".to_string();
            runner_path = "".to_string();
        }

        #[cfg(target_os = "linux")]
        {
            let data_path = app.path().app_data_dir().unwrap();
            let comppath = data_path.join("compatibility");
            let wine = comppath.join("runners");
            let dxvk = comppath.join("dxvk");

            // Remove prefix just in case
            if Path::exists(runner_prefix.as_ref()) { fs::remove_dir_all(&runner_prefix.clone()).unwrap(); }

            runner_path = wine.join(runner_version.clone()).to_str().unwrap().to_string();
            dxvk_path = dxvk.join(dxvk_version.clone()).to_str().unwrap().to_string();

            if !Path::exists(runner_path.as_ref()) { fs::create_dir_all(runner_path.clone()).unwrap(); }
            if !Path::exists(dxvk_path.as_ref()) { fs::create_dir_all(dxvk_path.clone()).unwrap(); }
            if !Path::exists(runner_prefix.as_ref()) { fs::create_dir_all(runner_prefix.clone()).unwrap(); }
            
            let archandle = Arc::new(app.clone());
            let runv = Arc::new(runner_version.clone());
            let dxvkpp = Arc::new(dxvk_path.clone());
            let runpp = Arc::new(runner_path.clone());
            let dxvkv = Arc::new(dxvk_version.clone());
            let rpp = Arc::new(runner_prefix.clone());

            std::thread::spawn(move || {
                let rm = get_compatibility(archandle.as_ref(), &runner_from_runner_version(runv.as_str().to_string()).unwrap()).unwrap();
                let rv = rm.versions.into_iter().filter(|v| v.version.as_str() == runv.as_str()).collect::<Vec<_>>();
                let runnerp = rv.get(0).unwrap().to_owned();
                let rp = Path::new(runpp.as_str()).to_path_buf();
                let dxp = Path::new(dxvkpp.as_str()).to_path_buf();

                // Download selected DXVK
                let dm = get_compatibility(archandle.as_ref(), &runner_from_runner_version(dxvkv.as_str().to_string()).unwrap()).unwrap();
                let dv = dm.versions.into_iter().filter(|v| v.version.as_str() == dxvkv.as_str()).collect::<Vec<_>>();
                let dxvkp = dv.get(0).unwrap().to_owned();
                if fs::read_dir(dxvkpp.as_str().to_string()).unwrap().next().is_none() { 
                    let r = Compatibility::download_dxvk(dxvkp.url, dxvkpp.as_str().to_string());
                    if r { extract_archive(dxp.join("dxvk.zip").to_str().unwrap().to_string(), dxp.to_str().unwrap().to_string(), true); }
                }

                if fs::read_dir(rp.as_path()).unwrap().next().is_none() {
                    archandle.emit("download_progress", runv.as_str().to_string()).unwrap();

                    let r0 = Compatibility::download_runner(runnerp.url, runpp.as_str().to_string());
                    if r0 {
                        let er = extract_archive(rp.join("runner.zip").to_str().unwrap().to_string(), rp.to_str().unwrap().to_string(), true);
                        let wine64 = if rm.paths.wine64.is_empty() { rm.paths.wine32 } else { rm.paths.wine64 };
                        let winebin = rp.join(wine64).to_str().unwrap().to_string();
                        let is_proton = rm.display_name.to_ascii_lowercase().contains("proton") && !rm.display_name.to_ascii_lowercase().contains("wine");

                        if is_proton {  } else {
                            let r1 = Compat::setup_prefix(winebin, rpp.as_str().to_string());
                            if r1.is_ok() && er {
                                let r = r1.unwrap();
                                let r2 = Compat::stop_processes(r.wine.binary.to_str().unwrap().to_string(), rpp.as_str().to_string(), false);
                                if r2.is_ok() {
                                    let da = Compat::add_dxvk(r.wine.binary.to_str().unwrap().to_string(), rpp.to_string(), dxvkpp.as_str().to_string(), false);
                                    if da.is_ok() {
                                        Compat::stop_processes(r.wine.binary.to_str().unwrap().to_string(), rpp.as_str().to_string(), false).unwrap();
                                        if skip_game_dl { archandle.emit("download_complete", runv.as_str().to_string()).unwrap(); }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    let wine64 = if rm.paths.wine64.is_empty() { rm.paths.wine32 } else { rm.paths.wine64 };
                    let winebin = rp.join(wine64).to_str().unwrap().to_string();
                    let is_proton = rm.display_name.to_ascii_lowercase().contains("proton") && !rm.display_name.to_ascii_lowercase().contains("wine");

                    if is_proton {  } else {
                        let r1 = Compat::setup_prefix(winebin, rpp.as_str().to_string());
                        if r1.is_ok() {
                            let r = r1.unwrap();
                            let r2 = Compat::stop_processes(r.wine.binary.to_str().unwrap().to_string(), rpp.as_str().to_string(), false);
                            if r2.is_ok() {
                                let da = Compat::add_dxvk(r.wine.binary.to_str().unwrap().to_string(), rpp.to_string(), dxvkpp.as_str().to_string(), false);
                                if da.is_ok() {
                                    Compat::stop_processes(r.wine.binary.to_str().unwrap().to_string(), rpp.as_str().to_string(), false).unwrap();
                                }
                            }
                        }
                    }
                }
            });
        }
        create_installation(&app, cuid.clone(), dbm.id, version, audio_lang, g.metadata.versioned_name.clone(), directory, runner_path, dxvk_path, runner_version, dxvk_version, g.assets.game_icon.clone(), g.assets.game_background.clone(), ignore_updates, skip_hash_check, use_jadeite, use_xxmi, use_fps_unlock, env_vars, pre_launch_command, launch_command, fps_value, runner_prefix, launch_args).unwrap();
        Some(AddInstallRsp {
            success: true,
            install_id: cuid.clone(),
            background: g.assets.game_background.clone()
        })
    }
}

#[tauri::command]
pub async fn remove_install(app: AppHandle, id: String, wipe_prefix: bool) -> Option<bool> {
    if id.is_empty() {
        None
    } else {
        let install = get_install_info_by_id(&app, id.clone());

        if install.is_some() {
            let i = install.unwrap();
            let installdir = i.directory;
            let prefixdir = i.runner_prefix;

            if wipe_prefix {
                if fs::exists(prefixdir.clone()).unwrap() { fs::remove_dir_all(prefixdir.clone()).unwrap(); }
            }

            if fs::exists(installdir.clone()).unwrap() { fs::remove_dir_all(installdir.clone()).unwrap(); }
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

        if !Path::exists(path.as_ref()) {
            fs::create_dir_all(path.clone()).unwrap();
        }

        // Initialize move only IF old path has files AND new path is empty directory
        if Path::exists(oldpath.as_ref().to_string().as_ref()) {
            if fs::read_dir(oldpath.as_ref()).unwrap().next().is_some() && fs::read_dir(&path).unwrap().next().is_none() {
                let op = oldpath.clone();
                std::thread::spawn(move || {
                    let ap = Path::new(op.as_ref());
                    copy_dir_all(&app1, ap, &path.clone(), installation_id, install_name.clone(), "Game".to_string()).unwrap();

                    let mut payload = HashMap::new();
                    payload.insert("install_name", install_name.clone());
                    payload.insert("install_type", "Game".to_string());
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

        if !Path::exists(path.as_ref()) {
            fs::create_dir_all(path.clone()).unwrap();
        }

        if Path::exists(oldpath.as_ref().to_string().as_ref()) {
            if fs::read_dir(oldpath.as_ref()).unwrap().next().is_some() && fs::read_dir(&path).unwrap().next().is_none() {
                let op = oldpath.clone();
                std::thread::spawn(move || {
                    let ap = Path::new(op.as_ref());
                    copy_dir_all(&app1, ap, &path.clone(), installation_id, install_name.clone(), "Runner".to_string()).unwrap();

                    let mut payload = HashMap::new();
                    payload.insert("install_name", install_name.clone());
                    payload.insert("install_type", "Runner".to_string());
                    app1.emit("move_complete", &payload).unwrap();
                });
            }
        }
        update_install_runner_location_by_id(&app, m.id, np);
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

        if !Path::exists(path.as_ref()) { fs::create_dir_all(path.clone()).unwrap(); }

        if Path::exists(oldpath.as_ref().to_string().as_ref()) {
            if fs::read_dir(oldpath.as_ref()).unwrap().next().is_some() && fs::read_dir(&path).unwrap().next().is_none() {
                let op = oldpath.clone();
                std::thread::spawn(move || {
                    let ap = Path::new(op.as_ref());
                    copy_dir_all(&app1, ap, &path.clone(), installation_id, install_name.clone(),"DXVK".to_string()).unwrap();

                    let mut payload = HashMap::new();
                    payload.insert("install_name", install_name.clone());
                    payload.insert("install_type", "DXVK".to_string());
                    app1.emit("move_complete", &payload).unwrap();
                });
            }
        }
        update_install_dxvk_location_by_id(&app, m.id, np);
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
        
        if fs::read_dir(&p).unwrap().next().is_none() && enabled {
            std::thread::spawn(move || {
                let dl = Extras::download_jadeite("mkrsym1/jadeite".parse().unwrap(), p.as_path().to_str().unwrap().parse().unwrap());
                if dl {
                    extract_archive(p.join("jadeite.zip").as_path().to_str().unwrap().parse().unwrap(), p.as_path().to_str().unwrap().parse().unwrap(), false);
                }
            });
        }
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

        update_install_use_xxmi_by_id(&app, m.id, enabled);

        if fs::read_dir(&p).unwrap().next().is_none() && enabled {
            std::thread::spawn(move || {
                let dl = Extras::download_xxmi("SpectrumQT/XXMI-Libs-Package".parse().unwrap(), p.as_path().to_str().unwrap().parse().unwrap(), true);
                app.emit("download_progress", String::from("XXMI Modding tool")).unwrap();
                if dl {
                    extract_archive(p.join("xxmi.zip").as_path().to_str().unwrap().parse().unwrap(), p.as_path().to_str().unwrap().parse().unwrap(), false);

                    let gimi = String::from("TTL-extras/GIMI-Package");
                    let srmi = String::from("TTL-extras/SRMI-Package");
                    let zzmi = String::from("TTL-extras/ZZMI-Package");
                    let wwmi = String::from("TTL-extras/WWMI-Package");
                    
                    /*let gimi = String::from("SilentNightSound/GIMI-Package");
                    let srmi = String::from("SpectrumQT/SRMI-Package");
                    let zzmi = String::from("leotorrez/ZZMI-Package");
                    let wwmi = String::from("SpectrumQT/WWMI-Package");*/
                    
                    let dl1 = Extras::download_xxmi_packages(gimi, srmi, zzmi, wwmi, p.as_path().to_str().unwrap().parse().unwrap(), true);
                    if dl1 {
                        extract_archive(p.join("gimi.zip").as_path().to_str().unwrap().parse().unwrap(), p.join("gimi").as_path().to_str().unwrap().parse().unwrap(), false);
                        extract_archive(p.join("srmi.zip").as_path().to_str().unwrap().parse().unwrap(), p.join("srmi").as_path().to_str().unwrap().parse().unwrap(), false);
                        extract_archive(p.join("zzmi.zip").as_path().to_str().unwrap().parse().unwrap(), p.join("zzmi").as_path().to_str().unwrap().parse().unwrap(), false);
                        extract_archive(p.join("wwmi.zip").as_path().to_str().unwrap().parse().unwrap(), p.join("wwmi").as_path().to_str().unwrap().parse().unwrap(), false);
                        
                        app.emit("download_complete", String::from("XXMI Modding tool")).unwrap();
                    }
                }
            });
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

        if fs::read_dir(&p).unwrap().next().is_none() && enabled {
            std::thread::spawn(move || {
                Extras::download_fps_unlock("mkrsym1/fpsunlock".parse().unwrap(), p.as_path().to_str().unwrap().parse().unwrap());
            });
        }
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

        if fs::read_dir(&p).unwrap().next().is_none() && m.use_fps_unlock {
            std::thread::spawn(move || {
                Extras::download_fps_unlock("mkrsym1/fpsunlock".parse().unwrap(), p.as_path().to_str().unwrap().parse().unwrap());
            });
        }
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

#[tauri::command]
pub fn update_install_runner_version(app: AppHandle, id: String, version: String) -> Option<bool> {
    let install = get_install_info_by_id(&app, id);

    if install.is_some() {
        let m = install.unwrap();
        let rp = m.runner_path.clone();
        let rpn = rp.replace(m.runner_version.as_str(), version.as_str());
        if !Path::exists(rpn.as_ref()) { fs::create_dir_all(rpn.clone()).unwrap(); }
        
        let archandle = Arc::new(app.clone());
        let runv = Arc::new(version.clone());
        let runpp = Arc::new(rpn.clone());
        let rpp = Arc::new(m.runner_prefix.clone());
        
        if fs::read_dir(rpn.as_str()).unwrap().next().is_none() { 
            std::thread::spawn(move || {
                let rm = get_compatibility(archandle.as_ref(), &runner_from_runner_version(runv.as_str().to_string()).unwrap()).unwrap();
                let rv = rm.versions.into_iter().filter(|v| v.version.as_str() == runv.as_str()).collect::<Vec<_>>();
                let runnerp = rv.get(0).unwrap().to_owned();
                let rp = Path::new(runpp.as_str()).to_path_buf();

                archandle.emit("download_progress", runv.as_str().to_string()).unwrap();

                let r0 = Compatibility::download_runner(runnerp.url, runpp.as_str().to_string());
                if r0 {
                    let er = extract_archive(rp.join("runner.zip").to_str().unwrap().to_string(), rp.to_str().unwrap().to_string(), true);
                    let wine64 = if rm.paths.wine64.is_empty() { rm.paths.wine32 } else { rm.paths.wine64 };
                    let winebin = rp.join(wine64).to_str().unwrap().to_string();

                    let is_proton = rm.display_name.to_ascii_lowercase().contains("proton") && !rm.display_name.to_ascii_lowercase().contains("wine");
                    if er {
                        if is_proton {  } else { Compat::update_prefix(winebin, rpp.as_str().to_string()).unwrap(); }
                        archandle.emit("download_complete", runv.as_str().to_string()).unwrap();
                    }
                }
            });
        } else {
            std::thread::spawn(move || {
                let rm = get_compatibility(archandle.as_ref(), &runner_from_runner_version(runv.as_str().to_string()).unwrap()).unwrap();
                let rp = Path::new(runpp.as_str()).to_path_buf();

                let wine64 = if rm.paths.wine64.is_empty() { rm.paths.wine32 } else { rm.paths.wine64 };
                let winebin = rp.join(wine64).to_str().unwrap().to_string();

                let is_proton = rm.display_name.to_ascii_lowercase().contains("proton") && !rm.display_name.to_ascii_lowercase().contains("wine");
                if is_proton {  } else { Compat::update_prefix(winebin, rpp.as_str().to_string()).unwrap(); }
            });
        }

        update_install_runner_version_by_id(&app, m.id.clone(), version);
        update_install_runner_location_by_id(&app, m.id, rpn);
        Some(true)
    } else {
        None
    }
}

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
        let rpp = Arc::new(m.runner_prefix.clone());
        let runv = Arc::new(m.runner_version.clone());
        let runp = Arc::new(m.runner_path.clone());
        
        if fs::read_dir(pn.as_str()).unwrap().next().is_none() {
            std::thread::spawn(move || {
                let rm = get_compatibility(archandle.as_ref(), &runner_from_runner_version(runv.as_str().to_string()).unwrap()).unwrap();
                let dm = get_compatibility(archandle.as_ref(), &runner_from_runner_version(dxvkv.as_str().to_string()).unwrap()).unwrap();
                let dv = dm.versions.into_iter().filter(|v| v.version.as_str() == dxvkv.as_str()).collect::<Vec<_>>();
                let dxp = dv.get(0).unwrap().to_owned();
                let dxpp = Path::new(dxpp.as_str()).to_path_buf();
                let rp = Path::new(runp.as_str()).to_path_buf();

                let is_proton = rm.display_name.to_ascii_lowercase().contains("proton") && !rm.display_name.to_ascii_lowercase().contains("wine");

                if is_proton {  } else {
                    archandle.emit("download_progress", runv.as_str().to_string()).unwrap();

                    let r0 = Compatibility::download_dxvk(dxp.url, dxpp.to_str().unwrap().to_string());
                    if r0 {
                        let er = extract_archive(dxpp.join("dxvk.zip").to_str().unwrap().to_string(), dxpp.to_str().unwrap().to_string(), true);
                        let wine64 = if rm.paths.wine64.is_empty() { rm.paths.wine32 } else { rm.paths.wine64 };
                        let winebin = rp.join(wine64).to_str().unwrap().to_string();

                        if er {
                            let r1 = Compat::remove_dxvk(winebin.clone(), rpp.as_str().to_string());
                            if r1.is_ok() {
                                Compat::add_dxvk(winebin, rpp.as_str().to_string(), dxpp.to_str().unwrap().to_string(), false).unwrap();
                                archandle.emit("download_complete", dxvkv.as_str().to_string()).unwrap();
                            }
                        }
                    }
                }
            });
        } else {
            std::thread::spawn(move || {
                let rm = get_compatibility(archandle.as_ref(), &runner_from_runner_version(runv.as_str().to_string()).unwrap()).unwrap();
                let dxpp = Path::new(dxpp.as_str()).to_path_buf();
                let rp = Path::new(runp.as_str()).to_path_buf();

                let is_proton = rm.display_name.to_ascii_lowercase().contains("proton") && !rm.display_name.to_ascii_lowercase().contains("wine");

                if is_proton {  } else {
                    let wine64 = if rm.paths.wine64.is_empty() { rm.paths.wine32 } else { rm.paths.wine64 };
                    let winebin = rp.join(wine64).to_str().unwrap().to_string();
                    let r1 = Compat::remove_dxvk(winebin.clone(), rpp.as_str().to_string());
                    if r1.is_ok() { Compat::add_dxvk(winebin, rpp.as_str().to_string(), dxpp.to_str().unwrap().to_string(), false).unwrap(); }
                }
            });
        }

        update_install_dxvk_version_by_id(&app, m.id.clone(), version);
        update_install_dxvk_location_by_id(&app, m.id, pn);
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn game_launch(app: AppHandle, id: String) -> Option<bool> {
    let install = get_install_info_by_id(&app, id);
    let global_settings = get_settings(&app).unwrap();

    if install.is_some() {
        let m = install.unwrap();
        let gmm = get_manifest_info_by_id(&app, m.clone().manifest_id).unwrap();
        let gm = get_manifest(&app, gmm.filename).unwrap();

        let rslt = launch(&app, m.clone(), gm, global_settings);
        if rslt.is_ok() {
            Some(true)
        } else {
            app.notification().builder().icon("dialog-error").title("TwintailLauncher").body("Failed to launch game! Please check game.log file inside game directory for more information.").show().unwrap();
            None
        }
    } else {
        app.notification().builder().icon("dialog-error").title("TwintailLauncher").body("Failed to find installation! How is this even possible? Some serious fuck up happened!").show().unwrap();
        None
    }
}

#[tauri::command]
pub fn get_download_sizes(app: AppHandle, biz: String, version: String, lang: String, path: String) -> Option<String> {
    let manifest = get_manifest(&app, biz + ".json");

    if manifest.is_some() {
        let m = manifest.unwrap();
        
        let entry = m.game_versions.into_iter().filter(|e| e.metadata.version == version).collect::<Vec<GameVersion>>();
        let g = entry.get(0).unwrap();
        let gs = g.game.full.iter().map(|x| x.decompressed_size.parse::<u64>().unwrap()).sum::<u64>();
        let mut fss = gs;
        if !g.audio.full.is_empty() {
            let audios: Vec<_> = g.audio.full.iter().filter(|x| x.language == lang).collect();
            let audio = audios.get(0).unwrap().decompressed_size.parse::<u64>().unwrap();
            fss = gs.add(audio);
        }
        
        let a = available(Path::new(&path));
        let stringified;
        
        if a.is_some() {
            stringified = serde_json::to_string(&DownloadSizesRsp {
                game_decompressed_size: prettify_bytes(fss),
                free_disk_space: prettify_bytes(a.unwrap()),
                game_decompressed_size_raw: fss,
                free_disk_space_raw: a.unwrap(),
            }).unwrap();
        } else {
            stringified = serde_json::to_string(&DownloadSizesRsp {
                game_decompressed_size: prettify_bytes(fss),
                free_disk_space: prettify_bytes(0),
                game_decompressed_size_raw: fss,
                free_disk_space_raw: 0,
            }).unwrap();
        };
        
        Some(stringified)
    } else {
        None
    }
}