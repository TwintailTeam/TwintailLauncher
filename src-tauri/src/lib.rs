#![feature(async_closure)]

use tauri::async_runtime::block_on;
use tauri::ipc::IpcResponse;
use crate::commands::manifest::{get_manifest_by_filename, get_manifest_by_id, list_manifests_by_repository_id};
use crate::commands::repository::list_repositories;
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
        .invoke_handler(tauri::generate_handler![list_repositories, get_manifest_by_id, get_manifest_by_filename, list_manifests_by_repository_id])
        .run(tauri::generate_context!())
        .expect("Error while running KeqingLauncher!");
}
