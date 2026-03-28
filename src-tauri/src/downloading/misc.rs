use crate::utils::{compare_version, db_manager::get_settings, empty_dir, find_package_version, run_async_command, show_dialog_with_callback};
use fischl::download::Extras;
use std::collections::HashMap;
use std::fs;
use std::path::{Path,PathBuf};
use tauri::{AppHandle,Emitter,Manager};
use crate::DownloadState;
use crate::downloading::{QueueJobPayload, queue::{QueueJobKind}};

#[cfg(target_os = "linux")]
use crate::downloading::queue::{QueueJobOutcome};
#[cfg(target_os = "linux")]
use crate::downloading::{RunnerDownloadPayload,SteamrtDownloadPayload};
#[cfg(target_os = "linux")]
use crate::utils::db_manager::update_installed_runner_is_installed_by_version;
#[cfg(target_os = "linux")]
use crate::utils::show_dialog;
#[cfg(target_os = "linux")]
use fischl::compat::{download_runner, check_steamrt_update, download_steamrt};
#[cfg(target_os = "linux")]
use std::sync::{Arc,Mutex};

#[cfg(target_os = "linux")]
pub fn download_or_update_steamrt3(app: &AppHandle) {
    let gs = get_settings(app);
    if let Some(s) = gs {
        let rp = Path::new(&s.default_runner_path);
        let steamrt = rp.join("steamrt").join("steamrt3");
        if !steamrt.exists() { if let Err(e) = fs::create_dir_all(&steamrt) { show_dialog(&app, "error", "TwintailLauncher", format!("Failed to prepare SteamLinuxRuntime 3 directory. {} - Please fix the error and restart the app!", e.to_string()).as_str(), None); return; } }
        let steamrt_path = steamrt.to_str().unwrap().to_string();

        if fs::read_dir(&steamrt).unwrap().next().is_none() {
            // Fresh download - enqueue via queue system
            let state = app.state::<DownloadState>();
            let q = state.queue.lock().unwrap().clone();
            if let Some(queue) = q { queue.enqueue(QueueJobKind::SteamrtDownload, QueueJobPayload::Steamrt(SteamrtDownloadPayload { steamrt_path, is_update: false })); }
        } else {
            // Check for updates
            let vp = steamrt.join("VERSIONS.txt");
            if !vp.exists() { return; }
            let cur_ver = crate::utils::find_steamrt_version(vp).unwrap();
            if cur_ver.is_empty() { return; }
            let remote_ver = check_steamrt_update("steamrt3".to_string(), "latest-public-beta".to_string());
            if let Some(rv) = remote_ver {
                if crate::utils::compare_steamrt_versions(&rv, &cur_ver) {
                    empty_dir(steamrt.as_path()).unwrap();
                    // Update - enqueue via queue system
                    let state = app.state::<DownloadState>();
                    let q = state.queue.lock().unwrap().clone();
                    if let Some(queue) = q { queue.enqueue(QueueJobKind::SteamrtDownload, QueueJobPayload::Steamrt(SteamrtDownloadPayload { steamrt_path, is_update: true })); }
                } else {
                    log::info!("SteamLinuxRuntime 3 is up to date!");
                    #[cfg(debug_assertions)]
                    println!("SteamLinuxRuntime 3 is up to date!");
                }
            }
        }
    }
}

