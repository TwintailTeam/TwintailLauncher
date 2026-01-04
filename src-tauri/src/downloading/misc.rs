use std::collections::HashMap;
use std::fs;
use std::path::{PathBuf};
use fischl::download::Extras;
use fischl::utils::extract_archive;
use tauri::{AppHandle, Emitter};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use crate::utils::db_manager::{get_install_info_by_id, get_manifest_info_by_id, update_install_xxmi_config_by_id};
use crate::utils::{apply_xxmi_tweaks, empty_dir, get_mi_path_from_game, prevent_exit, run_async_command};
use crate::utils::repo_manager::get_manifest;

#[cfg(target_os = "linux")]
use fischl::compat::{check_steamrt_update, download_steamrt};
#[cfg(target_os = "linux")]
use std::path::Path;
#[cfg(target_os = "linux")]
use crate::utils::{PathResolve, send_notification, db_manager::{get_settings}};

#[cfg(target_os = "linux")]
pub fn download_or_update_steamrt(app: &AppHandle) {
    let gs = get_settings(app);

    if gs.is_some() {
        let s = gs.unwrap();
        let rp = Path::new(&s.default_runner_path).follow_symlink().unwrap();
        let steamrt = rp.join("steamrt");
        if !steamrt.exists() {
            let r = fs::create_dir_all(&steamrt);
            match r {
                Ok(_) => {},
                Err(e) => { send_notification(&app, format!("Failed to prepare SteamLinuxRuntime directory. {} - Please fix the error and restart the app!", e.to_string()).as_str(), None); return; }
            }
        }

        if fs::read_dir(&steamrt).unwrap().next().is_none() {
            let app = app.clone();
            std::thread::spawn(move || {
                let app = app.clone();
                let mut dlpayload = HashMap::new();
                dlpayload.insert("name", String::from("SteamLinuxRuntime 3"));
                dlpayload.insert("progress", "0".to_string());
                dlpayload.insert("total", "1000".to_string());
                app.emit("download_progress", dlpayload.clone()).unwrap();
                prevent_exit(&app, true);

                let r = run_async_command(async {
                    download_steamrt(steamrt.clone(), steamrt.clone(), "steamrt3".to_string(), "latest-public-beta".to_string(), {
                        let app = app.clone();
                        let dlpayload = dlpayload.clone();
                        move |current, total| {
                            let mut dlpayload = dlpayload.clone();
                            dlpayload.insert("name", "SteamLinuxRuntime 3".to_string());
                            dlpayload.insert("progress", current.to_string());
                            dlpayload.insert("total", total.to_string());
                            app.emit("download_progress", dlpayload.clone()).unwrap();
                        }
                    }).await
                });
                if r {
                    app.emit("download_complete", String::from("SteamLinuxRuntime 3")).unwrap();
                    prevent_exit(&app, false);
                } else {
                    app.dialog().message("Error occurred while trying to download SteamLinuxRuntime! Please restart the application to retry.").title("TwintailLauncher")
                        .kind(MessageDialogKind::Error)
                        .buttons(MessageDialogButtons::OkCustom("Ok".to_string()))
                        .show(move |_action| {
                            prevent_exit(&app, false);
                            app.emit("download_complete", String::from("SteamLinuxRuntime 3")).unwrap();
                            empty_dir(steamrt.as_path()).unwrap();
                        });
                }
            });
        } else {
            let vp = steamrt.join("VERSIONS.txt");
            if !vp.exists() { return; }
            let cur_ver = crate::utils::find_steamrt_version(vp).unwrap();
            if cur_ver.is_empty() { return; }
            let remote_ver = check_steamrt_update("steamrt3".to_string(), "latest-public-beta".to_string());
            if remote_ver.is_some() {
                let rv = remote_ver.unwrap();
                if crate::utils::compare_steamrt_versions(&rv, &cur_ver) {
                    empty_dir(steamrt.as_path()).unwrap();
                    let app = app.clone();
                    std::thread::spawn(move || {
                        let app = app.clone();
                        let mut dlpayload = HashMap::new();
                        dlpayload.insert("name", String::from("SteamLinuxRuntime 3"));
                        dlpayload.insert("progress", "0".to_string());
                        dlpayload.insert("total", "1000".to_string());
                        app.emit("update_progress", dlpayload.clone()).unwrap();
                        prevent_exit(&app, true);

                        let r = run_async_command(async {
                            download_steamrt(steamrt.clone(), steamrt.clone(), "steamrt3".to_string(), "latest-public-beta".to_string(), {
                                let app = app.clone();
                                let dlpayload = dlpayload.clone();
                                move |current, total| {
                                    let mut dlpayload = dlpayload.clone();
                                    dlpayload.insert("name", "SteamLinuxRuntime 3".to_string());
                                    dlpayload.insert("progress", current.to_string());
                                    dlpayload.insert("total", total.to_string());
                                    app.emit("update_progress", dlpayload.clone()).unwrap();
                                }
                            }).await
                        });
                        if r {
                            app.emit("update_complete", String::from("SteamLinuxRuntime 3")).unwrap();
                            prevent_exit(&app, false);
                        } else {
                            app.dialog().message("Error occurred while trying to update SteamLinuxRuntime! Please restart the application to retry.").title("TwintailLauncher")
                                .kind(MessageDialogKind::Error)
                                .buttons(MessageDialogButtons::OkCustom("Ok".to_string()))
                                .show(move |_action| {
                                    prevent_exit(&app, false);
                                    app.emit("update_complete", String::from("SteamLinuxRuntime 3")).unwrap();
                                    empty_dir(steamrt.as_path()).unwrap();
                                });
                        }
                    });
                } else { println!("SteamLinuxRuntime is up to date!"); }
            }
        }
    }
}

