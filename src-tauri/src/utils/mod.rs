pub mod db_manager;
pub mod repo_manager;

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