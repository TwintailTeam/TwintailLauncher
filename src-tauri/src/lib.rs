#![feature(async_closure)]

use tauri::{Manager, RunEvent};
use crate::commands::install::{add_install, get_install_by_id, list_installs, list_installs_by_manifest_id, remove_install};
use crate::commands::manifest::{get_manifest_by_filename, get_manifest_by_id, list_game_manifests, get_game_manifest_by_filename, list_manifests_by_repository_id, update_manifest_enabled, get_game_manifest_by_manifest_id, list_compatibility_manifests, get_compatibility_manifest_by_manifest_id};
use crate::commands::repository::{list_repositories, remove_repository, add_repository, get_repository};
use crate::commands::settings::{list_settings, update_settings_default_fps_unlock_path, update_settings_default_game_path, update_settings_default_jadeite_path, update_settings_default_xxmi_path, update_settings_third_party_repo_updates};
use crate::utils::db_manager::{init_db, DbInstances};
use crate::utils::repo_manager::{load_manifests, ManifestLoader, ManifestLoaders, RunnerLoader};
use crate::utils::run_async_command;

mod utils;
mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(ManifestLoaders {game: ManifestLoader::default(), runner: RunnerLoader::default()})
        .setup(|app| {
            let handle = app.handle();
            run_async_command(async { init_db(&handle).await; });
            load_manifests(&handle);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![list_settings, update_settings_third_party_repo_updates, update_settings_default_game_path, update_settings_default_xxmi_path, update_settings_default_fps_unlock_path, update_settings_default_jadeite_path,
            remove_repository, add_repository, get_repository, list_repositories,
            get_manifest_by_id, get_manifest_by_filename, list_manifests_by_repository_id, update_manifest_enabled,
            get_game_manifest_by_filename, list_game_manifests, get_game_manifest_by_manifest_id,
            list_installs, list_installs_by_manifest_id, get_install_by_id, add_install, remove_install,
            list_compatibility_manifests, get_compatibility_manifest_by_manifest_id])
        .build(tauri::generate_context!())
        .expect("Error while running KeqingLauncher!");

    builder.run(|app, event| {
        match &event {
            RunEvent::Exit => {
                run_async_command(async {
                    app.state::<DbInstances>().0.lock().await.get("db").unwrap().close().await;
                });
            }
            _ => ()
        }
    })
}
