#![feature(async_closure)]

use tauri::async_runtime::block_on;
use tauri::ipc::IpcResponse;
use crate::commands::install::{add_install, get_install_by_id, list_installs, list_installs_by_manifest_id, remove_install};
use crate::commands::manifest::{get_manifest_by_filename, get_manifest_by_id, list_game_manifests, get_game_manifest_by_filename, list_manifests_by_repository_id};
use crate::commands::repository::{list_repositories, remove_repository, add_repository, get_repository};
use crate::utils::db_manager::init_db;
use crate::utils::repo_manager::load_manifests;

mod utils;
mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle();

            block_on(init_db(handle)).body().unwrap();

            load_manifests(&handle);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![remove_repository, add_repository, get_repository, list_repositories,
            get_manifest_by_id, get_manifest_by_filename, list_manifests_by_repository_id,
            get_game_manifest_by_filename, list_game_manifests,
            list_installs, list_installs_by_manifest_id, get_install_by_id, add_install, remove_install])
        .run(tauri::generate_context!())
        .expect("Error while running KeqingLauncher!");
}
