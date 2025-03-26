
pub mod db_manager;
pub mod repo_manager;
mod git_helpers;
pub mod game_launch_manager;

pub fn generate_cuid() -> String {
    cuid2::create_id()
}

/// Allows blocking on async code without creating a nested runtime.
pub fn run_async_command<F: std::future::Future>(cmd: F) -> F::Output {
    if tokio::runtime::Handle::try_current().is_ok() {
        tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(cmd))
    } else {
        tauri::async_runtime::block_on(cmd)
    }
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