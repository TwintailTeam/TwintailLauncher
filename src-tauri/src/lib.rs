use std::sync::{Mutex};
use tauri::{Emitter, Manager, RunEvent, WindowEvent};
use crate::commands::install::{add_install, game_launch, get_download_sizes, get_resume_states, get_install_by_id, list_installs, list_installs_by_manifest_id, remove_install, update_install_dxvk_path, update_install_dxvk_version, update_install_env_vars, update_install_fps_value, update_install_game_path, update_install_launch_args, update_install_launch_cmd, update_install_pre_launch_cmd, update_install_prefix_path, update_install_runner_path, update_install_runner_version, update_install_skip_hash_valid, update_install_skip_version_updates, update_install_use_fps_unlock, update_install_use_jadeite, update_install_use_xxmi, update_install_use_gamemode, update_install_use_mangohud, update_install_mangohud_config_path, add_shortcut};
use crate::commands::manifest::{get_manifest_by_filename, get_manifest_by_id, list_game_manifests, get_game_manifest_by_filename, list_manifests_by_repository_id, update_manifest_enabled, get_game_manifest_by_manifest_id, list_compatibility_manifests, get_compatibility_manifest_by_manifest_id};
use crate::commands::repository::{list_repositories, remove_repository, add_repository, get_repository};
use crate::commands::settings::{block_telemetry_cmd, list_settings, open_folder, open_uri, update_extras, update_settings_default_dxvk_path, update_settings_default_fps_unlock_path, update_settings_default_game_path, update_settings_default_jadeite_path, update_settings_default_mangohud_config_path, update_settings_default_prefix_path, update_settings_default_runner_path, update_settings_default_xxmi_path, update_settings_launcher_action, update_settings_manifests_hide, update_settings_third_party_repo_updates};
use crate::downloading::download::register_download_handler;
use crate::downloading::preload::register_preload_handler;
use crate::downloading::repair::register_repair_handler;
use crate::downloading::update::register_update_handler;
use crate::utils::db_manager::{init_db, DbInstances};
use crate::utils::repo_manager::{load_manifests, ManifestLoader, ManifestLoaders};
use crate::utils::{args, block_telemetry, deprecate_jadeite, notify_update, register_listeners, run_async_command, setup_or_fix_default_paths, ActionBlocks};
use crate::utils::system_tray::init_tray;

#[cfg(target_os = "linux")]
use crate::utils::repo_manager::RunnerLoader;

mod utils;
mod commands;
mod downloading;