#[cfg(target_os = "linux")]
pub fn run_steamrt3_download(app: AppHandle, payload: SteamrtDownloadPayload, job_id: String) -> QueueJobOutcome {
    let job_id = Arc::new(job_id);
    let dlpayload: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
    let steamrt_path = PathBuf::from(&payload.steamrt_path);
    let event_name = if payload.is_update { "update_progress" } else { "download_progress" };
    let complete_event = if payload.is_update { "update_complete" } else { "download_complete" };
    {
        let mut dlp = dlpayload.lock().unwrap();
        dlp.insert("job_id".to_string(), job_id.to_string());
        dlp.insert("name".to_string(), String::from("SteamLinuxRuntime 3"));
        dlp.insert("progress".to_string(), "0".to_string());
        dlp.insert("total".to_string(), "1000".to_string());
        dlp.insert("speed".to_string(), "0".to_string());
        dlp.insert("disk".to_string(), "0".to_string());
        dlp.insert("install_progress".to_string(), "0".to_string());
        dlp.insert("install_total".to_string(), "1000".to_string());
        app.emit(event_name, dlp.clone()).unwrap();
    }

    log::debug!("Starting SteamLinuxRuntime 3 {} process", if payload.is_update { "update" } else { "download" });
    let success = run_async_command(async {
        download_steamrt(steamrt_path.clone(), steamrt_path.clone(), "steamrt3".to_string(), "latest-public-beta".to_string(), {
            let app = app.clone();
            let dlpayload = dlpayload.clone();
            let job_id = job_id.clone();
            let event_name = event_name.to_string();
            move |current, total, net_speed, disk_speed| {
                let mut dlp = dlpayload.lock().unwrap();
                dlp.insert("job_id".to_string(), job_id.to_string());
                dlp.insert("name".to_string(), "SteamLinuxRuntime 3".to_string());
                dlp.insert("progress".to_string(), current.to_string());
                dlp.insert("total".to_string(), total.to_string());
                dlp.insert("speed".to_string(), net_speed.to_string());
                dlp.insert("disk".to_string(), disk_speed.to_string());
                dlp.insert("install_progress".to_string(), "0".to_string());
                dlp.insert("install_total".to_string(), "1000".to_string());
                dlp.insert("phase".to_string(), "2".to_string()); // downloading phase
                app.emit(&event_name, dlp.clone()).unwrap();
            }
        }, {
            let app = app.clone();
            let dlpayload = dlpayload.clone();
            let job_id = job_id.clone();
            let event_name = event_name.to_string();
            move |current, total| {
                let mut dlp = dlpayload.lock().unwrap();
                dlp.insert("job_id".to_string(), job_id.to_string());
                dlp.insert("name".to_string(), "SteamLinuxRuntime 3".to_string());
                dlp.insert("install_progress".to_string(), current.to_string());
                dlp.insert("install_total".to_string(), total.to_string());
                dlp.insert("phase".to_string(), "3".to_string()); // installing phase
                app.emit(&event_name, dlp.clone()).unwrap();
            }
        }).await
    });

    if success {
        app.emit(complete_event, String::from("SteamLinuxRuntime 3")).unwrap();
        log::debug!("Finished downloading and extracting SteamLinuxRuntime 3");
        QueueJobOutcome::Completed
    } else {
        show_dialog_with_callback(&app, "error", "TwintailLauncher", if payload.is_update { "Error occurred while trying to update SteamLinuxRuntime 3! Please restart the application to retry." } else { "Error occurred while trying to download SteamLinuxRuntime 3! Please restart the application to retry." }, Some(vec!["Ok"]), Some("dialog_steamrt3_dl_fail"));
        app.emit(complete_event, String::from("SteamLinuxRuntime 3")).unwrap();
        log::debug!("Failed downloading and extracting SteamLinuxRuntime 3");
        QueueJobOutcome::Failed
    }
}

