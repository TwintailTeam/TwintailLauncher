use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use fischl::download::game::{Game, Sophon};
use tauri::{AppHandle, Emitter, Listener};
use crate::utils::db_manager::{get_install_info_by_id, get_manifest_info_by_id};
use crate::utils::{prevent_exit, run_async_command, send_notification, DownloadGamePayload};
use crate::utils::repo_manager::{get_manifest, DiffGameFile};

pub fn register_preload_handler(app: &AppHandle) {
    let a = app.clone();
    app.listen("start_game_preload", move |event| {
        let h5 = a.clone();
        std::thread::spawn(move || {
            let payload: DownloadGamePayload = serde_json::from_str(event.payload()).unwrap();
            let install = get_install_info_by_id(&h5, payload.install).unwrap();
            let gid = get_manifest_info_by_id(&h5, install.manifest_id).unwrap();

            let mm = get_manifest(&h5, gid.filename);
            if let Some(gm) = mm {
                let version = gm.extra.preload;
                if let Some(picked) = version {
                    let tmp = Arc::new(h5.clone());

                    let pmd = picked.metadata.unwrap();
                    let instn = Arc::new(install.name.replace(install.version.as_str(), pmd.version.as_str()).clone());
                    let dlpayload = Arc::new(Mutex::new(HashMap::new()));

                    let mut dlp = dlpayload.lock().unwrap();
                    dlp.insert("name", instn.to_string());
                    dlp.insert("progress", "0".to_string());
                    dlp.insert("total", "1000".to_string());

                    h5.emit("preload_progress", dlp.clone()).unwrap();
                    drop(dlp);
                    prevent_exit(&h5, true);

                    match pmd.download_mode.as_str() {
                        // Generic zipped mode, Variety per game
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
                                send_notification(&h5, format!("Predownload for {inn} complete.", inn = instn).as_str(), None);
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
                        // Fallback mode
                        _ => {}
                    }
                }
            } else { eprintln!("Failed to preload game!"); }
        });
    });
}