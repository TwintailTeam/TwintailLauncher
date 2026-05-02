extern crate core;

use std::sync::{Mutex, Arc};
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use tauri::{AppHandle, Emitter, Manager, RunEvent, WindowEvent};
use crate::commands::install::{add_install, check_game_running, game_launch, get_download_sizes, get_resume_states, get_install_by_id, list_installs, list_installs_by_manifest_id, remove_install, set_installs_order, update_install_dxvk_path, update_install_dxvk_version, update_install_env_vars, update_install_fps_value, update_install_game_background, update_install_game_path, update_install_graphics_api, update_install_launch_args, update_install_launch_cmd, update_install_pre_launch_cmd, update_install_prefix_path, update_install_runner_path, update_install_runner_version, update_install_skip_hash_valid, update_install_skip_version_updates, update_install_use_fps_unlock, update_install_use_jadeite, update_install_use_xxmi, update_install_use_gamemode, update_install_use_mangohud, update_install_mangohud_config_path, add_shortcut, remove_shortcut, update_install_xxmi_config, update_install_show_drpc, update_install_disable_system_idle, copy_authkey};
use crate::commands::queue::{pause_game_download, queue_move_up, queue_move_down, queue_remove, queue_set_paused, queue_activate_job, queue_reorder, queue_resume_job, get_download_queue_state, queue_clear_completed};
use crate::commands::manifest::{get_manifest_by_filename, get_manifest_by_id, list_game_manifests, get_game_manifest_by_filename, list_manifests_by_repository_id, update_manifest_enabled, get_game_manifest_by_manifest_id, list_compatibility_manifests, get_compatibility_manifest_by_manifest_id, override_manifest_url, clear_manifest_override};
use crate::commands::repository::{list_repositories, remove_repository, add_repository, get_repository};
use crate::commands::settings::{empty_folder, list_settings, open_folder, open_in_prefix, open_uri, update_settings_default_dxvk_path, update_settings_default_fps_unlock_path, update_settings_default_game_path, update_settings_default_jadeite_path, update_settings_default_mangohud_config_path, update_settings_default_prefix_path, update_settings_default_runner_path, update_settings_default_xxmi_path, update_settings_download_speed_limit_cmd, update_settings_hide_app_tray, update_settings_launcher_action, update_settings_manifests_hide, update_settings_third_party_repo_updates};
use crate::downloading::download::register_download_handler;
use crate::downloading::preload::register_preload_handler;
use crate::downloading::repair::register_repair_handler;
use crate::downloading::update::register_update_handler;
use crate::downloading::queue::{start_download_queue_worker, QueueJob, QueueJobKind, QueueJobOutcome};
use crate::downloading::QueueJobPayload;
use crate::downloading::misc::check_extras_update;
use crate::utils::db_manager::{init_db, DbInstances};
use crate::utils::repo_manager::{load_manifests, ManifestLoader, ManifestLoaders};
use crate::utils::{args, notify_update, register_listeners, run_async_command, setup_or_fix_default_paths, sync_install_backgrounds};
use crate::utils::system_tray::init_tray;
use crate::commands::runners::{add_installed_runner, get_installed_runner_by_id, get_installed_runner_by_version, is_steamrt_installed, list_installed_runners, remove_installed_runner, update_installed_runner_install_status};
use crate::commands::network::check_network_connectivity;

mod utils;
mod commands;
mod downloading;

