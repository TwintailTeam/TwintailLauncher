use crate::DownloadState;
use crate::downloading::queue::{QueueJobKind, QueueJobOutcome};
use crate::downloading::{DownloadGamePayload, QueueJobPayload};
use crate::utils::db_manager::{get_install_info_by_id, get_manifest_info_by_id};
use crate::utils::repo_manager::get_manifest;
use crate::utils::{models::{FullGameFile, GameVersion}, run_async_command, show_dialog};
use fischl::download::game::{Game, Kuro, Sophon, Zipped};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool,Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Listener, Manager};

pub fn register_repair_handler(app: &AppHandle) {
    let a = app.clone();
    app.listen("start_game_repair", move |event| {
        let payload: DownloadGamePayload = serde_json::from_str(event.payload()).unwrap();
        let state = a.state::<DownloadState>();
        let q = state.queue.lock().unwrap().clone();
        if let Some(queue) = q {
            if queue.has_job_for_id(payload.install.clone()) { show_dialog(&a, "warning", "TwintailLauncher", "This game is already queued for repair!", None); return; }
            queue.enqueue(QueueJobKind::GameRepair, QueueJobPayload::Game(payload));
        } else {
            let h5 = a.clone();
            std::thread::spawn(move || {
                let job_id = format!("direct_repair_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis());
                let _ = run_game_repair(h5, payload, job_id);
            });
        }
    });
}

pub fn run_game_repair(h5: AppHandle, payload: DownloadGamePayload, job_id: String) -> QueueJobOutcome {
    let job_id = Arc::new(job_id);
    let install_id = payload.install.clone();
    let install = get_install_info_by_id(&h5, payload.install.clone());
    if install.is_none() { eprintln!("Failed to find installation for repair!");return QueueJobOutcome::Failed; }

    let i = install.unwrap();
    let lm = match get_manifest_info_by_id(&h5, i.manifest_id.clone()) {
        Some(v) => v,
        None => return QueueJobOutcome::Failed,
    };
    let gm = match get_manifest(&h5, lm.filename) {
        Some(v) => v,
        None => return QueueJobOutcome::Failed,
    };

    let version = gm.game_versions.iter().filter(|e| e.metadata.version == i.version).collect::<Vec<&GameVersion>>();
    let picked = match version.get(0) {
        Some(v) => *v,
        None => return QueueJobOutcome::Failed,
    };

    let tmp = Arc::new(h5.clone());
    let instn = Arc::new(i.name.clone());
    let dlpayload = Arc::new(Mutex::new(HashMap::new()));

    let mut dlp = dlpayload.lock().unwrap();
    dlp.insert("job_id", job_id.to_string());
    dlp.insert("name", i.name.clone());
    dlp.insert("progress", "0".to_string());
    dlp.insert("total", "1000".to_string());
    h5.emit("repair_progress", dlp.clone()).unwrap();
    drop(dlp);

    let cancel_token = Arc::new(AtomicBool::new(false));
    {
        let state = h5.state::<DownloadState>();
        let mut tokens = state.tokens.lock().unwrap();
        tokens.insert(install_id.clone(), cancel_token.clone());
    }

    let verified_files = {
        let state = h5.state::<DownloadState>();
        let mut vf = state.verified_files.lock().unwrap();
        vf.entry(payload.install.clone()).or_insert_with(|| Arc::new(Mutex::new(std::collections::HashSet::new()))).clone()
    };

    let mut success = false;
    match picked.metadata.download_mode.as_str() {
        "DOWNLOAD_MODE_FILE" => {
            let install_dir = std::path::Path::new(&i.directory);
            if !install_dir.exists() { std::fs::create_dir_all(install_dir).unwrap_or_default(); }

            log::debug!("Starting game repair using DOWNLOAD_MODE_FILE with {} file(s)", picked.game.full.len());
            let files = picked.game.full.clone();
            let urls = files.iter().map(|v| v.file_url.clone()).collect::<Vec<String>>();
            let combined_download_total: u64 = files.iter().map(|e| e.compressed_size.parse::<u64>().unwrap_or(0)).sum();
            let combined_install_total: u64 = files.iter().map(|e| e.decompressed_size.parse::<u64>().unwrap_or(0)).sum();
            let cumulative_download = Arc::new(std::sync::atomic::AtomicU64::new(0));
            let mut ok = true;
            for e in files.iter() {
                let url = e.file_url.clone();
                let cancel_token = cancel_token.clone();
                let dl_ok = run_async_command(async {
                    <Game as Zipped>::download(url.clone(), i.directory.clone(), false, true, {
                            let dlpayload = dlpayload.clone();
                            let h5 = h5.clone();
                            let instn = instn.clone();
                            let job_id = job_id.clone();
                            let cumulative_download = cumulative_download.clone();
                            move |current, _total, net_speed, disk_speed| {
                                let mut dlp = dlpayload.lock().unwrap();
                                let total_dl_progress = cumulative_download.load(Ordering::SeqCst) + current;
                                dlp.insert("job_id", job_id.to_string());
                                dlp.insert("name", instn.to_string());
                                dlp.insert("progress", total_dl_progress.to_string());
                                dlp.insert("total", combined_download_total.to_string());
                                dlp.insert("speed", net_speed.to_string());
                                dlp.insert("disk", disk_speed.to_string());
                                dlp.insert("install_progress", "0".to_string());
                                dlp.insert("install_total", combined_install_total.to_string());
                                dlp.insert("phase", "2".to_string());
                                h5.emit("repair_progress", dlp.clone()).unwrap();
                                drop(dlp);
                            }
                        }, Some(cancel_token.clone()), Some(verified_files.clone())).await
                });
                if !dl_ok { ok = false; break; }
                cumulative_download.fetch_add(e.compressed_size.parse::<u64>().unwrap_or(0), Ordering::SeqCst);
            }
            if ok {
                let first = urls.get(0).unwrap();
                let fnn = first.split('/').last().unwrap_or_default().to_string();
                let ap = std::path::Path::new(&i.directory).to_path_buf();
                let downloading_path = ap.join("downloading");
                let archive_path = downloading_path.join("staging").join(fnn.clone());
                let far = archive_path.to_str().unwrap().to_string();
                log::debug!("Download complete, starting extraction of {} (Multipart possible!) to {}", far, i.directory);
                let ext = fischl::utils::extract_archive_with_progress(far, i.directory.clone(), false, {
                    let dlpayload = dlpayload.clone();
                    let h5 = h5.clone();
                    let instn = instn.clone();
                    let job_id = job_id.clone();
                    move |current, total| {
                        let mut dlp = dlpayload.lock().unwrap();
                        dlp.insert("job_id", job_id.to_string());
                        dlp.insert("name", instn.to_string());
                        dlp.insert("install_progress", current.to_string());
                        dlp.insert("install_total", total.to_string());
                        dlp.insert("phase", "3".to_string());
                        h5.emit("repair_progress", dlp.clone()).unwrap();
                    }
                });
                if ext {
                    if downloading_path.exists() { std::fs::remove_dir_all(&downloading_path).unwrap_or_default(); }
                    h5.emit("repair_complete", ()).unwrap();
                    log::debug!("Extraction complete for {}, marking repair as complete", i.name);
                    success = true;
                }
            } else {
                if !cancel_token.load(Ordering::Relaxed) { show_dialog(&h5, "warning", "TwintailLauncher", &format!("Error occurred while trying to repair {}\nPlease try again!", i.name), Some(vec!["Ok"])); }
                h5.emit("repair_complete", ()).unwrap();
                log::debug!("Error occurred during DOWNLOAD_MODE_FILE repair for {}, marking as failed", i.name);
            }
        }
        "DOWNLOAD_MODE_CHUNK" => {
            let install_dir = std::path::Path::new(&i.directory);
            if !install_dir.exists() { std::fs::create_dir_all(install_dir).unwrap_or_default(); }

            log::debug!("Starting repair for {} with DOWNLOAD_MODE_CHUNK", i.name);
            let urls = if gm.biz == "bh3_global" { picked.game.full.clone().iter().filter(|e| e.region_code.clone() == i.region_code.clone()).cloned().collect::<Vec<FullGameFile>>() } else { picked.game.full.clone() };
            // Pre-calculate combined totals across all manifest files
            let combined_download_total: u64 = if gm.biz == "bh3_global" { urls.iter().filter(|e| e.region_code.clone() == i.region_code.clone()).map(|e| e.compressed_size.parse::<u64>().unwrap_or(0)).sum() } else { urls.iter().map(|e| e.compressed_size.parse::<u64>().unwrap_or(0)).sum() };
            let combined_install_total: u64 = if gm.biz == "bh3_global" { urls.iter().filter(|e| e.region_code.clone() == i.region_code.clone()).map(|e| e.decompressed_size.parse::<u64>().unwrap_or(0)).sum() } else { urls.iter().map(|e| e.decompressed_size.parse::<u64>().unwrap_or(0)).sum() };
            // Track cumulative progress from completed manifests
            let cumulative_download = Arc::new(std::sync::atomic::AtomicU64::new(0));
            let cumulative_install = Arc::new(std::sync::atomic::AtomicU64::new(0));
            let total_manifests = urls.len();
            let mut ok = true;
            for (manifest_idx, e) in urls.clone().into_iter().enumerate() {
                let h5 = h5.clone();
                let cancel_token = cancel_token.clone();
                let cumulative_download = cumulative_download.clone();
                let cumulative_install = cumulative_install.clone();
                let is_last_manifest = manifest_idx == total_manifests - 1;
                let rslt = run_async_command(async {
                    <Game as Sophon>::repair_game(e.file_url.clone(), e.file_path.clone(), i.directory.clone(), i.skip_hash_check.clone(), {
                        let dlpayload = dlpayload.clone();
                        let instn = instn.clone();
                        let job_id = job_id.clone();
                        let cumulative_download = cumulative_download.clone();
                        let cumulative_install = cumulative_install.clone();
                        move |download_current, _download_total, install_current, _install_total, net_speed, disk_speed, phase| {
                            let mut dlp = dlpayload.lock().unwrap();
                            let instn = instn.to_string();
                            // Add cumulative progress from previous manifests to current progress
                            let total_download_progress = cumulative_download.load(Ordering::SeqCst) + download_current;
                            let total_install_progress = cumulative_install.load(Ordering::SeqCst) + install_current;
                            dlp.insert("job_id", job_id.to_string());
                            dlp.insert("name", instn.clone());
                            dlp.insert("progress", total_download_progress.to_string());
                            dlp.insert("total", combined_download_total.to_string());
                            dlp.insert("speed", net_speed.to_string());
                            dlp.insert("disk", disk_speed.to_string());
                            // Include install progress in same event to avoid flickering
                            dlp.insert("install_progress", total_install_progress.to_string());
                            dlp.insert("install_total", combined_install_total.to_string());
                            // Phase: 0=idle, 1=verifying, 2=downloading, 3=installing, 4=validating, 5=moving
                            // Override phase 5 (moving) to phase 2 (downloading) if not on last manifest
                            let effective_phase = if phase == 5 && !is_last_manifest { 2 } else { phase };
                            dlp.insert("phase", effective_phase.to_string());
                            h5.emit("repair_progress", dlp.clone()).unwrap();
                            drop(dlp);
                        }
                    }, Some(cancel_token.clone()), Some(verified_files.clone())).await
                });
                if !rslt { ok = false;break; }
                // After manifest completes, add its size to cumulative progress
                cumulative_download.fetch_add(e.compressed_size.parse::<u64>().unwrap_or(0), Ordering::SeqCst);
                cumulative_install.fetch_add(e.decompressed_size.parse::<u64>().unwrap_or(0), Ordering::SeqCst);
            }
            if ok {
                h5.emit("repair_complete", ()).unwrap();
                log::debug!("Repair completed for {} with DOWNLOAD_MODE_CHUNK", i.name);
                success = true;
            } else {
                if !cancel_token.load(Ordering::Relaxed) { show_dialog(&h5, "warning", "TwintailLauncher", &format!("Error occurred while trying to repair {}\nPlease try again!", i.name), Some(vec!["Ok"])); }
                h5.emit("repair_complete", ()).unwrap();
                log::debug!("Repair failed for {} with DOWNLOAD_MODE_CHUNK", i.name);
            }
        }
        "DOWNLOAD_MODE_RAW" => {
            let install_dir = std::path::Path::new(&i.directory);
            if !install_dir.exists() { std::fs::create_dir_all(install_dir).unwrap_or_default(); }
            #[cfg(target_os = "linux")]
            crate::utils::apply_patch(&h5, i.directory.clone(), "aki".to_string(), "remove".to_string());

            log::debug!("Starting repair for {} with DOWNLOAD_MODE_RAW", i.name);
            let urls = picked.game.full.iter().map(|v| v.file_url.clone()).collect::<Vec<String>>();
            let manifest = urls.get(0).unwrap();
            let cancel_token = cancel_token.clone();
            let rslt = run_async_command(async {
                <Game as Kuro>::repair_game(manifest.to_owned(), picked.metadata.res_list_url.clone(), i.directory.clone(), i.skip_hash_check.clone(), {
                        let dlpayload = dlpayload.clone();
                        let job_id = job_id.clone();
                        move |download_current, download_total, install_current, install_total, net_speed, disk_speed, phase| {
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
                            tmp.emit("repair_progress", dlp.clone()).unwrap();
                            drop(dlp);
                        }
                    }, Some(cancel_token.clone()), Some(verified_files.clone())).await
            });
            if rslt {
                h5.emit("repair_complete", ()).unwrap();
                log::debug!("Repair completed for {} with DOWNLOAD_MODE_RAW", i.name);
                success = true;
                #[cfg(target_os = "linux")]
                crate::utils::apply_patch(&h5, i.directory.clone(), "aki".to_string(), "add".to_string());
            } else {
                if !cancel_token.load(Ordering::Relaxed) { show_dialog(&h5, "warning", "TwintailLauncher", &format!("Error occurred while trying to repair {}\nPlease try again!", i.name), Some(vec!["Ok"])); }
                h5.emit("repair_complete", ()).unwrap();
                log::debug!("Repair failed for {} with DOWNLOAD_MODE_RAW", i.name);
            }
        }
        "DOWNLOAD_MODE_MULTIFILE" => {
            let install_dir = std::path::Path::new(&i.directory);
            if !install_dir.exists() { std::fs::create_dir_all(install_dir).unwrap_or_default(); }

            log::debug!("Starting game repair using DOWNLOAD_MODE_MULTIFILE with {} file(s)", picked.game.full.len());
            let files = picked.game.full.clone();
            let combined_download_total: u64 = files.iter().map(|e| e.compressed_size.parse::<u64>().unwrap_or(0)).sum();
            let combined_install_total: u64 = files.iter().map(|e| e.decompressed_size.parse::<u64>().unwrap_or(0)).sum();
            let cumulative_download = Arc::new(std::sync::atomic::AtomicU64::new(0));
            let cumulative_install = Arc::new(std::sync::atomic::AtomicU64::new(0));
            let total_files = files.len();
            let mut ok = true;
            for (_file_idx, e) in files.iter().enumerate() {
                let url = e.file_url.clone();
                let cancel_token = cancel_token.clone();
                let dl_ok = run_async_command(async {
                    <Game as Zipped>::download(url.clone(), i.directory.clone(), false, true,{
                            let dlpayload = dlpayload.clone();
                            let h5 = h5.clone();
                            let instn = instn.clone();
                            let job_id = job_id.clone();
                            let cumulative_download = cumulative_download.clone();
                            move |current, _total, net_speed, disk_speed| {
                                let mut dlp = dlpayload.lock().unwrap();
                                let total_dl_progress = cumulative_download.load(Ordering::SeqCst) + current;
                                dlp.insert("job_id", job_id.to_string());
                                dlp.insert("name", instn.to_string());
                                dlp.insert("progress", total_dl_progress.to_string());
                                dlp.insert("total", combined_download_total.to_string());
                                dlp.insert("speed", net_speed.to_string());
                                dlp.insert("disk", disk_speed.to_string());
                                dlp.insert("install_progress", "0".to_string());
                                dlp.insert("install_total", combined_install_total.to_string());
                                dlp.insert("phase", "2".to_string());
                                h5.emit("repair_progress", dlp.clone()).unwrap();
                                drop(dlp);
                            }
                        }, Some(cancel_token.clone()), Some(verified_files.clone())).await
                });
                if !dl_ok { ok = false; break; }
                cumulative_download.fetch_add(e.compressed_size.parse::<u64>().unwrap_or(0), Ordering::SeqCst);
            }
            if ok {
                let ap = std::path::Path::new(&i.directory).to_path_buf();
                let downloading_path = ap.join("downloading");
                ok = true;
                for (file_idx, e) in files.iter().enumerate() {
                    let fnn = e.file_url.split('/').last().unwrap_or_default().to_string();
                    let archive_path = downloading_path.join("staging").join(&fnn);
                    let far = archive_path.to_str().unwrap().to_string();
                    let file_install_size = e.decompressed_size.parse::<u64>().unwrap_or(0);
                    if !archive_path.exists() { log::debug!("Archive {} not found at expected path, cannot extract ({}/{})", far, file_idx + 1, total_files); ok = false; break; }
                    log::debug!("Extracting archive {} to {} ({}/{})", far, i.directory, file_idx + 1, total_files);
                    let ext = fischl::utils::extract_archive_with_progress(far, i.directory.clone(), false, {
                        let dlpayload = dlpayload.clone();
                        let h5 = h5.clone();
                        let instn = instn.clone();
                        let job_id = job_id.clone();
                        let cumulative_install = cumulative_install.clone();
                        move |current, _total| {
                            let mut dlp = dlpayload.lock().unwrap();
                            let total_inst_progress = cumulative_install.load(Ordering::SeqCst) + current;
                            dlp.insert("job_id", job_id.to_string());
                            dlp.insert("name", instn.to_string());
                            dlp.insert("install_progress", total_inst_progress.to_string());
                            dlp.insert("install_total", combined_install_total.to_string());
                            dlp.insert("phase", "3".to_string());
                            h5.emit("repair_progress", dlp.clone()).unwrap();
                        }
                    });
                    if !ext { ok = false; break; }
                    cumulative_install.fetch_add(file_install_size, Ordering::SeqCst);
                }
                if ok {
                    if downloading_path.exists() { std::fs::remove_dir_all(&downloading_path).unwrap_or_default(); }
                    h5.emit("repair_complete", ()).unwrap();
                    log::debug!("All {} archives extracted for {}, marking repair as complete", total_files, i.name);
                    success = true;
                } else {
                    if !cancel_token.load(Ordering::Relaxed) { show_dialog(&h5, "warning", "TwintailLauncher", &format!("Error occurred while trying to repair {}\nPlease try again!", i.name), Some(vec!["Ok"])); }
                    h5.emit("repair_complete", ()).unwrap();
                    log::debug!("Error occurred during DOWNLOAD_MODE_MULTIFILE repair extraction for {}, marking as failed", i.name);
                }
            } else {
                if !cancel_token.load(Ordering::Relaxed) { show_dialog(&h5, "warning", "TwintailLauncher", &format!("Error occurred while trying to repair {}\nPlease try again!", i.name), Some(vec!["Ok"])); }
                h5.emit("repair_complete", ()).unwrap();
                log::debug!("Error occurred during DOWNLOAD_MODE_MULTIFILE repair for {}, marking as failed", i.name);
            }
        }
        _ => { log::debug!("We should not be here... HOW IN THE ABSOLUTE HELL DID WE GET HERE? DOWNLOAD_MODE_???"); show_dialog(&h5, "error", "TwintailLauncher", "Unsupported download mode for repair!", Some(vec!["Ok"])); }
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
        dlp.insert("name", i.name.clone());
        h5.emit("repair_paused", dlp).unwrap();
        return QueueJobOutcome::Cancelled;
    }
    if success {
        { verified_files.lock().unwrap().clear(); }
        QueueJobOutcome::Completed
    } else {
        { verified_files.lock().unwrap().clear(); }
        QueueJobOutcome::Failed
    }
}
