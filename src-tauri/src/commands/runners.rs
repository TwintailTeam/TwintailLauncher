use std::fs;
use std::path::Path;
use tauri::{AppHandle};
use crate::utils::db_manager::{get_installed_runner_info_by_id, get_installed_runner_info_by_version, get_installed_runners, get_installs, get_settings, update_install_runner_location_by_id, update_install_runner_version_by_id, update_installed_runner_is_installed_by_version};
use crate::utils::{send_notification, PathResolve};

#[cfg(target_os = "linux")]
use tauri::Emitter;
#[cfg(target_os = "linux")]
use std::collections::HashMap;
#[cfg(target_os = "linux")]
use fischl::compat::Compat;
#[cfg(target_os = "linux")]
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
#[cfg(target_os = "linux")]
use crate::utils::repo_manager::{get_compatibility};
#[cfg(target_os = "linux")]
use crate::utils::{runner_from_runner_version, prevent_exit, run_async_command, models::LauncherRunner};
#[cfg(target_os = "linux")]
use crate::utils::db_manager::{create_installed_runner};

#[allow(unused_variables)]
#[tauri::command]
pub fn list_installed_runners(app: AppHandle) -> Option<String> {
    #[cfg(target_os = "linux")]
    {
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
    #[cfg(target_os = "windows")]
    {
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

#[allow(unused_variables)]
#[tauri::command]
pub fn update_installed_runner_install_status(app: AppHandle, version: String, is_installed: bool) -> Option<bool> {
    #[cfg(target_os = "linux")]
    {
        let manifest = get_installed_runner_info_by_version(&app, version.clone());

        if manifest.is_some() {
            let m = manifest.unwrap();
            update_installed_runner_is_installed_by_version(&app, m.version, is_installed);
            Some(true)
        } else {
            None
        }
    }
    #[cfg(target_os = "windows")]
    {
        None
    }
}

#[allow(unused_variables)]
#[tauri::command]
pub fn add_installed_runner(app: AppHandle, runner_url: String, runner_version: String) -> Option<bool> {
    if runner_url.is_empty() || runner_version.is_empty() {
        None
    } else {
        #[cfg(target_os = "linux")]
        {
            let gs = get_settings(&app).unwrap();
            let rm = get_compatibility(&app, &runner_from_runner_version(runner_version.as_str().to_string()).unwrap()).unwrap();
            let rv = rm.versions.into_iter().filter(|v| v.version.as_str() == runner_version.as_str()).collect::<Vec<_>>();
            let runnerp = rv.get(0).unwrap().to_owned();
            let runner_path = Path::new(&gs.default_runner_path).follow_symlink().unwrap().join(runner_version.clone()).follow_symlink().unwrap();
            if !runner_path.exists() { fs::create_dir_all(&runner_path).unwrap(); }
            let ir = get_installed_runner_info_by_version(&app, runner_version.clone());

            // Empty folder download
            if fs::read_dir(runner_path.as_path()).unwrap().next().is_none() {
                let appc = app.clone();
                let runvc = runner_version.clone();
                let runpc = runner_path.clone();
                std::thread::spawn(move || {
                    let app = appc.clone();
                    let runnerp = runnerp.clone();
                    let runv = runvc.clone();
                    let runpc = runpc.clone();

                    let mut dlp = HashMap::new();
                    dlp.insert("name", runv.to_string());
                    dlp.insert("progress", "0".to_string());
                    dlp.insert("total", "1000".to_string());
                    app.emit("download_progress", dlp.clone()).unwrap();
                    prevent_exit(&app, true);

                    let mut dl_url = runnerp.url.clone(); // Always x86_64
                    if let Some(urls) = runnerp.urls {
                        #[cfg(target_arch = "x86_64")]
                        { dl_url = urls.x86_64; }
                        #[cfg(target_arch = "aarch64")]
                        { dl_url = if urls.aarch64.is_empty() { runnerp.url.clone() } else { urls.aarch64 }; }
                    }

                    let r0 = run_async_command(async {
                        Compat::download_runner(dl_url, runpc.to_str().unwrap().to_string(), true, {
                            let archandle = app.clone();
                            let dlpayload = dlp.clone();
                            let runv = runv.clone();
                            move |current, total| {
                                let mut dlp = dlpayload.clone();
                                dlp.insert("name", runv.to_string());
                                dlp.insert("progress", current.to_string());
                                dlp.insert("total", total.to_string());
                                archandle.emit("download_progress", dlp.clone()).unwrap();
                            }
                        }).await
                    });
                    if r0 {
                        app.emit("download_complete", ()).unwrap();
                        prevent_exit(&app, false);
                        send_notification(&app, format!("Download of {runn} complete.", runn = runv.clone().as_str().to_string()).as_str(), None);
                        true
                    } else {
                        app.dialog().message(format!("Error occurred while trying to download {runn} runner! Please retry later.", runn = runv.clone().as_str().to_string()).as_str()).title("TwintailLauncher")
                            .kind(MessageDialogKind::Error)
                            .buttons(MessageDialogButtons::OkCustom("Ok".to_string()))
                            .show(move |_action| {
                                prevent_exit(&app, false);
                                app.emit("download_complete", ()).unwrap();
                                if runpc.exists() { fs::remove_dir_all(&runpc).unwrap(); update_installed_runner_is_installed_by_version(&app, runv.clone(), false); }
                            });
                        false
                    }
                });
                if ir.is_some() { update_installed_runner_is_installed_by_version(&app, runner_version.clone(), true); } else { create_installed_runner(&app, runner_version.clone(), true, runner_path.to_str().unwrap().to_string()).unwrap(); }
                Some(true)
            } else {
                send_notification(&app, format!("Runner {runn} already installed!", runn = runner_version.clone().as_str().to_string()).as_str(), None);
                Some(false)
            }
        }
        #[cfg(target_os = "windows")]
        {
            Some(true)
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