use crate::utils::db_manager::{
    get_installed_runner_info_by_id, get_installed_runner_info_by_version, get_installed_runners,
    get_installs, get_settings, update_install_runner_location_by_id,
    update_install_runner_version_by_id, update_installed_runner_is_installed_by_version,
};
use std::fs;
use std::path::Path;
use tauri::AppHandle;

#[cfg(target_os = "linux")]
use crate::DownloadState;
#[cfg(target_os = "linux")]
use crate::downloading::queue::QueueJobKind;
#[cfg(target_os = "linux")]
use crate::downloading::{QueueJobPayload, RunnerDownloadPayload};
#[cfg(target_os = "linux")]
use crate::utils::db_manager::create_installed_runner;
#[cfg(target_os = "linux")]
use crate::utils::models::LauncherRunner;
#[cfg(target_os = "linux")]
use crate::utils::repo_manager::get_compatibility;
#[cfg(target_os = "linux")]
use crate::utils::runner_from_runner_version;
#[cfg(target_os = "linux")]
use tauri::Manager;

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
            let rm = get_compatibility(&app, &runner_from_runner_version(&app, runner_version.as_str().to_string()).unwrap_or_default()).unwrap();
            let rv = rm.versions.into_iter().filter(|v| v.version.as_str() == runner_version.as_str()).collect::<Vec<_>>();
            let runnerp = rv.get(0).unwrap().to_owned();
            let runner_path = Path::new(&gs.default_runner_path).join(runner_version.clone());
            if !runner_path.exists() { fs::create_dir_all(&runner_path).unwrap(); }
            let ir = get_installed_runner_info_by_version(&app, runner_version.clone());

            // Empty folder download
            if fs::read_dir(runner_path.as_path()).unwrap().next().is_none() {
                // Check if this runner version is already queued/downloading
                let state = app.state::<DownloadState>();
                let q = state.queue.lock().unwrap().clone();
                if let Some(ref queue) = q {
                    if queue.has_job_for_id(runner_version.clone()) {
                        crate::utils::show_dialog_with_callback(&app, "warning", "TwintailLauncher", format!("Runner {} is already queued for download!", runner_version.as_str()).as_str(), None, None);
                        return Some(false);
                    }
                }

                // Determine the download URL based on architecture
                let mut dl_url = runnerp.url.clone();
                if let Some(urls) = runnerp.urls {
                    #[cfg(target_arch = "x86_64")]
                    { dl_url = urls.x86_64; }
                    #[cfg(target_arch = "aarch64")]
                    { dl_url = if urls.aarch64.is_empty() { runnerp.url.clone() } else { urls.aarch64 }; }
                }

                // Enqueue the download job
                if let Some(queue) = q {
                    queue.enqueue(QueueJobKind::RunnerDownload, QueueJobPayload::Runner(RunnerDownloadPayload {
                            runner_version: runner_version.clone(),
                            runner_url: dl_url,
                            runner_path: runner_path.to_str().unwrap().to_string(),
                    }));
                }
                // Create/update database entry (will be marked as installed by the download job on completion)
                if ir.is_some() { update_installed_runner_is_installed_by_version(&app, runner_version.clone(), false); } else { create_installed_runner(&app, runner_version.clone(), false, runner_path.to_str().unwrap().to_string()).unwrap(); }
                Some(true)
            } else {
                crate::utils::show_dialog_with_callback(&app, "info", "TwintailLauncher", format!("Runner {runn} already installed!", runn = runner_version.clone().as_str().to_string()).as_str(), None, None);
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
        let runner_path = Path::new(&gs.default_runner_path).join(runner_version.clone());
        if !runner_path.exists() { fs::create_dir_all(&runner_path).unwrap(); }

        if fs::read_dir(runner_path.as_path()).unwrap().next().is_some() {
            fs::remove_dir_all(runner_path.as_path()).unwrap();
            update_installed_runner_is_installed_by_version(&app, runner_version.clone(), false);

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
            crate::utils::show_dialog_with_callback(&app, "info", "TwintailLauncher", format!("Runner {runn} is not installed!", runn = runner_version.as_str().to_string()).as_str(), None, None);
            Some(false)
        }
    }
}

#[allow(unused_variables)]
#[tauri::command]
pub fn is_steamrt_installed(app: AppHandle) -> bool {
    #[cfg(target_os = "linux")]
    {
        let gs = match get_settings(&app) {
            Some(s) => s,
            None => return false,
        };
        let steamrt_path = Path::new(&gs.default_runner_path).join("steamrt");
        if !steamrt_path.exists() { return false; }
        match fs::read_dir(&steamrt_path) { Ok(mut entries) => entries.next().is_some(), Err(_) => false }
    }
    #[cfg(target_os = "windows")]
    {
        // SteamRT is not needed on Windows
        true
    }
}
