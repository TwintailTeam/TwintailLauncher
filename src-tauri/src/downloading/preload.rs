use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use fischl::download::game::{Game, Kuro, Sophon};
use fischl::utils::free_space::available;
use tauri::{AppHandle, Emitter, Listener};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use crate::utils::db_manager::{get_install_info_by_id, get_manifest_info_by_id};
use crate::utils::{prevent_exit, run_async_command, send_notification, models::{DiffGameFile}};
use crate::utils::repo_manager::{get_manifest};
use crate::downloading::DownloadGamePayload;

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
                        // HoYoverse sophon chunk mode
                        "DOWNLOAD_MODE_CHUNK" => {
                            let pg = picked.game.unwrap();
                            let urls = pg.diff.iter().filter(|e| e.original_version.as_str() == install.version.clone().as_str()).collect::<Vec<&DiffGameFile>>();

                            if urls.is_empty() {
                                h5.emit("preload_complete", ()).unwrap();
                                prevent_exit(&h5, false);
                            } else {
                                let total_size: u64 = urls.clone().into_iter().map(|e| e.decompressed_size.parse::<u64>().unwrap()).sum();
                                let available = available(install.directory.clone());
                                let has_space = if let Some(av) = available { av >= total_size } else { false };
                                if has_space {
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
                                } else {
                                    h5.dialog().message(format!("Unable to predownload update for {inn} as there is not enough free space, please make sure there is enough free space for predownload!", inn = install.name).as_str()).title("TwintailLauncher")
                                        .kind(MessageDialogKind::Warning)
                                        .buttons(MessageDialogButtons::OkCustom("Ok".to_string())).show(move |_action| { prevent_exit(&h5, false); h5.emit("preload_complete", ()).unwrap(); });
                                }
                            }
                        }
                        // KuroGame only
                        "DOWNLOAD_MODE_RAW" => {
                            let pg = picked.game.unwrap();
                            let urls = pg.diff.iter().filter(|e| e.original_version.as_str() == install.version.clone().as_str()).collect::<Vec<&DiffGameFile>>();
                            let manifest = urls.get(0).unwrap();

                            if urls.is_empty() {
                                h5.emit("preload_complete", ()).unwrap();
                                prevent_exit(&h5, false);
                            } else {
                                let total_size: u64 = urls.clone().into_iter().map(|e| e.decompressed_size.parse::<u64>().unwrap()).sum();
                                let available = available(install.directory.clone());
                                let has_space = if let Some(av) = available { av >= total_size } else { false };
                                if has_space {
                                    let rslt = run_async_command(async {
                                        <Game as Kuro>::preload(manifest.file_url.clone(), manifest.file_hash.clone(), pmd.res_list_url.clone(), install.directory.clone(), {
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
                                    if rslt {
                                        h5.emit("preload_complete", ()).unwrap();
                                        prevent_exit(&h5, false);
                                        send_notification(&h5, format!("Predownload for {inn} complete.", inn = instn).as_str(), None);
                                    }
                                } else {
                                    h5.dialog().message(format!("Unable to predownload update for {inn} as there is not enough free space, please make sure there is enough free space for the update!", inn = install.name).as_str()).title("TwintailLauncher")
                                        .kind(MessageDialogKind::Warning)
                                        .buttons(MessageDialogButtons::OkCustom("Ok".to_string())).show(move |_action| { prevent_exit(&h5, false); h5.emit("preload_complete", ()).unwrap(); });
                                }
                            }
                        }
                        // Fallback mode
                        _ => {}
                    }
                }
            } else { eprintln!("Failed to preload game!"); }
        });
    });
}