#[allow(unused_variables)]
pub fn download_or_update_xxmi(app: &AppHandle, path: PathBuf, install_id: Option<String>, update_mode: bool) {
    if update_mode {
        if fs::read_dir(&path).unwrap().next().is_some() {
            let app = app.clone();
            std::thread::spawn(move || {
                let dl = run_async_command(async {
                    Extras::download_xxmi("SpectrumQT/XXMI-Libs-Package".parse().unwrap(), path.as_path().to_str().unwrap().parse().unwrap(), true, {
                        move |_current, _total| {}
                    }).await
                });
                if dl {
                    extract_archive(path.join("xxmi.zip").as_path().to_str().unwrap().parse().unwrap(), path.as_path().to_str().unwrap().parse().unwrap(), false);
                    let gimi = String::from("SilentNightSound/GIMI-Package");
                    let srmi = String::from("SpectrumQT/SRMI-Package");
                    let zzmi = String::from("leotorrez/ZZMI-Package");
                    let wwmi = String::from("SpectrumQT/WWMI-Package");
                    let himi = String::from("leotorrez/HIMI-Package");

                    let dl1 = run_async_command(async {
                        Extras::download_xxmi_packages(gimi, srmi, zzmi, wwmi, himi, path.as_path().to_str().unwrap().parse().unwrap()).await
                    });
                    if dl1 {
                        for mi in ["gimi", "srmi", "zzmi", "wwmi", "himi"] {
                            extract_archive(path.join(format!("{mi}.zip")).as_path().to_str().unwrap().parse().unwrap(), path.join(mi).as_path().to_str().unwrap().parse().unwrap(), false);
                            for lib in ["d3d11.dll", "d3dcompiler_47.dll"] {
                                let linkedpath = path.join(mi).join(lib);
                                if !linkedpath.exists() {
                                    #[cfg(target_os = "linux")]
                                    std::os::unix::fs::symlink(path.join(lib), linkedpath).unwrap();
                                    #[cfg(target_os = "windows")]
                                    fs::copy(path.join(lib), linkedpath).unwrap();
                                }
                            }
                        }
                        if let Some(id) = install_id {
                            let ai = get_install_info_by_id(&app, id).unwrap();
                            let repm = get_manifest_info_by_id(&app, ai.manifest_id).unwrap();
                            let gm = get_manifest(&app, repm.filename).unwrap();
                            let exe = gm.paths.exe_filename.clone().split('/').last().unwrap().to_string();
                            let mi = get_mi_path_from_game(exe).unwrap();
                            let base = path.join(mi);
                            let data = apply_xxmi_tweaks(base, ai.xxmi_config);
                            update_install_xxmi_config_by_id(&app, ai.id, data);
                        }
                    }
                }
            });
        }
    } else {
        if fs::read_dir(&path).unwrap().next().is_none() {
            let app = app.clone();
            let path = path.clone();
            std::thread::spawn(move || {
                let mut dlpayload = HashMap::new();
                dlpayload.insert("name", String::from("XXMI Modding tool"));
                dlpayload.insert("progress", "0".to_string());
                dlpayload.insert("total", "1000".to_string());
                app.emit("download_progress", dlpayload.clone()).unwrap();
                prevent_exit(&app, true);
                let dl = run_async_command(async {
                    Extras::download_xxmi("SpectrumQT/XXMI-Libs-Package".parse().unwrap(), path.as_path().to_str().unwrap().parse().unwrap(), true, {
                        let app = app.clone();
                        let dlpayload = dlpayload.clone();
                        move |current, total| {
                            let mut dlpayload = dlpayload.clone();
                            dlpayload.insert("name", "XXMI Modding tool".to_string());
                            dlpayload.insert("progress", current.to_string());
                            dlpayload.insert("total", total.to_string());
                            app.emit("download_progress", dlpayload.clone()).unwrap();
                        }
                    }).await
                });
                if dl {
                    extract_archive(path.join("xxmi.zip").as_path().to_str().unwrap().parse().unwrap(), path.as_path().to_str().unwrap().parse().unwrap(), false);
                    let gimi = String::from("SilentNightSound/GIMI-Package");
                    let srmi = String::from("SpectrumQT/SRMI-Package");
                    let zzmi = String::from("leotorrez/ZZMI-Package");
                    let wwmi = String::from("SpectrumQT/WWMI-Package");
                    let himi = String::from("leotorrez/HIMI-Package");

                    let dl1 = run_async_command(async {
                        Extras::download_xxmi_packages(gimi, srmi, zzmi, wwmi, himi, path.as_path().to_str().unwrap().parse().unwrap()).await
                    });
                    if dl1 {
                        for mi in ["gimi", "srmi", "zzmi", "wwmi", "himi"] {
                            extract_archive(path.join(format!("{mi}.zip")).as_path().to_str().unwrap().parse().unwrap(), path.join(mi).as_path().to_str().unwrap().parse().unwrap(), false);
                            for lib in ["d3d11.dll", "d3dcompiler_47.dll"] {
                                let linkedpath = path.join(mi).join(lib);
                                if !linkedpath.exists() {
                                    #[cfg(target_os = "linux")]
                                    std::os::unix::fs::symlink(path.join(lib), linkedpath).unwrap();
                                    #[cfg(target_os = "windows")]
                                    fs::copy(path.join(lib), linkedpath).unwrap();
                                }
                            }
                        }
                        app.emit("download_complete", String::from("XXMI Modding tool")).unwrap();
                        prevent_exit(&app, false);
                        if let Some(id) = install_id {
                            let ai = get_install_info_by_id(&app, id).unwrap();
                            let repm = get_manifest_info_by_id(&app, ai.manifest_id).unwrap();
                            let gm = get_manifest(&app, repm.filename).unwrap();
                            let exe = gm.paths.exe_filename.clone().split('/').last().unwrap().to_string();
                            let mi = get_mi_path_from_game(exe).unwrap();
                            let base = path.join(mi);
                            let data = apply_xxmi_tweaks(base, ai.xxmi_config);
                            update_install_xxmi_config_by_id(&app, ai.id, data);
                        }
                    }
                } else {
                    app.dialog().message("Error occurred while trying to download XXMI Modding tool! Please retry later by re-enabling the \"Inject XXMI\" in Install Settings.").title("TwintailLauncher")
                        .kind(MessageDialogKind::Error)
                        .buttons(MessageDialogButtons::OkCustom("Ok".to_string()))
                        .show(move |_action| {
                            prevent_exit(&app, false);
                            app.emit("download_complete", String::from("XXMI Modding tool")).unwrap();
                            empty_dir(&path).unwrap();
                        });
                }
            });
        }
    }
}

