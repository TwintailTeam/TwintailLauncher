use std::{fs, io};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Listener, Manager};

pub mod db_manager;
pub mod repo_manager;
mod git_helpers;
pub mod game_launch_manager;
pub mod system_tray;

pub fn generate_cuid() -> String {
    cuid2::create_id()
}

pub fn run_async_command<F: Future>(cmd: F) -> F::Output {
    if tokio::runtime::Handle::try_current().is_ok() {
        tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(cmd))
    } else {
        tauri::async_runtime::block_on(cmd)
    }
}

pub fn copy_dir_all(app: &AppHandle, src: impl AsRef<Path>, dst: impl AsRef<Path>, install: String, install_name: String, install_type: String) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    let mut payload = HashMap::new();

    for entry in fs::read_dir(src.as_ref())? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let f = entry.file_name();

        payload.insert("file", f.to_str().unwrap().to_string());
        payload.insert("install_id", install.clone());
        payload.insert("install_name", install_name.clone());
        payload.insert("install_type", install_type.clone());

        app.emit("move_progress", &payload).unwrap();

        if ty.is_dir() {
            copy_dir_all(&app, entry.path(), dst.as_ref().join(entry.file_name()), install.clone(), install_name.clone(), install_type.clone())?;
            fs::remove_dir_all(entry.path())?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
            fs::remove_file(entry.path())?;
        }
    }
    Ok(())
}

pub fn register_listeners(app: &AppHandle) {
    let h1 = app.clone();
    app.listen("launcher_action_exit", move |_event| {
        let blocks = h1.state::<Mutex<ActionBlocks>>();
        let state = blocks.lock().unwrap();

        if state.action_exit {
            h1.get_window("main").unwrap().hide().unwrap();
        } else {
            h1.cleanup_before_exit();
            h1.exit(0);
            std::process::exit(0);
        }
    });

    let h2 = app.clone();
    app.listen("launcher_action_minimize", move |_event| {
        h2.get_window("main").unwrap().hide().unwrap();
    });

    let h3 = app.clone();
    app.listen("prevent_exit", move |event| {
        let blocks = h3.state::<Mutex<ActionBlocks>>();
        let mut state = blocks.lock().unwrap();

        if event.payload().parse::<bool>().unwrap() == true {
            state.action_exit = true;
            drop(state);
        } else {
            state.action_exit = false;
            drop(state);
        }
    });
}

#[cfg(target_os = "linux")]
pub fn runner_from_runner_version(runner_version: String) -> Option<String> {
    let mut rslt = String::new();

    if runner_version.is_empty() {
        None
    } else {
        if runner_version.contains("vanilla") {
            rslt = "dxvk_vanilla.json".to_string();
        }
        if runner_version.contains("async") {
            rslt = "dxvk_async.json".to_string();
        }
        if runner_version.contains("gplasync") {
            rslt = "dxvk_gplasync.json".to_string();
        }
        if runner_version.contains("wine-vanilla") {
            rslt = "wine_vanilla.json".to_string();
        }
        if runner_version.contains("wine-staging") {
            rslt = "wine_staging.json".to_string();
        }
        if runner_version.contains("wine-staging-tkg") {
            rslt = "wine_staging_tkg.json".to_string();
        }
        if runner_version.contains("wine-vaniglia") {
            rslt = "wine_vaniglia.json".to_string();
        }
        if runner_version.contains("wine-soda") {
            rslt = "wine_soda.json".to_string();
        }
        if runner_version.contains("wine-lutris") {
            rslt = "wine_lutris.json".to_string();
        }
        if runner_version.contains("wine-ge-proton") {
            rslt = "wine_ge_proton.json".to_string();
        }
        if runner_version.contains("wine-caffe") {
            rslt = "wine_caffe.json".to_string();
        }
        if runner_version.contains("proton-ge") {
            rslt = "proton_ge.json".to_string();
        }
        if runner_version.contains("proton-cachyos") {
            rslt = "proton_cachyos.json".to_string();
        }
        Some(rslt)
    }
}

#[cfg(target_os = "windows")]
pub fn runner_from_runner_version(runner_version: String) -> Option<String> {
    None
}

pub struct ActionBlocks {
    pub action_exit: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddInstallRsp {
    pub success: bool,
    pub install_id: String,
    pub background: String,
}