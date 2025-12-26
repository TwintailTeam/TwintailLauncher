use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use fischl::download::game::{Game, Kuro, Sophon};
use tauri::{AppHandle, Emitter, Listener};
use crate::utils::db_manager::{get_install_info_by_id, get_manifest_info_by_id};
use crate::utils::{prevent_exit, run_async_command, send_notification, models::{FullGameFile, GameVersion}};
use crate::utils::repo_manager::{get_manifest};
use crate::downloading::DownloadGamePayload;

#[cfg(target_os = "linux")]
use crate::utils::{PathResolve, patch_aki, empty_dir};

pub fn register_repair_handler(app: &AppHandle) {
    let a = app.clone();
    app.listen("start_game_repair", move |event| {
        let h5 = a.clone();
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

                #[cfg(target_os = "linux")]
                {
                    // Set prefix in repair state by emptying the directory
                    let prefix_path = std::path::Path::new(&i.runner_prefix).follow_symlink().unwrap();
                    if prefix_path.exists() && !gm.extra.compat_overrides.install_to_prefix { empty_dir(prefix_path).unwrap(); }
                }

                match picked.metadata.download_mode.as_str() {
                    // Generic zipped mode, Variety per game
                    "DOWNLOAD_MODE_FILE" => {
                        h5.emit("repair_complete", ()).unwrap();
                        prevent_exit(&h5, false);
                    }
                    // HoYoverse sophon chunk mode
                    "DOWNLOAD_MODE_CHUNK" => {
                        let urls = if payload.biz == "bh3_global" { picked.game.full.clone().iter().filter(|e| e.region_code.clone().unwrap() == payload.region).cloned().collect::<Vec<FullGameFile>>() } else { picked.game.full.clone()};
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
                        // We finished the loop emit complete
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
                                let target = std::path::Path::new(&i.directory.clone()).join("Client/Binaries/Win64/ThirdParty/KrPcSdk_Global/KRSDKRes/KRSDK.bin").follow_symlink().unwrap();
                                patch_aki(target.to_str().unwrap().to_string());
                            }
                        }
                    }
                    // Fallback mode
                    _ => {}
                }
            } else { eprintln!("Failed to find installation for repair!"); }
        });
    });
}