#[cfg(target_os = "linux")]
pub fn download_or_update_steamrt4(app: &AppHandle) {
    let gs = get_settings(app);
    if let Some(s) = gs {
        let rp = Path::new(&s.default_runner_path);
        let steamrt = rp.join("steamrt").join("steamrt4");
        if !steamrt.exists() { if let Err(e) = fs::create_dir_all(&steamrt) { show_dialog(&app, "error", "TwintailLauncher", format!("Failed to prepare SteamLinuxRuntime 4 directory. {} - Please fix the error and restart the app!", e.to_string()).as_str(), None); return; } }
        let steamrt_path = steamrt.to_str().unwrap().to_string();

        if fs::read_dir(&steamrt).unwrap().next().is_none() {
            // Fresh download - enqueue via queue system
            let state = app.state::<DownloadState>();
            let q = state.queue.lock().unwrap().clone();
            if let Some(queue) = q { queue.enqueue(QueueJobKind::Steamrt4Download, QueueJobPayload::Steamrt4(SteamrtDownloadPayload { steamrt_path, is_update: false })); }
        } else {
            // Check for updates
            let vp = steamrt.join("VERSIONS.txt");
            if !vp.exists() { return; }
            let cur_ver = crate::utils::find_steamrt_version(vp).unwrap();
            if cur_ver.is_empty() { return; }
            let remote_ver = check_steamrt_update("steamrt4".to_string(), "latest-public-beta".to_string());
            if let Some(rv) = remote_ver {
                if crate::utils::compare_steamrt_versions(&rv, &cur_ver) {
                    empty_dir(steamrt.as_path()).unwrap();
                    // Update - enqueue via queue system
                    let state = app.state::<DownloadState>();
                    let q = state.queue.lock().unwrap().clone();
                    if let Some(queue) = q { queue.enqueue(QueueJobKind::Steamrt4Download, QueueJobPayload::Steamrt4(SteamrtDownloadPayload { steamrt_path, is_update: true })); }
                } else {
                    log::info!("SteamLinuxRuntime 4 is up to date!");
                    #[cfg(debug_assertions)]
                    println!("SteamLinuxRuntime 4 is up to date!");
                }
            }
        }
    }
}

#[cfg(target_os = "linux")]
pub fn run_steamrt4_download(app: AppHandle, payload: SteamrtDownloadPayload, job_id: String) -> QueueJobOutcome {
    let job_id = Arc::new(job_id);
    let dlpayload: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
    let steamrt_path = PathBuf::from(&payload.steamrt_path);
    let event_name = if payload.is_update { "update_progress" } else { "download_progress" };
    let complete_event = if payload.is_update { "update_complete" } else { "download_complete" };
    {
        let mut dlp = dlpayload.lock().unwrap();
        dlp.insert("job_id".to_string(), job_id.to_string());
        dlp.insert("name".to_string(), String::from("SteamLinuxRuntime 4"));
        dlp.insert("progress".to_string(), "0".to_string());
        dlp.insert("total".to_string(), "1000".to_string());
        dlp.insert("speed".to_string(), "0".to_string());
        dlp.insert("disk".to_string(), "0".to_string());
        dlp.insert("install_progress".to_string(), "0".to_string());
        dlp.insert("install_total".to_string(), "1000".to_string());
        app.emit(event_name, dlp.clone()).unwrap();
    }

    log::debug!("Starting SteamLinuxRuntime 4 {} process", if payload.is_update { "update" } else { "download" });
    let success = run_async_command(async {
        download_steamrt(steamrt_path.clone(), steamrt_path.clone(), "steamrt4".to_string(), "latest-public-beta".to_string(), {
            let app = app.clone();
            let dlpayload = dlpayload.clone();
            let job_id = job_id.clone();
            let event_name = event_name.to_string();
            move |current, total, net_speed, disk_speed| {
                let mut dlp = dlpayload.lock().unwrap();
                dlp.insert("job_id".to_string(), job_id.to_string());
                dlp.insert("name".to_string(), "SteamLinuxRuntime 4".to_string());
                dlp.insert("progress".to_string(), current.to_string());
                dlp.insert("total".to_string(), total.to_string());
                dlp.insert("speed".to_string(), net_speed.to_string());
                dlp.insert("disk".to_string(), disk_speed.to_string());
                dlp.insert("install_progress".to_string(), "0".to_string());
                dlp.insert("install_total".to_string(), "1000".to_string());
                dlp.insert("phase".to_string(), "2".to_string()); // downloading phase
                app.emit(&event_name, dlp.clone()).unwrap();
            }
        }, {
            let app = app.clone();
            let dlpayload = dlpayload.clone();
            let job_id = job_id.clone();
            let event_name = event_name.to_string();
            move |current, total| {
                let mut dlp = dlpayload.lock().unwrap();
                dlp.insert("job_id".to_string(), job_id.to_string());
                dlp.insert("name".to_string(), "SteamLinuxRuntime 4".to_string());
                dlp.insert("install_progress".to_string(), current.to_string());
                dlp.insert("install_total".to_string(), total.to_string());
                dlp.insert("phase".to_string(), "3".to_string()); // installing phase
                app.emit(&event_name, dlp.clone()).unwrap();
            }
        }).await
    });

    if success {
        app.emit(complete_event, String::from("SteamLinuxRuntime 4")).unwrap();
        log::debug!("Finished downloading and extracting SteamLinuxRuntime 4");
        QueueJobOutcome::Completed
    } else {
        show_dialog_with_callback(&app, "error", "TwintailLauncher", if payload.is_update { "Error occurred while trying to update SteamLinuxRuntime 4! Please restart the application to retry." } else { "Error occurred while trying to download SteamLinuxRuntime 4! Please restart the application to retry." }, Some(vec!["Ok"]), Some("dialog_steamrt4_dl_fail"));
        app.emit(complete_event, String::from("SteamLinuxRuntime 4")).unwrap();
        log::debug!("Failed downloading and extracting SteamLinuxRuntime 4");
        QueueJobOutcome::Failed
    }
}

