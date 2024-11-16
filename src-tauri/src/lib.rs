#![feature(async_closure)]

use tauri::async_runtime::block_on;
use tauri::ipc::IpcResponse;
use crate::utils::db_manager::init_db;

mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle();

            block_on(init_db(handle)).body().unwrap();

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("Error while running KeqingLauncher!");
}