pub fn download_or_update_jadeite(path: PathBuf, update_mode: bool) {
    if update_mode {
        if fs::read_dir(&path).unwrap().next().is_some() {
            std::thread::spawn(move || {
                empty_dir(&path).unwrap();
                let dl = run_async_command(async {
                    Extras::download_jadeite("MrLGamer/jadeite".parse().unwrap(), path.as_path().to_str().unwrap().parse().unwrap(), |_current, _total| {}).await
                });
                if dl { extract_archive(path.join("jadeite.zip").as_path().to_str().unwrap().parse().unwrap(), path.as_path().to_str().unwrap().parse().unwrap(), false); }
            });
        }
    } else {
        if fs::read_dir(&path).unwrap().next().is_none() {
            std::thread::spawn(move || {
                let dl = run_async_command(async {
                    Extras::download_jadeite("MrLGamer/jadeite".parse().unwrap(), path.as_path().to_str().unwrap().parse().unwrap(), |_current, _total| {}).await
                });
                if dl { extract_archive(path.join("jadeite.zip").as_path().to_str().unwrap().parse().unwrap(), path.as_path().to_str().unwrap().parse().unwrap(), false); }
            });
        }
    }
}

pub fn download_or_update_fps_unlock(path: PathBuf, update_mode: bool) {
    if update_mode {
        if fs::read_dir(&path).unwrap().next().is_some() {
            std::thread::spawn(move || {
                empty_dir(&path).unwrap();
                run_async_command(async {
                    Extras::download_fps_unlock("TwintailTeam/KeqingUnlock".parse().unwrap(), path.as_path().to_str().unwrap().parse().unwrap(), |_current, _total| {}).await
                });
            });
        }
    } else {
        if fs::read_dir(&path).unwrap().next().is_none() {
            std::thread::spawn(move || {
                run_async_command(async {
                    Extras::download_fps_unlock("TwintailTeam/KeqingUnlock".parse().unwrap(), path.as_path().to_str().unwrap().parse().unwrap(), |_current, _total| {}).await
                });
            });
        }
    }
}