#[cfg(target_os = "linux")]
pub fn run_runner_download(app: AppHandle, payload: RunnerDownloadPayload, job_id: String) -> QueueJobOutcome {
    let job_id = Arc::new(job_id);
    let dlpayload: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
    let runner_name = payload.runner_version.clone();
    {
        let mut dlp = dlpayload.lock().unwrap();
        dlp.insert("job_id".to_string(), job_id.to_string());
        dlp.insert("name".to_string(), runner_name.clone());
        dlp.insert("progress".to_string(), "0".to_string());
        dlp.insert("total".to_string(), "1000".to_string());
        dlp.insert("speed".to_string(), "0".to_string());
        dlp.insert("disk".to_string(), "0".to_string());
        dlp.insert("install_progress".to_string(), "0".to_string());
        dlp.insert("install_total".to_string(), "1000".to_string());
        app.emit("download_progress", dlp.clone()).unwrap();
    }

    log::debug!("Starting download process for runner {}", runner_name);
    let success = run_async_command(async {
        download_runner(payload.runner_url.clone(), payload.runner_path.clone(), true, {
            let app = app.clone();
            let dlpayload = dlpayload.clone();
            let job_id = job_id.clone();
            let runner_name = runner_name.clone();
            move |current, total, net_speed, disk_speed| {
                let mut dlp = dlpayload.lock().unwrap();
                dlp.insert("job_id".to_string(), job_id.to_string());
                dlp.insert("name".to_string(), runner_name.clone());
                dlp.insert("progress".to_string(), current.to_string());
                dlp.insert("total".to_string(), total.to_string());
                dlp.insert("speed".to_string(), net_speed.to_string());
                dlp.insert("disk".to_string(), disk_speed.to_string());
                dlp.insert("install_progress".to_string(), "0".to_string());
                dlp.insert("install_total".to_string(), "1000".to_string());
                dlp.insert("phase".to_string(), "2".to_string()); // downloading phase
                app.emit("download_progress", dlp.clone()).unwrap();
            }
        }, {
            let app = app.clone();
            let dlpayload = dlpayload.clone();
            let job_id = job_id.clone();
            let runner_name = runner_name.clone();
            move |current, total| {
                let mut dlp = dlpayload.lock().unwrap();
                dlp.insert("job_id".to_string(), job_id.to_string());
                dlp.insert("name".to_string(), runner_name.clone());
                dlp.insert("install_progress".to_string(), current.to_string());
                dlp.insert("install_total".to_string(), total.to_string());
                dlp.insert("phase".to_string(), "3".to_string()); // installing phase
                app.emit("download_progress", dlp.clone()).unwrap();
            }
        }).await
    });

    if success {
        update_installed_runner_is_installed_by_version(&app, payload.runner_version.clone(), true);
        app.emit("download_complete", payload.runner_version.clone()).unwrap();
        log::debug!("Finished downloading and extracting {}", runner_name);
        QueueJobOutcome::Completed
    } else {
        show_dialog_with_callback(&app, "error", "TwintailLauncher", format!("Error occurred while trying to download {runner_name}! Please retry later.").as_str(), Some(vec!["Ok"]), Some("dialog_runner_dl_fail"));
        app.emit("download_complete", payload.runner_version.clone()).unwrap();
        let _ = empty_dir(payload.runner_path.clone());
        log::debug!("Failed downloading and extracting {}", runner_name);
        QueueJobOutcome::Failed
    }
}

