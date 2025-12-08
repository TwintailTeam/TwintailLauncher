use std::collections::HashMap;
use std::fs;
use std::ops::Add;
use std::path::Path;
use std::sync::Arc;
use fischl::utils::{prettify_bytes};
use fischl::utils::free_space::available;
use tauri::{AppHandle, Emitter, Manager};
use crate::utils::db_manager::{create_installation, delete_installation_by_id, get_install_info_by_id, get_installs, get_installs_by_manifest_id, get_manifest_info_by_filename, get_manifest_info_by_id, get_settings, update_install_env_vars_by_id, update_install_fps_value_by_id, update_install_game_location_by_id, update_install_ignore_updates_by_id, update_install_launch_args_by_id, update_install_launch_cmd_by_id, update_install_mangohud_config_location_by_id, update_install_pre_launch_cmd_by_id, update_install_prefix_location_by_id, update_install_shortcut_location_by_id, update_install_skip_hash_check_by_id, update_install_use_fps_unlock_by_id, update_install_use_gamemode_by_id, update_install_use_jadeite_by_id, update_install_use_mangohud_by_id, update_install_use_xxmi_by_id};
// --- START MODIFICATION ---
// Removed 'launch' as we are moving the execution logic out of this file.
// use crate::utils::game_launch_manager::launch; 
// --- END MODIFICATION ---
use crate::utils::{copy_dir_all, download_or_update_fps_unlock, download_or_update_jadeite, download_or_update_xxmi, generate_cuid, prevent_exit, send_notification, AddInstallRsp, DownloadSizesRsp, PathResolve, ResumeStatesRsp};
use crate::utils::repo_manager::{get_manifest, GameVersion};
use crate::utils::shortcuts::{remove_desktop_shortcut};

#[cfg(target_os = "linux")]
use crate::utils::runner_from_runner_version;
#[cfg(target_os = "linux")]
use fischl::compat::Compat;
#[cfg(target_os = "linux")]
use crate::utils::repo_manager::get_compatibility;
#[cfg(target_os = "linux")]
use fischl::utils::patch_aki;
#[cfg(target_os = "linux")]
use crate::utils::is_flatpak;
#[cfg(target_os = "linux")]
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(target_os = "linux")]
use crate::utils::shortcuts::{add_steam_shortcut, remove_steam_shortcut};
#[cfg(target_os = "linux")]
use steam_shortcuts_util::app_id_generator::calculate_app_id;
#[cfg(target_os = "linux")]
use steam_shortcuts_util::Shortcut;
#[cfg(target_os = "linux")]
use crate::utils::db_manager::update_install_shortcut_is_steam_by_id;
#[cfg(target_os = "linux")]
use crate::utils::shortcuts::add_desktop_shortcut;

#[tauri::command]
pub async fn list_installs(app: AppHandle) -> Option<String> {
// ... (All existing functions above the deleted game_launch command remain unchanged)
// ...
// ... (The original game_launch command block is entirely deleted here)
// ...
#[tauri::command]
pub fn get_download_sizes(app: AppHandle, biz: String, version: String, lang: String, path: String) -> Option<String> {
// ... (All existing functions below the deleted game_launch command remain unchanged)
