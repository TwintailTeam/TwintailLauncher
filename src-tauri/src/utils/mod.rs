use std::{fs, io};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use fischl::download::game::{Game, Hoyo, Kuro, Sophon};
use fischl::utils::{assemble_multipart_archive, extract_archive};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Listener, Manager};
use tauri_plugin_notification::NotificationExt;
use crate::utils::db_manager::{get_install_info_by_id, get_manifest_info_by_id, update_install_after_update_by_id};
use crate::utils::repo_manager::{get_manifest, DiffGameFile, GameVersion};

#[cfg(target_os = "linux")]
use fischl::utils::patch_aki;
#[cfg(target_os = "linux")]
use std::process::Command;
#[cfg(target_os = "linux")]
use crate::utils::repo_manager::get_manifests;

pub mod db_manager;
pub mod repo_manager;
mod git_helpers;
pub mod game_launch_manager;
pub mod system_tray;

pub fn generate_cuid() -> String {
    cuid2::create_id()
}

pub fn run_async_command<F: Future>(cmd: F) -> F::Output {
    if tokio::runtime::Handle::try_current().is_ok() {
        tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(cmd))
    } else {
        tauri::async_runtime::block_on(cmd)
    }
}

pub fn copy_dir_all(app: &AppHandle, src: impl AsRef<Path>, dst: impl AsRef<Path>, install: String, install_name: String, install_type: String) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    let totalsize = dir_size(src.as_ref())?;
    let tracker = Arc::new(AtomicU64::new(0));

    prevent_exit(app, true);
    for entry in fs::read_dir(src.as_ref())? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let f = entry.file_name();
        let ep = entry.path();

        if ep == dst.as_ref() { continue; }

        if ty.is_dir() {
            copy_dir_all(&app, ep.clone(), dst.as_ref().join(f), install.clone(), install_name.clone(), install_type.clone())?;
            fs::remove_dir_all(ep)?;
        } else {
            let size = entry.metadata()?.len();
            tracker.fetch_add(size, Ordering::SeqCst);

            let mut payload = HashMap::new();
            payload.insert("file", f.to_str().unwrap().to_string());
            payload.insert("install_id", install.clone());
            payload.insert("install_name", install_name.clone());
            payload.insert("install_type", install_type.clone());
            payload.insert("progress", tracker.load(Ordering::SeqCst).to_string());
            payload.insert("total", totalsize.to_string());

            fs::copy(ep.clone(), dst.as_ref().join(f))?;
            app.emit("move_progress", &payload).unwrap();
            fs::remove_file(ep)?;
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn block_telemetry(app: &AppHandle) {
    let app1 = Arc::new(Mutex::new(app.clone()));
        std::thread::spawn(move || {
            let app = app1.lock().unwrap().clone();
            let manifests = get_manifests(&app);
            let mut allhosts = String::new();

            manifests.values().for_each(|manifest| {
                let hosts = manifest.telemetry_hosts.iter().map(|server| format!("echo '0.0.0.0 {server}' >> /etc/hosts")).collect::<Vec<String>>().join(" ; ");
                allhosts.push_str(&hosts);
                allhosts.push_str(" ; ");
            });

            if !allhosts.is_empty() { allhosts = allhosts.trim_end_matches(" ; ").to_string(); }

            let output = Command::new("pkexec")
                .arg("bash").arg("-c").arg(format!("echo '' >> /etc/hosts ; echo '# TwintailLauncher telemetry block start' >> /etc/hosts ; {allhosts} ; echo '# TwintailLauncher telemetry block end' >> /etc/hosts")).spawn();

            match output.and_then(|child| child.wait_with_output()) {
                Ok(output) => if !output.status.success() { send_notification(&app, r#"Failed to block telemetry servers, Please press "Block telemetry" in launcher settings!"#, None);
                } else {
                    let path = app.path().app_data_dir().unwrap().join(".telemetry_blocked");
                    if !path.exists() {
                        send_notification(&app, "Successfully blocked telemetry servers.", None);
                        fs::write(&path, ".").unwrap();
                    } else { send_notification(&app, "Telemetry servers already blocked.", None); }
                }
                Err(_err) => { send_notification(&app, r#"Failed to block telemetry servers, Please press "Block telemetry" in launcher settings!"#, None); }
            }
        });
}

#[cfg(target_os = "windows")]
pub fn block_telemetry(_app: &AppHandle) {}

pub fn register_listeners(app: &AppHandle) {
    let h1 = app.clone();
    app.listen("launcher_action_exit", move |_event| {
        let blocks = h1.state::<Mutex<ActionBlocks>>();
        let state = blocks.lock().unwrap();

        if state.action_exit { h1.get_window("main").unwrap().hide().unwrap(); } else {
            h1.cleanup_before_exit();
            h1.exit(0);
            std::process::exit(0);
        }
    });

    let h2 = app.clone();
    app.listen("launcher_action_minimize", move |_event| {
        h2.get_window("main").unwrap().minimize().unwrap();
    });

    // Start game download
    let h4 = app.clone();
    app.listen("start_game_download", move |event| {
        let h4 = h4.clone();
        std::thread::spawn(move || {
            let payload: DownloadGamePayload = serde_json::from_str(event.payload()).unwrap();
            let install = get_install_info_by_id(&h4, payload.install).unwrap(); // Should exist by now, if not we FUCKED UP
            let gid = get_manifest_info_by_id(&h4, install.manifest_id).unwrap();

            let mm = get_manifest(&h4, gid.filename);
            if let Some(gm) = mm {
                let version = gm.game_versions.iter().filter(|e| e.metadata.version == install.version).collect::<Vec<&GameVersion>>();
                let picked = version.get(0).unwrap();

                let instn = Arc::new(install.name.clone());
                let dlpayload = Arc::new(Mutex::new(HashMap::new()));

                let mut dlp = dlpayload.lock().unwrap();
                dlp.insert("name", install.name.clone());
                dlp.insert("progress", "0".to_string());
                dlp.insert("total", "1000".to_string());

                h4.emit("download_progress", dlp.clone()).unwrap();
                drop(dlp);
                prevent_exit(&h4, true);

                match picked.metadata.download_mode.as_str() {
                    // Generic zipped mode, PS: Currently only hoyo for backwards compatibility
                    "DOWNLOAD_MODE_FILE" => {
                        let urls = picked.game.full.iter().map(|v| v.file_url.clone()).collect::<Vec<String>>();
                        let totalsize = picked.game.full.iter().map(|x| x.compressed_size.parse::<u64>().unwrap()).sum::<u64>();
                        let rslt = <Game as Hoyo>::download(urls.clone(), install.directory.clone(), {
                            let dlpayload = dlpayload.clone();
                            let h4 = h4.clone();
                            move |current, _| {
                                let mut dlp = dlpayload.lock().unwrap();
                                dlp.insert("name", instn.to_string());
                                dlp.insert("progress", current.to_string());
                                dlp.insert("total", totalsize.to_string());
                                h4.emit("download_progress", dlp.clone()).unwrap();
                                drop(dlp);
                            }
                        });
                        if rslt {
                            // Get first entry in the list, and start extraction
                            let first = urls.get(0).unwrap();
                            let tmpf = first.split('/').collect::<Vec<&str>>();
                            let fnn = tmpf.last().unwrap().to_string();
                            let ap = Path::new(&install.directory).follow_symlink().unwrap();
                            let aps = ap.to_str().unwrap().to_string();
                            let parts = urls.into_iter().map(|e| e.split('/').collect::<Vec<&str>>().last().unwrap().to_string()).collect::<Vec<String>>();

                            if fnn.ends_with(".001") {
                                let r = assemble_multipart_archive(parts, aps);
                                if r {
                                    let aar = fnn.strip_suffix(".001").unwrap().to_string();
                                    let far = ap.join(aar).to_str().unwrap().to_string();
                                    let ext = extract_archive(far, install.directory.clone(), false);
                                    if ext {
                                        h4.emit("download_complete", install.name.clone()).unwrap();
                                        prevent_exit(&h4, false);
                                        send_notification(&h4, format!("Download of {inn} complete.", inn = install.name).as_str(), None);
                                    }
                                }
                            } else {
                                let far = ap.join(fnn.clone()).to_str().unwrap().to_string();
                                let ext = extract_archive(far, install.directory.clone(), false);
                                if ext {
                                    h4.emit("download_complete", install.name.clone()).unwrap();
                                    prevent_exit(&h4, false);
                                    send_notification(&h4, format!("Download of {inn} complete.", inn = install.name).as_str(), None);
                                }
                            }
                        }
                    }
                    // Sophon chunk mode, PS: Only hoyo supported as it is their literal format
                    "DOWNLOAD_MODE_CHUNK" => {
                        let urls = picked.game.full.clone();
                        for e in urls.clone() {
                            let h4 = h4.clone();
                            run_async_command(async {
                                <Game as Sophon>::download(e.file_url.clone(), e.file_path.clone(), install.directory.clone(), {
                                    let dlpayload = dlpayload.clone();
                                    let instn = instn.clone();
                                    move |current, total| {
                                        let mut dlp = dlpayload.lock().unwrap();
                                        let instn = instn.clone();
                                        dlp.insert("name", instn.to_string());
                                        dlp.insert("progress", current.to_string());
                                        dlp.insert("total", total.to_string());
                                        h4.emit("download_progress", dlp.clone()).unwrap();
                                        drop(dlp);
                                    }
                                }).await
                            });
                        }
                        // We finished the loop emit complete
                        h4.emit("download_complete", install.name.clone()).unwrap();
                        prevent_exit(&h4, false);
                        send_notification(&h4, format!("Download of {inn} complete.", inn = install.name).as_str(), None);
                    }
                    // KuroGame only currently
                    "DOWNLOAD_MODE_RAW" => {
                        let urls = picked.game.full.iter().map(|v| v.file_url.clone()).collect::<Vec<String>>();
                        let manifest = urls.get(0).unwrap();
                        let rslt = run_async_command(async {
                            <Game as Kuro>::download(manifest.to_owned(), picked.metadata.res_list_url.clone(), install.directory.clone(), {
                                let dlpayload = dlpayload.clone();
                                let h4 = h4.clone();
                                move |current, total| {
                                    let mut dlp = dlpayload.lock().unwrap();
                                    dlp.insert("name", instn.to_string());
                                    dlp.insert("progress", current.to_string());
                                    dlp.insert("total", total.to_string());
                                    h4.emit("download_progress", dlp.clone()).unwrap();
                                    drop(dlp);
                                }
                            }).await
                        });
                        if rslt {
                            h4.emit("download_complete", ()).unwrap();
                            prevent_exit(&h4, false);
                            send_notification(&h4, format!("Download of {inn} complete.", inn = install.name).as_str(), None);
                            #[cfg(target_os = "linux")]
                            {
                                let target = Path::new(&install.directory.clone()).join("Client/Binaries/Win64/ThirdParty/KrPcSdk_Global/KRSDKRes/KRSDK.bin").follow_symlink().unwrap();
                                patch_aki(target.to_str().unwrap().to_string());
                            }
                        }
                    }
                    // Fallback mode... NOT IMPLEMENTED AS I DID NOT WRITE ANY IN THE LIBRARY
                    _ => {}
                }
            } else {
                println!("Failed to download game!");
            }
        });
    });

    // Start game update
    let h5 = app.clone();
    app.listen("start_game_update", move |event| {
        let h5 = h5.clone();
        std::thread::spawn(move || {
            let payload: DownloadGamePayload = serde_json::from_str(event.payload()).unwrap();
            let install = get_install_info_by_id(&h5, payload.install).unwrap(); // Should exist by now, if not we FUCKED UP
            let gid = get_manifest_info_by_id(&h5, install.manifest_id).unwrap();

            let mm = get_manifest(&h5, gid.filename);
            if let Some(gm) = mm {
                let version = gm.game_versions.iter().filter(|e| e.metadata.version == gm.latest_version).collect::<Vec<&GameVersion>>();
                let picked = version.get(0).unwrap();
                let tmp = Arc::new(h5.clone());

                let instn = Arc::new(install.name.clone());
                let dlpayload = Arc::new(Mutex::new(HashMap::new()));

                let mut dlp = dlpayload.lock().unwrap();
                dlp.insert("name", install.name.clone());
                dlp.insert("progress", "0".to_string());
                dlp.insert("total", "1000".to_string());

                h5.emit("update_progress", dlp.clone()).unwrap();
                drop(dlp);
                prevent_exit(&h5, true);

                match picked.metadata.download_mode.as_str() {
                    // Generic zipped mode, Variety per game can not account for every case yet
                    "DOWNLOAD_MODE_FILE" => {
                        h5.emit("update_complete", ()).unwrap();
                        prevent_exit(&h5, false);
                    }
                    // Sophon chunk mode, PS: Only hoyo supported as it is their literal format
                    "DOWNLOAD_MODE_CHUNK" => {
                        let urls = picked.game.diff.iter().filter(|e| e.original_version.as_str() == install.version.clone().as_str()).collect::<Vec<&DiffGameFile>>();

                        if urls.is_empty() {
                            h5.emit("update_complete", ()).unwrap();
                            prevent_exit(&h5, false);
                        } else {
                            let is_preload = Path::new(&install.directory).join("patching").join(".preload").follow_symlink().unwrap().exists();
                            #[cfg(target_os = "linux")]
                            let hpatchz = h5.path().app_data_dir().unwrap().join("hpatchz");
                            #[cfg(target_os = "windows")]
                            let hpatchz = h5.path().app_data_dir().unwrap().join("hpatchz.exe");
                            urls.into_iter().for_each(|e| {
                                run_async_command(async {
                                    <Game as Sophon>::patch(e.file_url.to_owned(), install.version.clone(), e.file_hash.to_owned(), install.directory.clone(), hpatchz.to_str().unwrap().to_string(), is_preload, false, {
                                        let dlpayload = dlpayload.clone();
                                        let tmp = tmp.clone();
                                        let instn = instn.clone();
                                        move |current, total| {
                                            let mut dlp = dlpayload.lock().unwrap();
                                            dlp.insert("name", instn.to_string());
                                            dlp.insert("progress", current.to_string());
                                            dlp.insert("total", total.to_string());
                                            tmp.emit("update_progress", dlp.clone()).unwrap();
                                            drop(dlp);
                                        }
                                    }).await
                                });
                            });
                            if is_preload { let p = Path::new(&install.directory).join("patching").follow_symlink().unwrap(); fs::remove_dir_all(p).unwrap(); }
                            h5.emit("update_complete", ()).unwrap();
                            prevent_exit(&h5, false);
                            send_notification(&h5, format!("Updating {inn} complete.", inn = install.name).as_str(), None);
                            update_install_after_update_by_id(&h5, install.id, picked.metadata.versioned_name.clone(), picked.assets.game_icon.clone(), picked.assets.game_background.clone(), picked.metadata.version.clone());
                        }
                    }
                    // KuroGame only
                    "DOWNLOAD_MODE_RAW" => {
                        let urls = picked.game.diff.iter().filter(|e| e.original_version.as_str() == install.version.clone().as_str()).collect::<Vec<&DiffGameFile>>();
                        if urls.is_empty() {
                            h5.emit("update_complete", ()).unwrap();
                            prevent_exit(&h5, false);
                        } else {
                            let manifest = urls.get(0).unwrap().file_url.clone();
                            let rslt = run_async_command(async {
                                <Game as Kuro>::patch(manifest.to_owned(), install.version.clone(), picked.metadata.res_list_url.clone(), install.directory.clone(), false, {
                                    let dlpayload = dlpayload.clone();
                                    move |current, total| {
                                        let mut dlp = dlpayload.lock().unwrap();
                                        dlp.insert("name", instn.to_string());
                                        dlp.insert("progress", current.to_string());
                                        dlp.insert("total", total.to_string());
                                        tmp.emit("update_progress", dlp.clone()).unwrap();
                                        drop(dlp);
                                    }
                                }).await
                            });
                            if rslt {
                                h5.emit("update_complete", ()).unwrap();
                                prevent_exit(&h5, false);
                                send_notification(&h5, format!("Updating {inn} complete.", inn = install.name).as_str(), None);
                                update_install_after_update_by_id(&h5, install.id, picked.metadata.versioned_name.clone(), picked.assets.game_icon.clone(), picked.assets.game_background.clone(), picked.metadata.version.clone());
                                #[cfg(target_os = "linux")]
                                {
                                    let target = Path::new(&install.directory.clone()).join("Client/Binaries/Win64/ThirdParty/KrPcSdk_Global/KRSDKRes/KRSDK.bin").follow_symlink().unwrap();
                                    patch_aki(target.to_str().unwrap().to_string());
                                }
                            }
                        }
                    }
                    // Fallback mode... NOT IMPLEMENTED AS I DID NOT WRITE ANY IN THE LIBRARY
                    _ => {}
                }
            } else {
                println!("Failed to update game!");
            }
        });
    });

    // Start game repair
    let h5 = app.clone();
    app.listen("start_game_repair", move |event| {
        let h5 = h5.clone();
        std::thread::spawn(move || {
            let payload: DownloadGamePayload = serde_json::from_str(event.payload()).unwrap();
            let install = get_install_info_by_id(&h5, payload.install); // Should exist by now, if not we FUCKED UP
            let lm = get_manifest_info_by_id(&h5, install.clone().unwrap().manifest_id.clone()).unwrap();
            let gm = get_manifest(&h5, lm.filename).unwrap();

            if install.is_some() { 
                let i = install.unwrap();
                let version = gm.game_versions.iter().filter(|e| e.metadata.version == i.version).collect::<Vec<&GameVersion>>();
                let picked = version.get(0).unwrap();

                let tmp = Arc::new(h5.clone());
                let instn = Arc::new(i.name.clone());
                let dlpayload = Arc::new(Mutex::new(HashMap::new()));

                let mut dlp = dlpayload.lock().unwrap();
                dlp.insert("name", i.name.clone());
                dlp.insert("progress", "0".to_string());
                dlp.insert("total", "1000".to_string());

                h5.emit("repair_progress", dlp.clone()).unwrap();
                drop(dlp);
                prevent_exit(&h5, true);

                match picked.metadata.download_mode.as_str() {
                    // General game repair, PS: Only hoyo games for backwards compatibility
                    "DOWNLOAD_MODE_FILE" => {
                        let rslt = <Game as Hoyo>::repair_game(picked.metadata.res_list_url.clone(), i.directory.clone(), i.skip_hash_check, {
                            let dlpayload = dlpayload.clone();
                            move |current, total| {
                                let mut dlp = dlpayload.lock().unwrap();
                                dlp.insert("name", instn.to_string());
                                dlp.insert("progress", current.to_string());
                                dlp.insert("total", total.to_string());
                                tmp.emit("repair_progress", dlp.clone()).unwrap();
                                drop(dlp);
                            }
                        });
                        if rslt {
                            h5.emit("repair_complete", ()).unwrap();
                            prevent_exit(&h5, false);
                            send_notification(&h5, format!("Repair of {inn} complete.", inn = i.name).as_str(), None);
                        };
                    }
                    // Sophon chunk repair, PS: Only hoyo games as it is their literal format
                    "DOWNLOAD_MODE_CHUNK" => {
                        let urls = picked.game.full.clone();
                        urls.into_iter().for_each(|e| {
                            run_async_command(async {
                                <Game as Sophon>::repair_game(e.file_url.clone(), e.file_path.clone(), i.directory.clone(), false, {
                                    let dlpayload = dlpayload.clone();
                                    let instn = instn.clone();
                                    let tmp = tmp.clone();
                                    move |current, total| {
                                        let mut dlp = dlpayload.lock().unwrap();
                                        let instn = instn.clone();
                                        let tmp = tmp.clone();
                                        dlp.insert("name", instn.to_string());
                                        dlp.insert("progress", current.to_string());
                                        dlp.insert("total", total.to_string());
                                        tmp.emit("repair_progress", dlp.clone()).unwrap();
                                        drop(dlp);
                                    }
                                }).await
                            });
                        });
                        // We fnished the loop emit complete
                        h5.emit("repair_complete", ()).unwrap();
                        prevent_exit(&h5, false);
                        send_notification(&h5, format!("Repair of {inn} complete.", inn = i.name).as_str(), None);
                    }
                    // KuroGame only
                    "DOWNLOAD_MODE_RAW" => {
                        let urls = picked.game.full.iter().map(|v| v.file_url.clone()).collect::<Vec<String>>();
                        let manifest = urls.get(0).unwrap();
                        let rslt = run_async_command(async {
                            <Game as Kuro>::repair_game(manifest.to_owned(), picked.metadata.res_list_url.clone(), i.directory.clone(), false, {
                                let dlpayload = dlpayload.clone();
                                move |current, total| {
                                    let mut dlp = dlpayload.lock().unwrap();

                                    dlp.insert("name", instn.to_string());
                                    dlp.insert("progress", current.to_string());
                                    dlp.insert("total", total.to_string());
                                    tmp.emit("repair_progress", dlp.clone()).unwrap();
                                    drop(dlp);
                                }
                            }).await
                        });
                        if rslt {
                            h5.emit("repair_complete", ()).unwrap();
                            prevent_exit(&h5, false);
                            send_notification(&h5, format!("Repair of {inn} complete.", inn = i.name).as_str(), None);
                            #[cfg(target_os = "linux")]
                            {
                                let target = Path::new(&i.directory.clone()).join("Client/Binaries/Win64/ThirdParty/KrPcSdk_Global/KRSDKRes/KRSDK.bin").follow_symlink().unwrap();
                                patch_aki(target.to_str().unwrap().to_string());
                            }
                        }
                    }
                    // Fallback mode
                    _ => {}
                }
            } else { 
                println!("Failed to find installation for repair!");
            }
            
        });
    });

    // Start game preload
    let h5 = app.clone();
    app.listen("start_game_preload", move |event| {
        let h5 = h5.clone();
        std::thread::spawn(move || {
            let payload: DownloadGamePayload = serde_json::from_str(event.payload()).unwrap();
            let install = get_install_info_by_id(&h5, payload.install).unwrap(); // Should exist by now, if not we FUCKED UP
            let gid = get_manifest_info_by_id(&h5, install.manifest_id).unwrap();

            let mm = get_manifest(&h5, gid.filename);
            if let Some(gm) = mm {
                let version = gm.extra.preload;
                if let Some(picked) = version {
                    let tmp = Arc::new(h5.clone());

                    let pmd = picked.metadata.unwrap();
                    let instn = Arc::new(install.name.clone());
                    let dlpayload = Arc::new(Mutex::new(HashMap::new()));

                    let mut dlp = dlpayload.lock().unwrap();
                    dlp.insert("name", install.name.clone());
                    dlp.insert("progress", "0".to_string());
                    dlp.insert("total", "1000".to_string());

                    h5.emit("preload_progress", dlp.clone()).unwrap();
                    drop(dlp);
                    prevent_exit(&h5, true);

                    match pmd.download_mode.as_str() {
                        // Generic zipped mode, Variety per game can not account for every case yet
                        "DOWNLOAD_MODE_FILE" => {
                            h5.emit("preload_complete", ()).unwrap();
                            prevent_exit(&h5, false);
                        }
                        // Sophon chunk mode, PS: Only hoyo supported as it is their literal format
                        "DOWNLOAD_MODE_CHUNK" => {
                            let pg = picked.game.unwrap();
                            let urls = pg.diff.iter().filter(|e| e.original_version.as_str() == install.version.clone().as_str()).collect::<Vec<&DiffGameFile>>();

                            if urls.is_empty() {
                                h5.emit("preload_complete", ()).unwrap();
                                prevent_exit(&h5, false);
                            } else {
                                urls.into_iter().for_each(|e| {
                                    run_async_command(async {
                                        <Game as Sophon>::preload(e.file_url.to_owned(), install.version.clone(), e.file_hash.to_owned(), install.directory.clone(), {
                                            let dlpayload = dlpayload.clone();
                                            let tmp = tmp.clone();
                                            let instn = instn.clone();
                                            move |current, total| {
                                                let mut dlp = dlpayload.lock().unwrap();
                                                let tmp = tmp.clone();
                                                let instn = instn.clone();

                                                dlp.insert("name", instn.to_string());
                                                dlp.insert("progress", current.to_string());
                                                dlp.insert("total", total.to_string());
                                                tmp.emit("preload_progress", dlp.clone()).unwrap();
                                                drop(dlp);
                                            }
                                        }).await
                                    });
                                });
                                h5.emit("preload_complete", ()).unwrap();
                                prevent_exit(&h5, false);
                                send_notification(&h5, format!("Predownload for {inn} complete.", inn = install.name).as_str(), None);
                            }
                        }
                        // KuroGame only
                        "DOWNLOAD_MODE_RAW" => {
                            h5.emit("preload_complete", ()).unwrap();
                            prevent_exit(&h5, false);
                            /*let urls = picked.game.diff.iter().filter(|e| e.original_version.as_str() == install.version.clone().as_str()).collect::<Vec<&DiffGameFile>>();

                            if urls.is_empty() {  } else {
                                let manifest = urls.get(0).unwrap().file_url.clone();
                                let totalsize = urls.iter().map(|x| x.decompressed_size.parse::<u64>().unwrap()).sum::<u64>();
                                run_async_command(async {
                                    <Game as Kuro>::patch(manifest.to_owned(), install.version.clone(), picked.metadata.res_list_url.clone(), install.directory.clone(), {
                                        let dlpayload = dlpayload.clone();
                                        let tc = tracker.clone();
                                        move |current, total| {
                                            let mut dlp = dlpayload.lock().unwrap();
                                            let mut tracker = tc.lock().unwrap();
                                            *tracker += current;

                                            dlp.insert("name", instn.to_string());
                                            dlp.insert("progress", current.to_string());
                                            dlp.insert("total", total.to_string());
                                            tmp.emit("update_progress", dlp.clone()).unwrap();
                                            drop(dlp);
                                        }
                                    }).await;
                                });
                                if *tracker.lock().unwrap() == totalsize {
                                    h5.emit("update_complete", install.name.clone()).unwrap();

                                    let nd = install.directory.clone().replace(install.version.clone().as_str(), picked.metadata.version.as_str());
                                    let np = install.runner_prefix.clone().replace(install.version.clone().as_str(), picked.metadata.version.as_str());
                                    fs::rename(install.directory.clone(), nd.clone()).unwrap();
                                    fs::rename(install.runner_prefix.clone(), np.clone()).unwrap();
                                    update_install_after_update_by_id(&h5, install.id, picked.metadata.versioned_name.clone(), picked.assets.game_icon.clone(), picked.assets.game_background.clone(), picked.metadata.version.clone(), nd, np);
                                }
                            }*/
                        }
                        // Fallback mode... NOT IMPLEMENTED AS I DID NOT WRITE ANY IN THE LIBRARY
                        _ => {}
                    }
                }
            } else {
                println!("Failed to preload game!");
            }
        });
    });
}

pub fn send_notification(app: &AppHandle, body: &str, icon: Option<&str>) {
    if body.is_empty() { return; }
    if icon.is_some() {
        let i = icon.unwrap();
        app.notification().builder().icon(i).title("TwintailLauncher").body(body).show().unwrap();
    } else { app.notification().builder().title("TwintailLauncher").body(body).show().unwrap(); }
}

pub fn prevent_exit(app: &AppHandle, val: bool) {
    let blocks = app.state::<Mutex<ActionBlocks>>();
    let mut state = blocks.lock().unwrap();
    match val {
        true => {
            state.action_exit = true;
            drop(state);
        }
        false => {
            state.action_exit = false;
            drop(state);
        }
    }
}

#[cfg(target_os = "linux")]
pub fn runner_from_runner_version(runner_version: String) -> Option<String> {
    let mut rslt = String::new();

    if runner_version.is_empty() {
        None
    } else {
        if runner_version.contains("vanilla") {
            rslt = "dxvk_vanilla.json".to_string();
        }
        if runner_version.contains("async") {
            rslt = "dxvk_async.json".to_string();
        }
        if runner_version.contains("gplasync") {
            rslt = "dxvk_gplasync.json".to_string();
        }
        if runner_version.contains("wine-vanilla") {
            rslt = "wine_vanilla.json".to_string();
        }
        if runner_version.contains("wine-staging") {
            rslt = "wine_staging.json".to_string();
        }
        if runner_version.contains("wine-staging-tkg") {
            rslt = "wine_staging_tkg.json".to_string();
        }
        if runner_version.contains("wine-vaniglia") {
            rslt = "wine_vaniglia.json".to_string();
        }
        if runner_version.contains("wine-soda") {
            rslt = "wine_soda.json".to_string();
        }
        if runner_version.contains("wine-lutris") {
            rslt = "wine_lutris.json".to_string();
        }
        if runner_version.contains("wine-ge-proton") {
            rslt = "wine_ge_proton.json".to_string();
        }
        if runner_version.contains("wine-caffe") {
            rslt = "wine_caffe.json".to_string();
        }
        if runner_version.contains("proton-ge") {
            rslt = "proton_ge.json".to_string();
        }
        if runner_version.contains("proton-cachyos") {
            rslt = "proton_cachyos.json".to_string();
        }
        if runner_version.contains("proton-umu") {
            rslt = "proton_umu.json".to_string();
        }
        Some(rslt)
    }
}

pub fn get_mi_path_from_game(exe_name: String) -> Option<String> {
    if exe_name.is_empty() { None } else {
        let exe = exe_name.split('/').last().unwrap().to_string();
        match exe.to_ascii_lowercase().as_str() {
            "genshinimpact.exe" => { Some("gimi".parse().unwrap()) },
            "starrail.exe" => { Some("srmi".parse().unwrap()) },
            "zenlesszonezero.exe" => { Some("zzmi".parse().unwrap()) },
            "bh3.exe" => { Some("himi".parse().unwrap()) },
            "client-win64-shipping.exe" => { Some("wwmi".parse().unwrap()) },
            _ => { None }
        }
    }
}

fn dir_size(path: &Path) -> io::Result<u64> {
    let mut size = 0;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_dir() { size += dir_size(&entry.path())?; } else { size += metadata.len(); }
    }
    Ok(size)
}

pub struct ActionBlocks {
    pub action_exit: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddInstallRsp {
    pub success: bool,
    pub install_id: String,
    pub background: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DownloadGamePayload {
    pub install: String,
    pub biz: String,
    pub lang: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DownloadSizesRsp {
    pub game_decompressed_size: String,
    pub free_disk_space: String,
    pub game_decompressed_size_raw: u64,
    pub free_disk_space_raw: u64
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResumeStatesRsp {
    pub downloading: bool,
    pub updating: bool,
    pub preloading: bool,
    pub repairing: bool
}

pub trait PathResolve {
    fn follow_symlink(&self) -> io::Result<std::path::PathBuf>;
}

impl PathResolve for Path {
    fn follow_symlink(&self) -> io::Result<std::path::PathBuf> {
        if self.is_symlink() { self.canonicalize() } else { Ok(self.to_path_buf()) }
    }
}