pub fn check_extras_update(app: &AppHandle) {
    let gs = get_settings(app);
    if gs.is_some() {
        let s = gs.unwrap();
        //let jadeite = Path::new(&s.jadeite_path).to_path_buf();
        let fpsunlock = Path::new(&s.fps_unlock_path).to_path_buf();
        let xxmi = Path::new(&s.xxmi_path).to_path_buf();
        let gimi = xxmi.join("gimi");
        let srmi = xxmi.join("srmi");
        let zzmi = xxmi.join("zzmi");
        let himi = xxmi.join("himi");
        let wwmi = xxmi.join("wwmi");
        let efmi = xxmi.join("efmi");

        //let ver_jadeite = jadeite.join("VERSION.txt");
        let ver_fpsunlock = fpsunlock.join("VERSION.txt");
        let ver_xxmi = xxmi.join("VERSION.txt");
        let ver_gimi = gimi.join("VERSION.txt");
        let ver_srmi = srmi.join("VERSION.txt");
        let ver_zzmi = zzmi.join("VERSION.txt");
        let ver_himi = himi.join("VERSION.txt");
        let ver_wwmi = wwmi.join("VERSION.txt");
        let ver_efmi = efmi.join("VERSION.txt");

        log::info!("Starting extras update check");
        /*if ver_jadeite.exists() {
            download_or_update_extra(app, jadeite, "jadeite".to_string(), "v5.0.1-hotfix".to_string(), true, None);
        } else if jadeite.exists() && fs::read_dir(&jadeite).ok().and_then(|mut d| d.next()).is_some() {
            empty_dir(&jadeite).unwrap();
            download_or_update_extra(app, jadeite, "jadeite".to_string(), "v5.0.1-hotfix".to_string(), false, None);
        }*/

        if ver_fpsunlock.exists() {
            download_or_update_extra(app, fpsunlock, "keqingunlock".to_string(), "keqing_unlock".to_string(), true, None);
        } else if fpsunlock.exists() && fs::read_dir(&fpsunlock).ok().and_then(|mut d| d.next()).is_some() {
            empty_dir(&fpsunlock).unwrap();
            download_or_update_extra(app, fpsunlock, "keqingunlock".to_string(), "keqing_unlock".to_string(), false, None);
        }

        if ver_xxmi.exists() {
            download_or_update_extra(app, xxmi.clone(), "xxmi".to_string(), "xxmi".to_string(), true, None);
        } else if xxmi.exists() && fs::read_dir(&xxmi).ok().and_then(|mut d| d.next()).is_some() {
            empty_dir(&xxmi).unwrap();
            download_or_update_extra(app, xxmi.clone(), "xxmi".to_string(), "xxmi".to_string(), false, None);
        }

        if ver_gimi.exists() {
            download_or_update_extra(app, xxmi.clone(), "xxmi".to_string(), "gimi".to_string(), true, None);
        } else if gimi.exists() && fs::read_dir(&gimi).ok().and_then(|mut d| d.next()).is_some() {
            empty_dir(&gimi).unwrap();
            download_or_update_extra(app, xxmi.clone(), "xxmi".to_string(), "gimi".to_string(), false, None);
        }

        if ver_srmi.exists() {
            download_or_update_extra(app, xxmi.clone(), "xxmi".to_string(), "srmi".to_string(), true, None);
        } else if srmi.exists() && fs::read_dir(&srmi).ok().and_then(|mut d| d.next()).is_some() {
            empty_dir(&srmi).unwrap();
            download_or_update_extra(app, xxmi.clone(), "xxmi".to_string(), "srmi".to_string(), false, None);
        }

        if ver_zzmi.exists() {
            download_or_update_extra(app, xxmi.clone(), "xxmi".to_string(), "zzmi".to_string(), true, None);
        } else if zzmi.exists() && fs::read_dir(&zzmi).ok().and_then(|mut d| d.next()).is_some() {
            empty_dir(&zzmi).unwrap();
            download_or_update_extra(app, xxmi.clone(), "xxmi".to_string(), "zzmi".to_string(), false, None);
        }

        if ver_himi.exists() {
            download_or_update_extra(app, xxmi.clone(), "xxmi".to_string(), "himi".to_string(), true, None);
        } else if himi.exists() && fs::read_dir(&himi).ok().and_then(|mut d| d.next()).is_some() {
            empty_dir(&himi).unwrap();
            download_or_update_extra(app, xxmi.clone(), "xxmi".to_string(), "himi".to_string(), false, None);
        }

        if ver_wwmi.exists() {
            download_or_update_extra(app, xxmi.clone(), "xxmi".to_string(), "wwmi".to_string(), true, None);
        } else if wwmi.exists() && fs::read_dir(&wwmi).ok().and_then(|mut d| d.next()).is_some() {
            empty_dir(&wwmi).unwrap();
            download_or_update_extra(app, xxmi.clone(), "xxmi".to_string(), "wwmi".to_string(), false, None);
        }

        if ver_efmi.exists() {
            download_or_update_extra(app, xxmi.clone(), "xxmi".to_string(), "efmi".to_string(), true, None);
        } else if efmi.exists() && fs::read_dir(&efmi).ok().and_then(|mut d| d.next()).is_some() {
            empty_dir(&efmi).unwrap();
            download_or_update_extra(app, xxmi.clone(), "xxmi".to_string(), "efmi".to_string(), false, None);
        }
        log::info!("Completed extras update check");
    }
}