pub struct DownloadState {
    pub tokens: Mutex<HashMap<String, Arc<AtomicBool>>>,
    pub queue: Mutex<Option<downloading::queue::DownloadQueueHandle>>,
    pub verified_files: Mutex<HashMap<String, Arc<Mutex<std::collections::HashSet<String>>>>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let logger = tauri_plugin_log::Builder::new().filter(|metadata| !metadata.target().contains("h2")).filter(|metadata| !metadata.target().contains("tracing")).filter(|metadata| !metadata.target().contains("hyper")).max_file_size(8000000).clear_targets().targets([tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir { file_name: Some("twintaillauncher".to_string()) })]).rotation_strategy(tauri_plugin_log::RotationStrategy::KeepSome(5)).timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal).level(if cfg!(debug_assertions) { log::LevelFilter::Trace } else { if std::env::var("TTL_DEBUG").is_ok() { log::LevelFilter::Debug } else { log::LevelFilter::Info } }).build();
    let builder = {
        #[cfg(target_os = "linux")]
        {
            utils::gpu::fuck_nvidia();
            utils::raise_fd_limit(999999);
            let base = tauri::Builder::default()
                .manage(ManifestLoaders {game: ManifestLoader::default(), runner: utils::repo_manager::RunnerLoader::default()})
                .manage(DownloadState { tokens: Mutex::new(HashMap::new()), queue: Mutex::new(None), verified_files: Mutex::new(HashMap::new()) })
                .plugin(tauri_plugin_dialog::init())
                .plugin(tauri_plugin_opener::init())
                .plugin(tauri_plugin_clipboard_manager::init())
                .plugin(logger);
            if args::get_launch_install().is_none() { base.plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| { let _ = app.get_window("main").expect("no main window").show(); let _ = app.get_window("main").expect("no main window").set_focus(); })) } else { base }
        }
        #[cfg(target_os = "windows")]
        {
            let base = tauri::Builder::default()
                .manage(DownloadState { tokens: Mutex::new(HashMap::new()), queue: Mutex::new(None), verified_files: Mutex::new(HashMap::new()) })
                .manage(ManifestLoaders {game: ManifestLoader::default()})
                .plugin(tauri_plugin_dialog::init())
                .plugin(tauri_plugin_opener::init())
                .plugin(tauri_plugin_clipboard_manager::init())
                .plugin(logger);
            if args::get_launch_install().is_none() { base.plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| { let _ = app.get_window("main").expect("no main window").show(); let _ = app.get_window("main").expect("no main window").set_focus(); })) } else { base }
        }
    }.setup(|app| {
            let handle = app.handle();
            #[cfg(target_arch = "aarch64")]
            {
                use tauri_plugin_dialog::DialogExt;
                use tauri::Listener;
                let h = handle.clone();
                handle.dialog().message("TwintailLauncher does not support ARM based architectures. Flatpak required ARM builds to be provided but they are not supported!").kind(tauri_plugin_dialog::MessageDialogKind::Warning).title("Unsupported Architecture").show(move |_| { let h = h.clone();h.cleanup_before_exit();h.exit(0);std::process::exit(0); });
            }

            #[cfg(target_arch = "x86_64")]
            {
                notify_update(handle);
                run_async_command(async { init_db(handle).await; });

                // Start download queue worker (limits concurrent download-like jobs)
                fn run_queued_job(app: AppHandle, job: QueueJob) -> QueueJobOutcome {
                    match (&job.kind, job.payload) {
                        (QueueJobKind::GameDownload, QueueJobPayload::Game(p)) => downloading::download::run_game_download(app, p, job.id),
                        (QueueJobKind::GameUpdate, QueueJobPayload::Game(p)) => downloading::update::run_game_update(app, p, job.id),
                        (QueueJobKind::GamePreload, QueueJobPayload::Game(p)) => downloading::preload::run_game_preload(app, p, job.id),
                        (QueueJobKind::GameRepair, QueueJobPayload::Game(p)) => downloading::repair::run_game_repair(app, p, job.id),
                        #[cfg(target_os = "linux")]
                        (QueueJobKind::RunnerDownload, QueueJobPayload::Runner(p)) => downloading::misc::run_runner_download(app, p, job.id),
                        #[cfg(target_os = "linux")]
                        (QueueJobKind::SteamrtDownload, QueueJobPayload::Steamrt(p)) => downloading::misc::run_steamrt3_download(app, p, job.id),
                        #[cfg(target_os = "linux")]
                        (QueueJobKind::Steamrt4Download, QueueJobPayload::Steamrt4(p)) => downloading::misc::run_steamrt4_download(app, p, job.id),
                        (QueueJobKind::ExtrasDownload, QueueJobPayload::Extras(p)) => {
                            let path = std::path::PathBuf::from(&p.path);
                            if downloading::misc::download_or_update_extra(&app, path, p.package_id, p.package_type, p.update_mode, Some(job.id)) { QueueJobOutcome::Completed } else { QueueJobOutcome::Failed }
                        }
                        // Mismatch between kind and payload - should never happen
                        _ => QueueJobOutcome::Failed,
                    }
                }

                // Only 1 game can download at a time - others wait in queue
                let queue_handle = start_download_queue_worker(handle.clone(), 1, run_queued_job);
                {
                    let state = handle.state::<DownloadState>();
                    let mut q = state.queue.lock().unwrap();
                    *q = Some(queue_handle);
                }

                // Start connection monitor for auto-pause/resume on connectivity changes
                downloading::connection_monitor::start_connection_monitor(handle.clone());
                load_manifests(handle);
                init_tray(handle).unwrap();
                // Initialize the listeners
                register_listeners(handle);
                register_download_handler(handle);
                register_update_handler(handle);
                register_repair_handler(handle);
                register_preload_handler(handle);

                if args::get_launch_install().is_some() {
                    let id = args::get_launch_install().unwrap();
                    game_launch(handle.clone(), id);
                    handle.get_window("main").unwrap().hide().unwrap();
                    std::thread::sleep(std::time::Duration::from_secs(10));
                    handle.cleanup_before_exit();
                    handle.exit(0);
                    std::process::exit(0);
                }

                // Why in the absolute fuck is fedora atomic garbage distros doing /home -> var/home symlink???
                #[cfg(target_os = "linux")]
                let data_dir = { let d = app.path().app_data_dir().unwrap(); if utils::is_flatpak() && std::fs::symlink_metadata("/home").map(|m| m.file_type().is_symlink()).unwrap_or(false) { std::fs::canonicalize(&d).unwrap_or(d) } else { d } };
                #[cfg(target_os = "windows")]
                let data_dir = app.path().app_data_dir().unwrap();
                setup_or_fix_default_paths(handle, data_dir.clone(), true);
                sync_install_backgrounds(handle);
                check_extras_update(handle);

                // https://github.com/tauri-apps/tauri/issues/14596
                #[cfg(target_os = "windows")]
                if let Ok(icon) = tauri::image::Image::from_bytes(include_bytes!("../icons/128x128@2x.png")) { let _ = app.get_window("main").unwrap().set_icon(icon); }

                #[cfg(target_os = "linux")]
                {
                    utils::fix_window_decorations(handle);
                    utils::deprecate_jadeite(handle);
                    utils::sync_installed_runners(handle);
                    downloading::misc::download_or_update_steamrt3(handle);
                    downloading::misc::download_or_update_steamrt4(handle);
                }
                // Delete deprecated resource files (PS: reaper binary is executable in resources dir so useless to copy)
                for df in ["7zr", "7zr.exe", "krpatchz", "krpatchz.exe", "reaper", "hpatchz", "hpatchz.exe"] {
                    let fd = data_dir.join(df);
                    if fd.exists() { std::fs::remove_file(fd).unwrap(); }
                }
            }
            Ok(())
        }).invoke_handler(tauri::generate_handler![open_uri, open_folder, empty_folder, open_in_prefix, list_settings, update_settings_third_party_repo_updates, update_settings_default_game_path, update_settings_default_xxmi_path, update_settings_default_fps_unlock_path, update_settings_default_jadeite_path, update_settings_default_prefix_path, update_settings_default_runner_path, update_settings_default_dxvk_path, update_settings_default_mangohud_config_path, update_settings_download_speed_limit_cmd, update_settings_launcher_action, update_settings_manifests_hide, update_settings_hide_app_tray,
            remove_repository, add_repository, get_repository, list_repositories,
            get_manifest_by_id, get_manifest_by_filename, list_manifests_by_repository_id, update_manifest_enabled,
            get_game_manifest_by_filename, list_game_manifests, get_game_manifest_by_manifest_id, override_manifest_url, clear_manifest_override,
            list_installs, list_installs_by_manifest_id, get_install_by_id, add_install, remove_install, set_installs_order,
            update_install_game_path, update_install_runner_path, update_install_dxvk_path, update_install_skip_version_updates, update_install_skip_hash_valid, update_install_use_jadeite, update_install_use_xxmi, update_install_use_fps_unlock, update_install_fps_value, update_install_graphics_api, update_install_env_vars, update_install_pre_launch_cmd, update_install_launch_cmd, update_install_game_background, update_install_prefix_path, update_install_launch_args, update_install_dxvk_version, update_install_runner_version, update_install_use_gamemode, update_install_use_mangohud, update_install_xxmi_config, update_install_show_drpc, update_install_disable_system_idle, copy_authkey,
            list_compatibility_manifests, get_compatibility_manifest_by_manifest_id,
            game_launch, check_game_running, get_download_sizes, get_resume_states, update_install_mangohud_config_path, update_settings_default_mangohud_config_path, add_shortcut, remove_shortcut, pause_game_download, queue_move_up, queue_move_down, queue_remove, queue_set_paused, queue_activate_job, queue_reorder, queue_resume_job, get_download_queue_state, queue_clear_completed,
            add_installed_runner, remove_installed_runner, get_installed_runner_by_version, get_installed_runner_by_id, list_installed_runners, update_installed_runner_install_status, is_steamrt_installed, check_network_connectivity])
        .build(tauri::generate_context!())
        .expect("Error while running TwintailLauncher!");

    builder.run(|app, event| {
        match &event {
            RunEvent::WindowEvent {event, ..} => {
                match event {
                    WindowEvent::CloseRequested { api, .. } => {
                        let tray_minmize = utils::db_manager::get_settings(&app);
                        if tray_minmize.is_some() {
                            let sett = tray_minmize.unwrap();
                            if sett.hide_app_to_tray {
                                api.prevent_close();
                                app.get_window("main").unwrap().hide().unwrap();
                                app.emit("sync_tray_toggle", "Show").unwrap();
                            }
                        }
                    }
                    _ => {}
                }
            }
            RunEvent::Exit => { run_async_command(async { app.state::<DbInstances>().0.lock().await.get("db").unwrap().close().await; }); }
            _ => ()
        }
    })
}
