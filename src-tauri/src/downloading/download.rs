use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use fischl::download::game::{Game, Kuro, Sophon, Zipped};
use fischl::utils::{assemble_multipart_archive, extract_archive};
use tauri::{AppHandle, Emitter, Listener};
use crate::utils::db_manager::{get_install_info_by_id, get_manifest_info_by_id};
use crate::utils::{prevent_exit, run_async_command, send_notification, PathResolve, models::{FullGameFile, GameVersion}};
use crate::utils::repo_manager::{get_manifest};
use crate::downloading::DownloadGamePayload;

#[cfg(target_os = "linux")]
use crate::utils::patch_aki;

pub fn register_download_handler(app: &AppHandle) {
    let a = app.clone();
    app.listen("start_game_download", move |event| {
        let h4 = a.clone();
        std::thread::spawn(move || {
            let payload: DownloadGamePayload = serde_json::from_str(event.payload()).unwrap();
            let install = get_install_info_by_id(&h4, payload.install).unwrap(); // Should exist by now, if not we FUCKED UP
            let gid = get_manifest_info_by_id(&h4, install.manifest_id).unwrap();

            let mm = get_manifest(&h4, gid.filename);
            if let Some(gm) = mm {
                let version = if payload.is_latest.is_some() { gm.game_versions.iter().filter(|e| e.metadata.version == gm.latest_version).collect::<Vec<&GameVersion>>() } else { gm.game_versions.iter().filter(|e| e.metadata.version == install.version).collect::<Vec<&GameVersion>>() };
                let picked = version.get(0).unwrap();

                let instn = if payload.is_latest.is_some() { Arc::new(picked.metadata.versioned_name.clone()) } else { Arc::new(install.name.clone()) };
                let inna = instn.clone();
                let dlpayload = Arc::new(Mutex::new(HashMap::new()));

                let mut dlp = dlpayload.lock().unwrap();
                dlp.insert("name", instn.clone().to_string());
                dlp.insert("progress", "0".to_string());
                dlp.insert("total", "1000".to_string());

                h4.emit("download_progress", dlp.clone()).unwrap();
                drop(dlp);
                prevent_exit(&h4, true);

                match picked.metadata.download_mode.as_str() {
                    // Generic zipped mode
                    "DOWNLOAD_MODE_FILE" => {
                        let urls = picked.game.full.iter().map(|v| v.file_url.clone()).collect::<Vec<String>>();
                        let totalsize = picked.game.full.iter().map(|x| x.compressed_size.parse::<u64>().unwrap()).sum::<u64>();
                        let rslt = run_async_command(async {
                            <Game as Zipped>::download(urls.clone(), install.directory.clone(), {
                                let dlpayload = dlpayload.clone();
                                let h4 = h4.clone();
                                move |current, _| {
                                    let mut dlp = dlpayload.lock().unwrap();
                                    dlp.insert("name", instn.clone().to_string());
                                    dlp.insert("progress", current.to_string());
                                    dlp.insert("total", totalsize.to_string());
                                    h4.emit("download_progress", dlp.clone()).unwrap();
                                    drop(dlp);
                                }
                            }).await
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
                                        h4.emit("download_complete", ()).unwrap();
                                        prevent_exit(&h4, false);
                                        send_notification(&h4, format!("Download of {inn} complete.", inn = inna.to_string()).as_str(), None);
                                    }
                                }
                            } else {
                                let far = ap.join(fnn.clone()).to_str().unwrap().to_string();
                                let ext = extract_archive(far, install.directory.clone(), false);
                                if ext {
                                    h4.emit("download_complete", ()).unwrap();
                                    prevent_exit(&h4, false);
                                    send_notification(&h4, format!("Download of {inn} complete.", inn = inna.to_string()).as_str(), None);
                                }
                            }
                        }
                    }
                    // HoYoverse sophon chunk mode
                    "DOWNLOAD_MODE_CHUNK" => {
                        let urls = if payload.biz == "bh3_global" { picked.game.full.clone().iter().filter(|e| e.region_code.clone().unwrap() == payload.region).cloned().collect::<Vec<FullGameFile>>() } else { picked.game.full.clone()};
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
                        h4.emit("download_complete", ()).unwrap();
                        prevent_exit(&h4, false);
                        send_notification(&h4, format!("Download of {inn} complete.", inn = inna.to_string()).as_str(), None);
                    }
                    // KuroGame only
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
                            send_notification(&h4, format!("Download of {inn} complete.", inn = inna.to_string()).as_str(), None);
                            #[cfg(target_os = "linux")]
                            {
                                let target = Path::new(&install.directory.clone()).join("Client/Binaries/Win64/ThirdParty/KrPcSdk_Global/KRSDKRes/KRSDK.bin").follow_symlink().unwrap();
                                patch_aki(target.to_str().unwrap().to_string());
                            }
                        }
                    }
                    // Fallback mode
                    _ => {}
                }
            } else { eprintln!("Failed to download game!"); }
        });
    });
}