pub fn download_or_update_extra(app: &AppHandle, path: PathBuf, package_id: String, package_type: String, update_mode: bool, job_id: Option<String>) -> bool {
    if job_id.is_none() {
        if !update_mode {
            let state = app.state::<DownloadState>();
            let q = state.queue.lock().unwrap().clone();
            if let Some(queue) = q { if !queue.has_job_for_id(package_type.clone()) { queue.enqueue(QueueJobKind::ExtrasDownload, QueueJobPayload::Extras(crate::downloading::ExtrasDownloadPayload { path: path.to_str().unwrap().to_string(), package_id, package_type, update_mode: false })); } }
        } else {
            let app = app.clone();
            let path = path.clone();
            std::thread::spawn(move || { download_or_update_extra(&app, path, package_id, package_type, true, Some(String::new())); }); }
        return true;
    }
    let app = app.clone();
    if update_mode {
        if fs::read_dir(&path).unwrap().next().is_some() {
                let manifest = Extras::fetch_ttl_manifest(package_id.clone());
                if let Some(m) = manifest {
                    if m.retcode != 0 {
                        show_dialog_with_callback(&app, "error", "TwintailLauncher", format!("Error occurred while trying to update {package_id}! Please retry later.").as_str(), Some(vec!["Ok"]), Some("dialog_extra_dl_fail"));
                        app.emit("update_complete", package_id.clone()).unwrap();
                        log::debug!("Failed to fetch TTL manifest for {package_id} during update check");
                        return false;
                    } else {
                        let ap = if package_type.as_str() == "xxmi" || package_id.as_str() == "jadeite" || package_id == "keqingunlock" { path.clone() } else { path.join(&package_type) };
                        let ver_path = if package_id == "keqingunlock" || package_id == "jadeite" || package_type == "xxmi" { path.join("VERSION.txt") } else { path.join(package_type.clone()).join("VERSION.txt") };
                        if !ver_path.exists() { return false; }
                        let pkg_type = if package_id == "keqingunlock" || package_id == "jadeite" { package_id.as_str() } else { package_type.as_str() };
                        let local_ver = find_package_version(ver_path.clone(), &pkg_type);
                        if local_ver.is_some() {
                            let lv = local_ver.unwrap();
                            let pkgs = m.data.unwrap();
                            let pkg = pkgs.packages.iter().find(|e| e.package_name.to_ascii_lowercase().contains(package_type.as_str()));
                            if let Some(p) = pkg {
                                if compare_version(lv.as_str(), p.version.as_str()).is_lt() {
                                    if job_id.as_ref().unwrap().is_empty() {
                                        let state = app.state::<DownloadState>();
                                        let q = state.queue.lock().unwrap().clone();
                                        if let Some(queue) = q {
                                            if !queue.has_job_for_id(package_type.clone()) { queue.enqueue(QueueJobKind::ExtrasDownload, QueueJobPayload::Extras(crate::downloading::ExtrasDownloadPayload { path: path.to_str().unwrap().to_string(), package_id: package_id.clone(), package_type: package_type.clone(), update_mode: true })); }
                                        }
                                        return true;
                                    }
                                    if package_type == "xxmi" {
                                        for file in &p.file_list {
                                            let f_path = path.join(file);
                                            if f_path.exists() { let _ = fs::remove_file(f_path); }
                                        }
                                    } else { empty_dir(&ap).unwrap(); }
                                    let dl = run_async_command(async {
                                        let needs_extract = if package_type.as_str() == "keqing_unlock" || package_type.as_str() == "xxmi" { false } else { true };
                                        let needs_append = if package_type.as_str() == "gimi" || package_type.as_str() == "srmi" || package_type.as_str() == "zzmi" || package_type.as_str() == "himi" || package_type.as_str() == "wwmi" || package_type.as_str() == "ssmi" || package_type.as_str() == "efmi" { true } else { false };
                                        Extras::download_extra_package(package_id.clone(), package_type.clone(), needs_extract, false, needs_append, ap.as_path().to_str().unwrap().parse().unwrap(), |_current, _total| {}).await
                                    });
                                    if dl {
                                        let mi_variants = if package_type == "xxmi" { vec!["gimi", "srmi", "zzmi", "wwmi", "himi"/*, "efmi"*/] } else if package_type.as_str() == "gimi" || package_type.as_str() == "srmi" || package_type.as_str() == "zzmi" || package_type.as_str() == "himi" || package_type.as_str() == "wwmi" || package_type.as_str() == "ssmi" || package_type.as_str() == "efmi" { vec![package_type.as_str()] } else { vec![] };
                                        for mi in mi_variants {
                                            for lib in ["d3d11.dll", "d3dcompiler_47.dll"] {
                                                let linkedpath = path.join(mi).join(lib);
                                                let _ = fs::remove_file(&linkedpath);
                                                let source_lib = path.join(lib);
                                                if !linkedpath.exists() && source_lib.exists() {
                                                    #[cfg(target_os = "linux")]
                                                    let _ = std::os::unix::fs::symlink(&source_lib, &linkedpath);
                                                    #[cfg(target_os = "windows")]
                                                    let _ = fs::copy(&source_lib, &linkedpath);
                                                }
                                            }
                                        }
                                        app.emit("update_complete", package_id.clone()).unwrap();
                                        log::debug!("Successfully updated {package_id} to version {}", p.version);
                                        return true;
                                    } else {
                                        show_dialog_with_callback(&app, "error", "TwintailLauncher", format!("Error occurred while trying to update {package_id}! Please retry later.").as_str(), Some(vec!["Ok"]), Some("dialog_extra_dl_fail"));
                                        app.emit("update_complete", package_id.clone()).unwrap();
                                        empty_dir(&path).unwrap();
                                        log::debug!("Failed to update {package_id} to version {}", p.version);
                                        return false;
                                    }
                                }
                            }
                        }
                    }
                }
        }
        true // Nothing to update
    } else {
        let ap = if package_type.as_str() == "gimi" || package_type.as_str() == "srmi" || package_type.as_str() == "zzmi" || package_type.as_str() == "himi" || package_type.as_str() == "wwmi" || package_type.as_str() == "ssmi" || package_type.as_str() == "efmi" { path.join(&package_type) } else { path.clone() };
        let entries: Vec<_> = fs::read_dir(&ap).ok().map(|r| r.filter_map(|e| e.ok()).collect()).unwrap_or_default();
        let is_effectively_empty = if package_type == "xxmi" { entries.iter().all(|e| { let name = e.file_name(); e.path().is_dir() && (name == "gimi" || name == "srmi" || name == "zzmi" || name == "himi" || name == "wwmi" || name == "ssmi" || name == "efmi") }) } else { entries.is_empty() || entries.iter().all(|e| e.file_name().to_str().unwrap().contains("Mods") || e.file_name().to_str().unwrap().contains("ShaderCache") || e.file_name() == "d3dx_user.ini") };
        if is_effectively_empty {
                let mut dlpayload = HashMap::new();
                dlpayload.insert("name", package_id.clone().chars().next().map(|first| first.to_uppercase().collect::<String>() + &package_id[first.len_utf8()..]).unwrap_or_default());
                dlpayload.insert("progress", "0".to_string());
                dlpayload.insert("total", "1000".to_string());
                if let Some(ref jid) = job_id { if !jid.is_empty() { dlpayload.insert("job_id", jid.clone()); } }
                app.emit("download_progress", dlpayload.clone()).unwrap();

                log::debug!("Starting download process for {package_id} ({package_type})");
                if !ap.exists() { let _ = fs::create_dir_all(&ap); }
                let dl = run_async_command(async {
                    let needs_extract = if package_type.as_str() == "keqing_unlock" || package_type.as_str() == "xxmi" { false } else { true };
                    let needs_append = if package_type.as_str() == "gimi" || package_type.as_str() == "srmi" || package_type.as_str() == "zzmi" || package_type.as_str() == "himi" || package_type.as_str() == "wwmi" || package_type.as_str() == "ssmi" || package_type.as_str() == "efmi" { true } else { false };
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
                    let mi_variants = if package_id == "xxmi" { vec!["gimi", "srmi", "zzmi", "wwmi", "himi", "efmi"] } else if package_type.as_str() == "gimi" || package_type.as_str() == "srmi" || package_type.as_str() == "zzmi" || package_type.as_str() == "himi" || package_type.as_str() == "wwmi" || package_type.as_str() == "ssmi" || package_type.as_str() == "efmi" { vec![package_type.as_str()] } else { vec![] };
                    for mi in mi_variants {
                        for lib in ["d3d11.dll", "d3dcompiler_47.dll"] {
                            let linkedpath = path.join(mi).join(lib);
                            let _ = fs::remove_file(&linkedpath);
                            let source_lib = path.join(lib);
                            if !linkedpath.exists() && source_lib.exists() {
                                #[cfg(target_os = "linux")]
                                let _ = std::os::unix::fs::symlink(&source_lib, &linkedpath);
                                #[cfg(target_os = "windows")]
                                let _ = fs::copy(&source_lib, &linkedpath);
                            }
                        }
                    }
                    app.emit("download_complete", package_id.clone()).unwrap();
                    log::debug!("Finished downloading {package_id}");
                    return true;
                } else {
                    show_dialog_with_callback(&app, "error", "TwintailLauncher", format!("Error occurred while trying to download {package_id}! Please retry later.").as_str(), Some(vec!["Ok"]), Some("dialog_extra_dl_fail"));
                    app.emit("download_complete", package_id.clone()).unwrap();
                    empty_dir(&path).unwrap();
                    log::debug!("Failed downloading {package_id}");
                    return false;
                }
        }
        true // Already downloaded
    }
}
