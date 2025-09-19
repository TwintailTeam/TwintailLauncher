use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use fischl::download::game::{Game, Kuro, Sophon};
use tauri::{AppHandle, Emitter, Listener, Manager};
use crate::utils::db_manager::{get_install_info_by_id, get_manifest_info_by_id, update_install_after_update_by_id};
use crate::utils::{prevent_exit, run_async_command, send_notification, PathResolve};
use crate::utils::repo_manager::{get_manifest, DiffGameFile, GameVersion};
use crate::downloading::DownloadGamePayload;

#[cfg(target_os = "linux")]
use fischl::utils::patch_aki;

pub fn register_update_handler(app: &AppHandle) {
    let a = app.clone();
    app.listen("start_game_update", move |event| {
        let h5 = a.clone();
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
                    // Generic zipped mode, Variety per game
                    "DOWNLOAD_MODE_FILE" => {
                        h5.emit("update_complete", ()).unwrap();
                        prevent_exit(&h5, false);
                    }
                    // HoYoverse sophon chunk mode
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
                            // We finished the loop emit complete
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
                            let manifest = urls.get(0).unwrap();
                            #[cfg(target_os = "linux")]
                            let krpatchz = h5.path().app_data_dir().unwrap().join("krpatchz");
                            #[cfg(target_os = "windows")]
                            let krpatchz = h5.path().app_data_dir().unwrap().join("krpatchz.exe");
                            let rslt = run_async_command(async {
                                <Game as Kuro>::patch(manifest.file_url.to_owned(), install.version.clone(), manifest.file_hash.to_owned(), install.directory.clone(), krpatchz.to_str().unwrap().to_string(), false, {
                                    let dlpayload = dlpayload.clone();
                                    move |current: u64, total: u64| {
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
                    // Fallback mode
                    _ => {}
                }
            } else { eprintln!("Failed to update game!"); }
        });
    });
}