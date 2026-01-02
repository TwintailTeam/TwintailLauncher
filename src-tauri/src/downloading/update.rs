use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use fischl::download::game::{Game, Kuro, Sophon};
use fischl::utils::free_space::available;
use tauri::{AppHandle, Emitter, Listener, Manager};
use crate::utils::db_manager::{get_install_info_by_id, get_manifest_info_by_id, update_install_after_update_by_id};
use crate::utils::{empty_dir, prevent_exit, run_async_command, send_notification, PathResolve, models::{DiffGameFile, GameVersion}};
use crate::utils::repo_manager::{get_manifest};
use crate::downloading::DownloadGamePayload;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

#[cfg(target_os = "linux")]
use crate::utils::patch_aki;

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
                let lv = gm.latest_version.clone();
                let version = gm.game_versions.iter().filter(|e| e.metadata.version == lv).collect::<Vec<&GameVersion>>();
                let picked = version.get(0).unwrap();
                let tmp = Arc::new(h5.clone());
                let vn = picked.metadata.versioned_name.clone();
                let vc = picked.metadata.version.clone();
                let ig = picked.assets.game_icon.clone();
                let gb = picked.assets.game_background.clone();
                let gbiz = gm.biz.clone();

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
                        let urls = picked.game.diff.iter().filter(|e| e.original_version.as_str() == install.version.clone().as_str()).collect::<Vec<&DiffGameFile>>();
                        if urls.is_empty() {
                            h5.dialog().message(format!("Could not find update for {inn}!\nRedownload latest version by pressing \"Redownload\" button.", inn = install.name.clone()).as_str()).title("TwintailLauncher")
                                .kind(MessageDialogKind::Info)
                                .buttons(MessageDialogButtons::OkCancelCustom("Redownload".to_string(), "Cancel".to_string()))
                                .show(move |action| {
                                    if action {
                                        let ip = Path::new(&install.directory).follow_symlink().unwrap();
                                        empty_dir(&ip).unwrap();
                                        let mut data = HashMap::new();
                                        data.insert("install", install.id.clone());
                                        data.insert("biz", gbiz);
                                        data.insert("lang", payload.lang);
                                        data.insert("region", install.region_code);
                                        data.insert("is_latest", "1".to_string());
                                        h5.emit("start_game_download", data).unwrap();
                                        update_install_after_update_by_id(&h5, install.id, vn, ig, gb, vc);
                                    } else {
                                        prevent_exit(&h5, false);
                                        h5.emit("update_complete", ()).unwrap();
                                    }
                                });
                        } else {
                            prevent_exit(&h5, false);
                            h5.emit("update_complete", ()).unwrap();
                        }
                    }
                    // HoYoverse sophon chunk mode
                    "DOWNLOAD_MODE_CHUNK" => {
                        let urls = picked.game.diff.iter().filter(|e| e.original_version.as_str() == install.version.clone().as_str()).collect::<Vec<&DiffGameFile>>();
                        if urls.is_empty() {
                            h5.dialog().message(format!("Could not find update for {inn}!\nRedownload latest version by pressing \"Redownload\" button.", inn = install.name.clone()).as_str()).title("TwintailLauncher")
                                .kind(MessageDialogKind::Info)
                                .buttons(MessageDialogButtons::OkCancelCustom("Redownload".to_string(), "Cancel".to_string()))
                                .show(move |action| {
                                    if action {
                                        let ip = Path::new(&install.directory).follow_symlink().unwrap();
                                        empty_dir(&ip).unwrap();
                                        let mut data = HashMap::new();
                                        data.insert("install", install.id.clone());
                                        data.insert("biz", gbiz);
                                        data.insert("lang", payload.lang);
                                        data.insert("region", install.region_code);
                                        data.insert("is_latest", "1".to_string());
                                        h5.emit("start_game_download", data).unwrap();
                                        update_install_after_update_by_id(&h5, install.id, vn, ig, gb, vc);
                                    } else {
                                        prevent_exit(&h5, false);
                                        h5.emit("update_complete", ()).unwrap();
                                    }
                                });
                        } else {
                            let total_size: u64 = urls.clone().into_iter().map(|e| e.decompressed_size.parse::<u64>().unwrap()).sum();
                            let available = available(install.directory.clone());
                            let has_space = if let Some(av) = available { av >= total_size } else { false };
                            if has_space {
                                let is_preload = Path::new(&install.directory).join("patching").join(".preload").follow_symlink().unwrap().exists();
                                #[cfg(target_os = "linux")]
                                let hpatchz = h5.path().app_data_dir().unwrap().join("hpatchz");
                                #[cfg(target_os = "windows")]
                                let hpatchz = h5.path().app_data_dir().unwrap().join("hpatchz.exe");
                                urls.into_iter().for_each(|e| {
                                    run_async_command(async {
                                        <Game as Sophon>::patch(e.file_url.to_owned(), install.version.clone(), e.file_hash.to_owned(), install.directory.clone(), hpatchz.to_str().unwrap().to_string(), is_preload, {
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
                                send_notification(&h5, format!("Updating {inn} complete.", inn = install.name.clone()).as_str(), None);
                                update_install_after_update_by_id(&h5, install.id, picked.metadata.versioned_name.clone(), picked.assets.game_icon.clone(), picked.assets.game_background.clone(), picked.metadata.version.clone());
                            } else {
                                h5.dialog().message(format!("Unable to update {inn} as there is not enough free space, please make sure there is enough free space for the update!", inn = install.name).as_str()).title("TwintailLauncher")
                                    .kind(MessageDialogKind::Warning)
                                    .buttons(MessageDialogButtons::OkCustom("Ok".to_string())).show(move |_action| { prevent_exit(&h5, false); h5.emit("update_complete", ()).unwrap(); });
                            }
                        }
                    }
                    // KuroGame only
                    "DOWNLOAD_MODE_RAW" => {
                        let urls = picked.game.diff.iter().filter(|e| e.original_version.as_str() == install.version.clone().as_str()).collect::<Vec<&DiffGameFile>>();
                        if urls.is_empty() {
                            h5.dialog().message(format!("Could not find update for {inn}!\nRedownload latest version by pressing \"Redownload\" button.", inn = install.name.clone()).as_str()).title("TwintailLauncher")
                                .kind(MessageDialogKind::Info)
                                .buttons(MessageDialogButtons::OkCancelCustom("Redownload".to_string(), "Cancel".to_string()))
                                .show(move |action| {
                                    if action {
                                        let ip = Path::new(&install.directory).follow_symlink().unwrap();
                                        empty_dir(&ip).unwrap();
                                        let mut data = HashMap::new();
                                        data.insert("install", install.id.clone());
                                        data.insert("biz", gbiz);
                                        data.insert("lang", payload.lang);
                                        data.insert("region", install.region_code);
                                        data.insert("is_latest", "1".to_string());
                                        h5.emit("start_game_download", data).unwrap();
                                        update_install_after_update_by_id(&h5, install.id, vn, ig, gb, vc);
                                    } else {
                                        prevent_exit(&h5, false);
                                        h5.emit("update_complete", ()).unwrap();
                                    }
                                });
                        } else {
                            let total_size: u64 = urls.clone().into_iter().map(|e| e.decompressed_size.parse::<u64>().unwrap()).sum();
                            let available = available(install.directory.clone());
                            let has_space = if let Some(av) = available { av >= total_size } else { false };
                            if has_space {
                                let manifest = urls.get(0).unwrap();
                                let is_preload = Path::new(&install.directory).join("patching").join(".preload").follow_symlink().unwrap().exists();
                                let rslt = run_async_command(async {
                                    <Game as Kuro>::patch(manifest.file_url.to_owned(), manifest.file_hash.clone(), picked.metadata.res_list_url.clone(), install.directory.clone(), is_preload, {
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
                            } else {
                                h5.dialog().message(format!("Unable to update {inn} as there is not enough free space, please make sure there is enough free space for the update!", inn = install.name).as_str()).title("TwintailLauncher")
                                    .kind(MessageDialogKind::Warning)
                                    .buttons(MessageDialogButtons::OkCustom("Ok".to_string())).show(move |_action| { prevent_exit(&h5, false); h5.emit("update_complete", ()).unwrap(); });
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