#[derive(Clone, serde::Serialize)]
struct Payload {
    args: Vec<String>,
    cwd: String,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = {
        #[cfg(target_os = "linux")]
        {
            utils::gpu::fuck_nvidia();
            // Raise file descriptor limit for the app so downloads go smoothly
            utils::raise_fd_limit(999999);
            tauri::Builder::default()
                .manage(Mutex::new(ActionBlocks { action_exit: false }))
                .manage(ManifestLoaders {game: ManifestLoader::default(), runner: RunnerLoader::default()})
                .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| { app.emit("single-instance", Payload { args: argv, cwd }).unwrap(); }))
                .plugin(tauri_plugin_notification::init())
                .plugin(tauri_plugin_dialog::init())
                .plugin(tauri_plugin_opener::init())
        }
        #[cfg(target_os = "windows")]
        {
            tauri::Builder::default()
                .manage(Mutex::new(ActionBlocks { action_exit: false }))
                .manage(ManifestLoaders {game: ManifestLoader::default()})
                .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| { app.emit("single-instance", Payload { args: argv, cwd }).unwrap(); }))
                .plugin(tauri_plugin_notification::init())
                .plugin(tauri_plugin_dialog::init())
                .plugin(tauri_plugin_opener::init())
        }
    }.setup(|app| {
            let handle = app.handle();
            #[cfg(target_arch = "aarch64")]
            {
                use tauri_plugin_dialog::DialogExt;
                let h = handle.clone();
                handle.dialog().message("TwintailLauncher does not support ARM based architectures. Flatpak required ARM builds to be provided but they are not supported!").kind(tauri_plugin_dialog::MessageDialogKind::Warning).title("Unsupported Architecture").show(move |_| { let h = h.clone();h.cleanup_before_exit();h.exit(0);std::process::exit(0); });
            }

            notify_update(handle);
            run_async_command(async { init_db(handle).await; });

            #[cfg(target_arch = "x86_64")]
            {
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
                    std::thread::sleep(std::time::Duration::from_secs(3));
                    handle.cleanup_before_exit();
                    handle.exit(0);
                    std::process::exit(0);
                }

                // Hide decorations on most common tiler WindowManagers on linux
                #[cfg(target_os = "linux")]
                {
                    match std::env::var("XDG_SESSION_DESKTOP") {
                        Ok(val) => {
                            if val.to_ascii_lowercase() == "hyprland" ||
                                val.to_ascii_lowercase() == "i3" ||
                                val.to_ascii_lowercase() == "sway" ||
                                val.to_ascii_lowercase() == "bspwm" ||
                                val.to_ascii_lowercase() == "awesome" ||
                                val.to_ascii_lowercase() == "dwm" ||
                                val.to_ascii_lowercase() == "xmonad" ||
                                val.to_ascii_lowercase() == "qtile" ||
                                val.to_ascii_lowercase() == "niri" {
                                app.get_window("main").unwrap().set_decorations(false).unwrap();
                            } else { app.get_window("main").unwrap().set_decorations(true).unwrap(); }
                        },
                        Err(_e) => {},
                    }
                }

                let res_dir = app.path().resource_dir().unwrap();
                let data_dir = app.path().app_data_dir().unwrap();

                setup_or_fix_default_paths(handle, data_dir.clone(), true);
                update_extras(handle.clone(), false);
                deprecate_jadeite(handle);

                let path = data_dir.join(".telemetry_blocked");
                if !path.exists() { block_telemetry(&handle); }

                for r in ["hpatchz", "hpatchz.exe", "krpatchz", "krpatchz.exe", "7zr", "7zr.exe", "mangohud_default.conf"] {
                    let rd = res_dir.join("resources").join(r);
                    let fd = data_dir.join(r);
                    if rd.file_name().unwrap().to_str().unwrap().contains("mangohud_default.conf") {
                        if rd.exists() && !fd.exists() { std::fs::copy(rd, fd).unwrap(); }
                    } else {
                        if rd.exists() { std::fs::copy(rd, fd).unwrap(); }
                    }
                }
            }
            Ok(())
        }).invoke_handler(tauri::generate_handler![open_uri, open_folder, update_extras, block_telemetry_cmd, list_settings, update_settings_third_party_repo_updates, update_settings_default_game_path, update_settings_default_xxmi_path, update_settings_default_fps_unlock_path, update_settings_default_jadeite_path, update_settings_default_prefix_path, update_settings_default_runner_path, update_settings_default_dxvk_path, update_settings_launcher_action, update_settings_manifests_hide,
            remove_repository, add_repository, get_repository, list_repositories,
            get_manifest_by_id, get_manifest_by_filename, list_manifests_by_repository_id, update_manifest_enabled,
            get_game_manifest_by_filename, list_game_manifests, get_game_manifest_by_manifest_id,
            list_installs, list_installs_by_manifest_id, get_install_by_id, add_install, remove_install,
            update_install_game_path, update_install_runner_path, update_install_dxvk_path, update_install_skip_version_updates, update_install_skip_hash_valid, update_install_use_jadeite, update_install_use_xxmi, update_install_use_fps_unlock, update_install_fps_value, update_install_env_vars, update_install_pre_launch_cmd, update_install_launch_cmd, update_install_prefix_path, update_install_launch_args, update_install_dxvk_version, update_install_runner_version, update_install_use_gamemode, update_install_use_mangohud,
            list_compatibility_manifests, get_compatibility_manifest_by_manifest_id,
            game_launch, get_download_sizes, get_resume_states, update_install_mangohud_config_path, update_settings_default_mangohud_config_path, add_shortcut])
        .build(tauri::generate_context!())
        .expect("Error while running TwintailLauncher!");

    builder.run(|app, event| {
        match &event {
            RunEvent::WindowEvent {event, ..} => {
                match event {
                    WindowEvent::CloseRequested { api, .. } => {
                        let blocks = app.state::<Mutex<ActionBlocks>>();
                        let state = blocks.lock().unwrap();
                        if state.action_exit {
                            app.get_window("main").unwrap().hide().unwrap();
                            api.prevent_close();
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
