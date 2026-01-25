use std::collections::HashMap;
use std::fs;
use std::path::{PathBuf, Path};
use fischl::download::Extras;
use tauri::{AppHandle, Emitter};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use crate::utils::{compare_version, empty_dir, find_package_version, prevent_exit, run_async_command, db_manager::{get_settings}};

#[cfg(target_os = "linux")]
use fischl::compat::{check_steamrt_update, download_steamrt};
#[cfg(target_os = "linux")]
use crate::utils::{send_notification};

#[cfg(target_os = "linux")]
pub fn download_or_update_steamrt(app: &AppHandle) {
    let gs = get_settings(app);

    if gs.is_some() {
        let s = gs.unwrap();
        let rp = Path::new(&s.default_runner_path);
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

pub fn check_extras_update(app: &AppHandle) {
    let gs = get_settings(app);
    if gs.is_some() {
        let s = gs.unwrap();
        let jadeite = Path::new(&s.jadeite_path).to_path_buf();
        let fpsunlock = Path::new(&s.fps_unlock_path).to_path_buf();
        let xxmi = Path::new(&s.xxmi_path).to_path_buf();
        let gimi = xxmi.join("gimi");
        let srmi = xxmi.join("srmi");
        let zzmi = xxmi.join("zzmi");
        let himi = xxmi.join("himi");
        let wwmi = xxmi.join("wwmi");

        let ver_jadeite = jadeite.join("VERSION.txt");
        let ver_fpsunlock = fpsunlock.join("VERSION.txt");
        let ver_xxmi = xxmi.join("VERSION.txt");
        let ver_gimi = gimi.join("VERSION.txt");
        let ver_srmi = srmi.join("VERSION.txt");
        let ver_zzmi = zzmi.join("VERSION.txt");
        let ver_himi = himi.join("VERSION.txt");
        let ver_wwmi = wwmi.join("VERSION.txt");

        if ver_jadeite.exists() {
            download_or_update_extra(app, jadeite, "jadeite".to_string(), "v5.0.1-hotfix".to_string(), true);
        } else if jadeite.exists() && fs::read_dir(&jadeite).ok().and_then(|mut d| d.next()).is_some() {
            empty_dir(&jadeite).unwrap();
            download_or_update_extra(app, jadeite, "jadeite".to_string(), "v5.0.1-hotfix".to_string(), false);
        }

        if ver_fpsunlock.exists() {
            download_or_update_extra(app, fpsunlock, "keqingunlock".to_string(), "keqing_unlock".to_string(), true);
        } else if fpsunlock.exists() && fs::read_dir(&fpsunlock).ok().and_then(|mut d| d.next()).is_some() {
            empty_dir(&fpsunlock).unwrap();
            download_or_update_extra(app, fpsunlock, "keqingunlock".to_string(), "keqing_unlock".to_string(), false);
        }

        if ver_xxmi.exists() {
            download_or_update_extra(app, xxmi.clone(), "xxmi".to_string(), "xxmi".to_string(), true);
        } else if xxmi.exists() && fs::read_dir(&xxmi).ok().and_then(|mut d| d.next()).is_some() {
            empty_dir(&xxmi).unwrap();
            download_or_update_extra(app, xxmi.clone(), "xxmi".to_string(), "xxmi".to_string(), false);
        }

        if ver_gimi.exists() {
            download_or_update_extra(app, xxmi.clone(), "xxmi".to_string(), "gimi".to_string(), true);
        } else if gimi.exists() && fs::read_dir(&gimi).ok().and_then(|mut d| d.next()).is_some() {
            empty_dir(&gimi).unwrap();
            download_or_update_extra(app, xxmi.clone(), "xxmi".to_string(), "gimi".to_string(), false);
        }

        if ver_srmi.exists() {
            download_or_update_extra(app, xxmi.clone(), "xxmi".to_string(), "srmi".to_string(), true);
        } else if srmi.exists() && fs::read_dir(&srmi).ok().and_then(|mut d| d.next()).is_some() {
            empty_dir(&srmi).unwrap();
            download_or_update_extra(app, xxmi.clone(), "xxmi".to_string(), "srmi".to_string(), false);
        }

        if ver_zzmi.exists() {
            download_or_update_extra(app, xxmi.clone(), "xxmi".to_string(), "zzmi".to_string(), true);
        } else if zzmi.exists() && fs::read_dir(&zzmi).ok().and_then(|mut d| d.next()).is_some() {
            empty_dir(&zzmi).unwrap();
            download_or_update_extra(app, xxmi.clone(), "xxmi".to_string(), "zzmi".to_string(), false);
        }

        if ver_himi.exists() {
            download_or_update_extra(app, xxmi.clone(), "xxmi".to_string(), "himi".to_string(), true);
        } else if himi.exists() && fs::read_dir(&himi).ok().and_then(|mut d| d.next()).is_some() {
            empty_dir(&himi).unwrap();
            download_or_update_extra(app, xxmi.clone(), "xxmi".to_string(), "himi".to_string(), false);
        }

        if ver_wwmi.exists() {
            download_or_update_extra(app, xxmi.clone(), "xxmi".to_string(), "wwmi".to_string(), true);
        } else if wwmi.exists() && fs::read_dir(&wwmi).ok().and_then(|mut d| d.next()).is_some() {
            empty_dir(&wwmi).unwrap();
            download_or_update_extra(app, xxmi.clone(), "xxmi".to_string(), "wwmi".to_string(), false);
        }
    }
}

pub fn download_or_update_extra(app: &AppHandle, path: PathBuf, package_id: String, package_type: String, update_mode: bool) {
    if update_mode {
        if fs::read_dir(&path).unwrap().next().is_some() {
            let app = app.clone();
            let path = path.clone();
            std::thread::spawn(move || {
                let manifest = Extras::fetch_ttl_manifest(package_id.clone());
                if let Some(m) = manifest {
                    if m.retcode != 0 {
                        app.dialog().message(format!("Error occurred while trying to update {package_id}! Please retry later.").as_str()).title("TwintailLauncher")
                            .kind(MessageDialogKind::Error)
                            .buttons(MessageDialogButtons::OkCustom("Ok".to_string()))
                            .show(move |_action| {
                                app.emit("update_complete", package_id.clone()).unwrap();
                            });
                        return;
                    } else {
                        let ap = if package_type.as_str() == "xxmi" || package_id.as_str() == "jadeite" || package_id == "keqingunlock" { path.clone() } else { path.join(&package_type) };
                        let ver_path = if package_id == "keqingunlock" || package_id == "jadeite" || package_type == "xxmi" { path.join("VERSION.txt") } else { path.join(package_type.clone()).join("VERSION.txt") };
                        if !ver_path.exists() { return; }
                        let pkg_type = if package_id == "keqingunlock" || package_id == "jadeite" { package_id.as_str() } else { package_type.as_str() };
                        let local_ver = find_package_version(ver_path.clone(), &pkg_type);
                        if local_ver.is_some() {
                            let lv = local_ver.unwrap();
                            let pkgs = m.data.unwrap();
                            let pkg = pkgs.packages.iter().find(|e| e.package_name.to_ascii_lowercase().contains(package_type.as_str()));
                            if let Some(p) = pkg {
                                if compare_version(lv.as_str(), p.version.as_str()).is_lt() {
                                    if package_type == "xxmi" {
                                        for file in &p.file_list {
                                            let f_path = path.join(file);
                                            if f_path.exists() { let _ = fs::remove_file(f_path); }
                                        }
                                    } else { empty_dir(&ap).unwrap(); }
                                    prevent_exit(&app, true);
                                    let dl = run_async_command(async {
                                        let needs_extract = if package_type.as_str() == "keqing_unlock" || package_type.as_str() == "xxmi" { false } else { true };
                                        let needs_append = if package_type.as_str() == "gimi" || package_type.as_str() == "srmi" || package_type.as_str() == "zzmi" || package_type.as_str() == "himi" || package_type.as_str() == "wwmi" { true } else { false };
                                        Extras::download_extra_package(package_id.clone(), package_type.clone(), needs_extract, false, needs_append, ap.as_path().to_str().unwrap().parse().unwrap(), |_current, _total| {}).await
                                    });
                                    if dl {
                                        if package_type.as_str() == "gimi" || package_type.as_str() == "srmi" || package_type.as_str() == "zzmi" || package_type.as_str() == "himi" || package_type.as_str() == "wwmi" {
                                            for mi in ["gimi", "srmi", "zzmi", "wwmi", "himi"] {
                                                for lib in ["d3d11.dll", "d3dcompiler_47.dll"] {
                                                    let linkedpath = path.join(mi).join(lib);
                                                    let _ = fs::remove_file(&linkedpath);
                                                    if !linkedpath.exists() {
                                                        #[cfg(target_os = "linux")]
                                                        std::os::unix::fs::symlink(path.join(lib), linkedpath).unwrap();
                                                        #[cfg(target_os = "windows")]
                                                        fs::copy(path.join(lib), linkedpath).unwrap();
                                                    }
                                                }
                                            }
                                        }
                                        app.emit("update_complete", package_id.clone()).unwrap();
                                        prevent_exit(&app, false);
                                    } else {
                                        app.dialog().message(format!("Error occurred while trying to update {package_id}! Please retry later.").as_str()).title("TwintailLauncher")
                                            .kind(MessageDialogKind::Error)
                                            .buttons(MessageDialogButtons::OkCustom("Ok".to_string()))
                                            .show(move |_action| {
                                                prevent_exit(&app, false);
                                                app.emit("update_complete", package_id.clone()).unwrap();
                                                empty_dir(&path).unwrap();
                                            });
                                    }
                                }
                            }
                        }
                    }
                }
            });
        }
    } else {
        let ap = if package_type.as_str() == "gimi" || package_type.as_str() == "srmi" || package_type.as_str() == "zzmi" || package_type.as_str() == "himi" || package_type.as_str() == "wwmi" { path.join(&package_type) } else { path.clone() };
        let entries: Vec<_> = fs::read_dir(&ap).ok().map(|r| r.filter_map(|e| e.ok()).collect()).unwrap_or_default();
        let is_effectively_empty = if package_type == "xxmi" { entries.iter().all(|e| { let name = e.file_name(); e.path().is_dir() && (name == "gimi" || name == "srmi" || name == "zzmi" || name == "himi" || name == "wwmi") }) } else { entries.is_empty() || entries.iter().all(|e| e.file_name().to_str().unwrap().contains("Mods") || e.file_name().to_str().unwrap().contains("ShaderCache") || e.file_name() == "d3dx_user.ini") };
        if is_effectively_empty {
            let app = app.clone();
            let path = path.clone();
            std::thread::spawn(move || {
                let mut dlpayload = HashMap::new();
                dlpayload.insert("name", package_id.clone().chars().next().map(|first| first.to_uppercase().collect::<String>() + &package_id[first.len_utf8()..]).unwrap_or_default());
                dlpayload.insert("progress", "0".to_string());
                dlpayload.insert("total", "1000".to_string());
                app.emit("download_progress", dlpayload.clone()).unwrap();
                prevent_exit(&app, true);

                if !ap.exists() { let _ = fs::create_dir_all(&ap); }
                let dl = run_async_command(async {
                    let needs_extract = if package_type.as_str() == "keqing_unlock" || package_type.as_str() == "xxmi" { false } else { true };
                    let needs_append = if package_type.as_str() == "gimi" || package_type.as_str() == "srmi" || package_type.as_str() == "zzmi" || package_type.as_str() == "himi" || package_type.as_str() == "wwmi" { true } else { false };
                    Extras::download_extra_package(package_id.clone(), package_type.clone(), needs_extract, false, needs_append, ap.as_path().to_str().unwrap().parse().unwrap(), {
                        let app = app.clone();
                        let pkg_id = package_id.clone();
                        let dlpayload = dlpayload.clone();
                        move |current, total| {
                            let mut dlpayload = dlpayload.clone();
                            dlpayload.insert("name", pkg_id.clone().chars().next().map(|first| first.to_uppercase().collect::<String>() + &pkg_id[first.len_utf8()..]).unwrap_or_default());
                            dlpayload.insert("progress", current.to_string());
                            dlpayload.insert("total", total.to_string());
                            app.emit("download_progress", dlpayload.clone()).unwrap();
                        }
                    }).await
                });
                if dl {
                    if package_type.as_str() == "gimi" || package_type.as_str() == "srmi" || package_type.as_str() == "zzmi" || package_type.as_str() == "himi" || package_type.as_str() == "wwmi" {
                        for mi in ["gimi", "srmi", "zzmi", "wwmi", "himi"] {
                            for lib in ["d3d11.dll", "d3dcompiler_47.dll"] {
                                let linkedpath = path.join(mi).join(lib);
                                let _ = fs::remove_file(&linkedpath);
                                if !linkedpath.exists() {
                                    #[cfg(target_os = "linux")]
                                    std::os::unix::fs::symlink(path.join(lib), linkedpath).unwrap();
                                    #[cfg(target_os = "windows")]
                                    fs::copy(path.join(lib), linkedpath).unwrap();
                                }
                            }
                        }
                    }
                    app.emit("download_complete", package_id.clone()).unwrap();
                    prevent_exit(&app, false);
                } else {
                    app.dialog().message(format!("Error occurred while trying to download {package_id}! Please retry later.").as_str()).title("TwintailLauncher")
                        .kind(MessageDialogKind::Error)
                        .buttons(MessageDialogButtons::OkCustom("Ok".to_string()))
                        .show(move |_action| {
                            prevent_exit(&app, false);
                            app.emit("download_complete", package_id.clone()).unwrap();
                            empty_dir(&path).unwrap();
                        });
                }
            });
        }
    }
}