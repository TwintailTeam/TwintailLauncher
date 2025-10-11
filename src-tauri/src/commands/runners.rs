use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tauri::{AppHandle, Emitter};
use crate::utils::db_manager::{create_installed_runner, get_installed_runner_info_by_id, get_installed_runner_info_by_version, get_installed_runners, get_installs, get_settings, update_install_runner_location_by_id, update_install_runner_version_by_id, update_installed_runner_is_installed_by_version};
use crate::utils::{prevent_exit, send_notification, PathResolve};

#[cfg(target_os = "linux")]
use fischl::compat::Compat;
#[cfg(target_os = "linux")]
use std::sync::Arc;
use crate::utils::repo_manager::LauncherRunner;

#[tauri::command]
pub fn list_installed_runners(app: AppHandle) -> Option<String> {
    let repos = get_installed_runners(&app);

    if repos.is_some() {
        let repository = repos.unwrap();
        let d: Vec<&LauncherRunner> = repository.iter().filter(|r| !r.version.to_ascii_lowercase().contains("dxvk")).collect::<_>();
        let stringified = serde_json::to_string(&d).unwrap();
        Some(stringified)
    } else {
        None
    }
}

#[tauri::command]
pub fn get_installed_runner_by_id(app: AppHandle, runner_id: String) -> Option<String> {
    let repo = get_installed_runner_info_by_id(&app, runner_id);

    if repo.is_some() {
        let repository = repo.unwrap();
        let stringified = serde_json::to_string(&repository).unwrap();
        Some(stringified)
    } else {
        None
    }
}

#[tauri::command]
pub fn get_installed_runner_by_version(app: AppHandle, runner_version: String) -> Option<String> {
    let repo = get_installed_runner_info_by_version(&app, runner_version);

    if repo.is_some() {
        let repository = repo.unwrap();
        let stringified = serde_json::to_string(&repository).unwrap();
        Some(stringified)
    } else {
        None
    }
}

#[tauri::command]
pub fn update_installed_runner_install_status(app: AppHandle, version: String, is_installed: bool) -> Option<bool> {
    let manifest = get_installed_runner_info_by_version(&app, version.clone());

    if manifest.is_some() {
        let m = manifest.unwrap();
        update_installed_runner_is_installed_by_version(&app, m.version, is_installed);
        Some(true)
    } else {
        None
    }
}

#[tauri::command]
pub fn add_installed_runner(app: AppHandle, runner_url: String, runner_version: String) -> Option<bool> {
    if runner_url.is_empty() || runner_version.is_empty() {
        None
    } else {
        let gs = get_settings(&app).unwrap();
        let runner_path = Path::new(&gs.default_runner_path).follow_symlink().unwrap().join(runner_version.clone()).follow_symlink().unwrap();
        if !runner_path.exists() { fs::create_dir_all(&runner_path).unwrap(); }

        let ir = get_installed_runner_info_by_version(&app, runner_version.clone());

        // Empty folder download
        if fs::read_dir(runner_path.as_path()).unwrap().next().is_none() {
            let mut dlpayload = HashMap::new();

            dlpayload.insert("name", runner_version.to_string());
            dlpayload.insert("progress", "0".to_string());
            dlpayload.insert("total", "1000".to_string());
            app.emit("download_progress", dlpayload.clone()).unwrap();
            prevent_exit(&app, true);

            #[cfg(target_os = "linux")]
            {
                let archandle = Arc::new(app.clone());
                let runvc = runner_version.clone();
                let runpc = runner_path.clone();

                std::thread::spawn(move || {
                    let r0 = Compat::download_runner(runner_url, runpc.to_str().unwrap().to_string(), true, {
                        let archandle = archandle.clone();
                        let dlpayload = dlpayload.clone();
                        let runv = runvc.clone();
                        move |current, total| {
                            let mut dlpayload = dlpayload.clone();
                            dlpayload.insert("name", runv.to_string());
                            dlpayload.insert("progress", current.to_string());
                            dlpayload.insert("total", total.to_string());
                            archandle.emit("download_progress", dlpayload.clone()).unwrap();
                        }
                    });
                    if r0 {
                        archandle.emit("download_complete", ()).unwrap();
                        prevent_exit(&*archandle, false);
                        send_notification(&*archandle, format!("Download of {runn} complete.", runn = runvc.clone().as_str().to_string()).as_str(), None);
                        true
                    } else { false }
                });
            }
            if ir.is_some() { update_installed_runner_is_installed_by_version(&app, runner_version.clone(), true); } else { create_installed_runner(&app, runner_version.clone(), true, runner_path.to_str().unwrap().to_string()).unwrap(); }
            Some(true)
        } else {
            send_notification(&app, format!("Runner {runn} already installed!", runn = runner_version.clone().as_str().to_string()).as_str(), None);
            Some(false)
        }
    }
}

#[tauri::command]
pub fn remove_installed_runner(app: AppHandle, runner_version: String) -> Option<bool> {
    if runner_version.is_empty() {
        None
    } else {
        let gs = get_settings(&app).unwrap();
        let runner_path = Path::new(&gs.default_runner_path).follow_symlink().unwrap().join(runner_version.clone()).follow_symlink().unwrap();
        if !runner_path.exists() { fs::create_dir_all(&runner_path).unwrap(); }

        if fs::read_dir(runner_path.as_path()).unwrap().next().is_some() {
            fs::remove_dir_all(runner_path.as_path()).unwrap();
            update_installed_runner_is_installed_by_version(&app, runner_version.clone(), false);
            send_notification(&app, format!("Successfully removed {runn} runner.", runn = runner_version.as_str().to_string()).as_str(), None);

            // Set installations using the removed runner to first available one as fallback
            let installs = get_installs(&app);
            if installs.is_some() {
                let insts = installs.unwrap();
                for i in insts {
                    if i.runner_version == runner_version {
                        let available_runners = get_installed_runners(&app);
                        if available_runners.is_some() {
                            let avr = available_runners.unwrap();
                            let filtered_runners = avr.iter().filter(|r| r.is_installed).collect::<Vec<_>>();
                            let first = filtered_runners.get(0).unwrap();
                            update_install_runner_version_by_id(&app, i.id.clone(), first.version.clone());
                            update_install_runner_location_by_id(&app, i.id, first.runner_path.clone());
                        }
                    }
                }
            }
            Some(true)
        } else {
            send_notification(&app, format!("Runner {runn} is not installed!", runn = runner_version.as_str().to_string()).as_str(), None);
            Some(false)
        }
    }
}