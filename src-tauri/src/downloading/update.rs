use crate::DownloadState;
use crate::downloading::queue::{QueueJobKind, QueueJobOutcome};
use crate::downloading::{DownloadGamePayload, QueueJobPayload};
use crate::utils::db_manager::{
    get_install_info_by_id, get_manifest_info_by_id, update_install_after_update_by_id,
};
use crate::utils::repo_manager::get_manifest;
use crate::utils::{empty_dir, models::{DiffGameFile, GameVersion}, run_async_command, show_dialog};
use fischl::download::game::{Game, Kuro, Sophon};
use fischl::utils::free_space::available;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool,AtomicU64,Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Listener, Manager};

pub fn register_update_handler(app: &AppHandle) {
    let a = app.clone();
    app.listen("start_game_update", move |event| {
        let payload: DownloadGamePayload = serde_json::from_str(event.payload()).unwrap();
        let state = a.state::<DownloadState>();
        let q = state.queue.lock().unwrap().clone();
        if let Some(queue) = q {
            queue.enqueue(QueueJobKind::GameUpdate, QueueJobPayload::Game(payload));
        } else {
            let h5 = a.clone();
            std::thread::spawn(move || {
                let job_id = format!("direct_update_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis());
                let _ = run_game_update(h5, payload, job_id);
            });
        }
    });
}

pub fn run_game_update(h5: AppHandle, payload: DownloadGamePayload, job_id: String) -> QueueJobOutcome {
    let job_id = Arc::new(job_id);
    let install_id = payload.install.clone();
    let install = match get_install_info_by_id(&h5, payload.install) {
        Some(v) => v,
        None => return QueueJobOutcome::Failed,
    };
    let gid = match get_manifest_info_by_id(&h5, install.manifest_id.clone()) {
        Some(v) => v,
        None => return QueueJobOutcome::Failed,
    };

    let mm = get_manifest(&h5, gid.filename);
    if let Some(gm) = mm {
        let lv = gm.latest_version.clone();
        let version = gm.game_versions.iter().filter(|e| e.metadata.version == lv).collect::<Vec<&GameVersion>>();
        let picked = match version.get(0) {
            Some(v) => *v,
            None => return QueueJobOutcome::Failed,
        };
        let tmp = Arc::new(h5.clone());
        let vn = picked.metadata.versioned_name.clone();
        let vc = picked.metadata.version.clone();
        let ig = picked.assets.game_icon.clone();
        #[cfg(target_os = "linux")]
        let gb = picked.assets.game_background.clone();
        #[cfg(not(target_os = "linux"))]
        let gb = if install.game_background.ends_with(".webm") || install.game_background.ends_with(".mp4") { if let Some(ref lbg) = picked.assets.game_live_background { if !lbg.is_empty() { lbg.clone() } else { picked.assets.game_background.clone() } } else { picked.assets.game_background.clone() } } else { picked.assets.game_background.clone() };
        let gbiz = gm.biz.clone();

        let instn = Arc::new(install.name.clone());
        let dlpayload = Arc::new(Mutex::new(HashMap::new()));

        let mut dlp = dlpayload.lock().unwrap();
        dlp.insert("job_id", job_id.to_string());
        dlp.insert("name", install.name.clone());
        dlp.insert("progress", "0".to_string());
        dlp.insert("total", "1000".to_string());
        h5.emit("update_progress", dlp.clone()).unwrap();
        drop(dlp);

        let cancel_token = Arc::new(AtomicBool::new(false));
        {
            let state = h5.state::<DownloadState>();
            let mut tokens = state.tokens.lock().unwrap();
            tokens.insert(install_id.clone(), cancel_token.clone());
        }

        let verified_files = { let state = h5.state::<DownloadState>(); let mut vf = state.verified_files.lock().unwrap(); vf.entry(install_id.clone()).or_insert_with(|| Arc::new(Mutex::new(std::collections::HashSet::new()))).clone() };
        let mut success = false;
        match picked.metadata.download_mode.as_str() {
            "DOWNLOAD_MODE_FILE" => {
                let urls = picked.game.diff.iter().filter(|e| e.original_version.as_str() == install.version.clone().as_str()).collect::<Vec<&DiffGameFile>>();
                if urls.is_empty() {
                    log::debug!("No diff files found for this update using DOWNLOAD_MODE_FILE, treating as full download");
                    let h5_clone = h5.clone();
                    let payload_lang = payload.lang.clone();

                    empty_dir(&install.directory).unwrap_or_default();
                    let mut data = HashMap::new();
                    data.insert("install", install.id.clone());
                    data.insert("biz", gbiz.clone());
                    data.insert("lang", payload_lang.clone());
                    data.insert("region", install.region_code.clone());
                    data.insert("is_latest", "1".to_string());
                    h5_clone.emit("start_game_download", data).unwrap();
                    update_install_after_update_by_id(&h5_clone, install.id.clone(), vn.clone(), ig.clone(), gb.clone(), vc.clone());
                    h5.emit("update_complete", ()).unwrap();
                    #[cfg(target_os = "linux")]
                    crate::utils::shortcuts::sync_desktop_shortcut(&h5, install.id.clone(), picked.metadata.versioned_name.clone());
                } else {
                    h5.emit("update_complete", ()).unwrap();
                    log::warn!("Diff files found for this update using DOWNLOAD_MODE_FILE, but this mode does not support patching, marking as complete");
                }
                success = true;
            }
            "DOWNLOAD_MODE_CHUNK" => {
                let urls = picked.game.diff.iter().filter(|e| e.original_version.as_str() == install.version.clone().as_str()).cloned().collect::<Vec<DiffGameFile>>();
                if urls.is_empty() {
                    log::debug!("No diff files found for this update using DOWNLOAD_MODE_CHUNK, treating as full download");
                    let h5_clone = h5.clone();
                    let payload_lang = payload.lang.clone();

                    empty_dir(&install.directory).unwrap_or_default();
                    let mut data = HashMap::new();
                    data.insert("install", install.id.clone());
                    data.insert("biz", gbiz.clone());
                    data.insert("lang", payload_lang.clone());
                    data.insert("region", install.region_code.clone());
                    data.insert("is_latest", "1".to_string());
                    h5_clone.emit("start_game_download", data).unwrap();
                    update_install_after_update_by_id(&h5_clone, install.id.clone(), vn.clone(), ig.clone(), gb.clone(), vc.clone());
                    h5.emit("update_complete", ()).unwrap();
                    #[cfg(target_os = "linux")]
                    crate::utils::shortcuts::sync_desktop_shortcut(&h5, install.id.clone(), picked.metadata.versioned_name.clone());
                    success = true;
                } else {
                    let total_size: u64 = urls.iter().map(|e| e.compressed_size.parse::<u64>().unwrap_or(0)).sum();
                    let available = available(install.directory.clone());
                    let has_space = if let Some(av) = available { av >= total_size } else { false };
                    if has_space {
                        log::debug!("Starting update of {} using DOWNLOAD_MODE_CHUNK, total size: {}, available space: {:?}", install.name, total_size, available);
                        let patching_marker = Path::new(&install.directory).join("patching");
                        let is_preload = patching_marker.join(".preload").exists();
                        let combined_download_total = total_size;
                        let combined_install_total: u64 = urls.iter().map(|e| e.decompressed_size.parse::<u64>().unwrap_or(0)).sum();
                        let cumulative_download = Arc::new(AtomicU64::new(0));
                        let cumulative_install = Arc::new(AtomicU64::new(0));
                        let total_manifests = urls.len();
                        let mut ok = true;
                        for (manifest_idx, e) in urls.clone().into_iter().enumerate() {
                            let compressed = e.compressed_size.parse::<u64>().unwrap_or(0);
                            let decompressed = e.decompressed_size.parse::<u64>().unwrap_or(0);
                            let h5 = h5.clone();
                            let cancel_token = cancel_token.clone();
                            let cumulative_download = cumulative_download.clone();
                            let cumulative_install = cumulative_install.clone();
                            let is_last_manifest = manifest_idx == total_manifests - 1;
                            let rslt = run_async_command(async {
                                <Game as Sophon>::patch(e.file_url.clone(), install.version.clone(), e.file_path.clone(), install.directory.clone(), is_preload, {
                                        let dlpayload = dlpayload.clone();
                                        let instn = instn.clone();
                                        let job_id = job_id.clone();
                                        let cumulative_download = cumulative_download.clone();
                                        let cumulative_install = cumulative_install.clone();
                                        move |download_current, _download_total, install_current, _install_total, net_speed, disk_speed, phase| {
                                            let mut dlp = dlpayload.lock().unwrap();
                                            let total_download_progress = cumulative_download.load(Ordering::SeqCst) + download_current;
                                            let total_install_progress = cumulative_install.load(Ordering::SeqCst) + install_current;
                                            dlp.insert("job_id", job_id.to_string());
                                            dlp.insert("name", instn.to_string());
                                            if is_preload { dlp.insert("progress", total_install_progress.to_string()); dlp.insert("total", combined_install_total.to_string()); dlp.insert("speed", disk_speed.to_string()); } else { dlp.insert("progress", total_download_progress.to_string()); dlp.insert("total", combined_download_total.to_string()); dlp.insert("speed", net_speed.to_string()); }
                                            dlp.insert("disk", disk_speed.to_string());
                                            dlp.insert("install_progress", total_install_progress.to_string());
                                            dlp.insert("install_total", combined_install_total.to_string());
                                            let effective_phase = if phase == 5 && !is_last_manifest { 2 } else { phase };
                                            dlp.insert("phase", effective_phase.to_string());
                                            h5.emit("update_progress", dlp.clone()).unwrap();
                                            drop(dlp);
                                        }
                                    }, Some(cancel_token), Some(verified_files.clone())).await
                            });
                            if !rslt { ok = false; break; }
                            cumulative_download.fetch_add(compressed, Ordering::SeqCst);
                            cumulative_install.fetch_add(decompressed, Ordering::SeqCst);
                        }
                        if ok {
                            if patching_marker.exists() { fs::remove_dir_all(&patching_marker).unwrap_or_default(); }
                            update_install_after_update_by_id(&h5, install.id.clone(), picked.metadata.versioned_name.clone(), picked.assets.game_icon.clone(), gb.clone(), picked.metadata.version.clone());
                            h5.emit("update_complete", ()).unwrap();
                            log::debug!("Successfully updated {} using DOWNLOAD_MODE_CHUNK, marking as complete", install.name);
                            #[cfg(target_os = "linux")]
                            crate::utils::shortcuts::sync_desktop_shortcut(&h5, install.id.clone(), picked.metadata.versioned_name.clone());
                            success = true;
                        } else {
                            if !cancel_token.load(Ordering::Relaxed) { show_dialog(&h5, "warning", "TwintailLauncher", &format!("Error occurred while trying to update {}\nPlease try again!", install.name), Some(vec!["Ok"])); }
                            h5.emit("update_complete", ()).unwrap();
                            log::debug!("Error occurred during update of {} using DOWNLOAD_MODE_CHUNK, marking as failed", install.name);
                        }
                    } else {
                        show_dialog(&h5, "warning", "TwintailLauncher", &format!("Unable to update {} as there is not enough free space, please make sure there is enough free space for the update!", install.name), Some(vec!["Ok"]));
                        h5.emit("update_complete", ()).unwrap();
                        log::debug!("Not enough space to update {} using DOWNLOAD_MODE_CHUNK, required: {}, available: {:?}", install.name, total_size, available);
                    }
                }
            }
            "DOWNLOAD_MODE_RAW" => {
                let urls = picked.game.diff.iter().filter(|e| e.original_version.as_str() == install.version.clone().as_str()).collect::<Vec<&DiffGameFile>>();
                if urls.is_empty() {
                    let h5_clone = h5.clone();
                    let payload_lang = payload.lang.clone();

                    empty_dir(&install.directory).unwrap_or_default();
                    let mut data = HashMap::new();
                    data.insert("install", install.id.clone());
                    data.insert("biz", gbiz.clone());
                    data.insert("lang", payload_lang.clone());
                    data.insert("region", install.region_code.clone());
                    data.insert("is_latest", "1".to_string());
                    h5_clone.emit("start_game_download", data).unwrap();
                    update_install_after_update_by_id(&h5_clone, install.id.clone(), vn.clone(), ig.clone(), gb.clone(), vc.clone());
                    h5.emit("update_complete", ()).unwrap();
                    #[cfg(target_os = "linux")]
                    {
                        crate::utils::shortcuts::sync_desktop_shortcut(&h5, install.id.clone(), picked.metadata.versioned_name.clone());
                        crate::utils::apply_patch(&h5, Path::new(&install.directory.clone()).to_str().unwrap().to_string(), "aki".to_string(), "add".to_string());
                    }
                    success = true;
                } else {
                    let total_size: u64 = urls.clone().into_iter().map(|e| e.decompressed_size.parse::<u64>().unwrap()).sum();
                    let available = available(install.directory.clone());
                    let has_space = if let Some(av) = available { av >= total_size } else { false };
                    if has_space {
                        let manifest = urls.get(0).unwrap();
                        let patching_marker = Path::new(&install.directory).join("patching");
                        let is_preload = patching_marker.join(".preload").exists();
                        let cancel_token = cancel_token.clone();
                        let rslt = run_async_command(async {
                            <Game as Kuro>::patch(manifest.file_url.to_owned(), manifest.file_path.clone(), picked.metadata.res_list_url.clone(), install.directory.clone(), is_preload, {
                                    let dlpayload = dlpayload.clone();
                                    let job_id = job_id.clone();
                                    move |download_current: u64, download_total: u64, install_current: u64, install_total: u64, net_speed: u64, disk_speed: u64, phase: u8| {
                                        let mut dlp = dlpayload.lock().unwrap();
                                        dlp.insert("job_id", job_id.to_string());
                                        dlp.insert("name", instn.to_string());
                                        dlp.insert("progress", download_current.to_string());
                                        dlp.insert("total", download_total.to_string());
                                        dlp.insert("speed", net_speed.to_string());
                                        dlp.insert("disk", disk_speed.to_string());
                                        dlp.insert("install_progress", install_current.to_string());
                                        dlp.insert("install_total", install_total.to_string());
                                        // Phase: 0=idle, 1=verifying, 2=downloading, 3=installing, 4=validating, 5=moving
                                        dlp.insert("phase", phase.to_string());
                                        tmp.emit("update_progress", dlp.clone()).unwrap();
                                        drop(dlp);
                                    }
                                }, Some(cancel_token.clone()), Some(verified_files.clone())).await
                        });
                        if rslt {
                            // Remove patching marker on success
                            if patching_marker.exists() { fs::remove_dir_all(&patching_marker).unwrap_or_default(); }
                            update_install_after_update_by_id(&h5, install.id.clone(), picked.metadata.versioned_name.clone(), picked.assets.game_icon.clone(), gb.clone(), picked.metadata.version.clone());
                            h5.emit("update_complete", ()).unwrap();
                            #[cfg(target_os = "linux")]
                            {
                                crate::utils::shortcuts::sync_desktop_shortcut(&h5, install.id.clone(), picked.metadata.versioned_name.clone());
                                crate::utils::apply_patch(&h5, Path::new(&install.directory.clone()).to_str().unwrap().to_string(), "aki".to_string(), "add".to_string());
                            }
                            success = true;
                        } else {
                            if !cancel_token.load(Ordering::Relaxed) { show_dialog(&h5, "warning", "TwintailLauncher", &format!("Error occurred while trying to update {}\nPlease try again!", install.name), Some(vec!["Ok"])); }
                            h5.emit("update_complete", ()).unwrap();
                        }
                    } else {
                        show_dialog(&h5, "warning", "TwintailLauncher", &format!("Unable to update {} as there is not enough free space, please make sure there is enough free space for the update!", install.name), Some(vec!["Ok"]));
                        h5.emit("update_complete", ()).unwrap();
                    }
                }
            }
            _ => { log::debug!("We should not be here... HOW IN THE ABSOLUTE HELL DID WE GET HERE? DOWNLOAD_MODE_???"); show_dialog(&h5, "error", "TwintailLauncher", "Unsupported download mode for update!", Some(vec!["Ok"])); }
        }

        let mut cancelled = false;
        {
            let state = h5.state::<DownloadState>();
            let tokens = state.tokens.lock().unwrap();
            if let Some(token) = tokens.get(&install_id) { if token.load(Ordering::Relaxed) { cancelled = true; } }
        }
        {
            let state = h5.state::<DownloadState>();
            let mut tokens = state.tokens.lock().unwrap();
            tokens.remove(&install_id);
        }
        if cancelled {
            let mut dlp = HashMap::new();
            dlp.insert("job_id", job_id.to_string());
            dlp.insert("name", install.name.clone());
            h5.emit("update_paused", dlp).unwrap();
            return QueueJobOutcome::Cancelled;
        }
        if success {
            { verified_files.lock().unwrap().clear(); }
            QueueJobOutcome::Completed
        } else {
            { verified_files.lock().unwrap().clear(); }
            QueueJobOutcome::Failed
        }
    } else {
        log::debug!("Failed to update game, wtf??? we are SO FUCKED!");
        QueueJobOutcome::Failed
    }
}
