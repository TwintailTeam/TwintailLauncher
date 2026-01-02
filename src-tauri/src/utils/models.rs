use serde::{Deserialize, Serialize};
use sqlx::types::Json;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct XXMISettings {
    pub hunting_mode: u64,
    pub require_admin: bool,
    pub dll_init_delay: u64,
    pub close_delay: u64,
    pub show_warnings: u64,
    pub dump_shaders: bool
}

// === DATABASE ===

#[derive(Serialize, Deserialize, Debug)]
pub struct GlobalSettings {
    pub default_game_path: String,
    pub xxmi_path: String,
    pub fps_unlock_path: String,
    pub jadeite_path: String,
    pub third_party_repo_updates: i32,
    pub default_runner_prefix_path: String,
    pub launcher_action: String,
    pub hide_manifests: bool,
    pub default_runner_path: String,
    pub default_dxvk_path: String,
    pub default_mangohud_config_path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RepositoryManifest {
    pub name: String,
    pub description: String,
    pub maintainers: Vec<String>,
    pub manifests: Vec<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LauncherRepository {
    pub id: String,
    pub github_id: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LauncherManifest {
    pub id: String,
    pub repository_id: String,
    pub display_name: String,
    pub filename: String,
    pub enabled: bool
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LauncherInstall {
    pub id: String,
    pub manifest_id: String,
    pub version: String,
    pub audio_langs: String,
    pub name: String,
    pub directory: String,
    pub runner_path: String,
    pub dxvk_path: String,
    pub runner_version: String,
    pub dxvk_version: String,
    pub game_icon: String,
    pub game_background: String,
    pub ignore_updates: bool,
    pub skip_hash_check: bool,
    pub use_jadeite: bool,
    pub use_xxmi: bool,
    pub use_fps_unlock: bool,
    pub env_vars: String,
    pub pre_launch_command: String,
    pub launch_command: String,
    pub fps_value: String,
    pub runner_prefix: String,
    pub launch_args: String,
    pub use_gamemode: bool,
    pub use_mangohud: bool,
    pub mangohud_config_path: String,
    pub shortcut_is_steam: bool,
    pub shortcut_path: String,
    pub region_code: String,
    pub xxmi_config: Json<XXMISettings>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LauncherRunner {
    pub id: i64,
    pub runner_path: String,
    pub is_installed: bool,
    pub version: String,
    pub value: String,
    pub name: String
}

// === STRUCTS FOR MANIFESTS ===

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RunnerManifest {
    pub version: i32,
    pub display_name: String,
    pub versions: Vec<RunnerVersion>,
    pub paths: RunnerPaths
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RunnerPlatformUrls {
    pub x86_64: String,
    pub aarch64: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RunnerVersion {
    pub version: String,
    pub url: String,
    pub urls: Option<RunnerPlatformUrls>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RunnerPaths {
    pub wine32: String,
    pub wine64: String,
    pub wine_server: String,
    pub wine_boot: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameManifest {
    pub version: i32,
    pub display_name: String,
    pub biz: String,
    pub latest_version: String,
    pub game_versions: Vec<GameVersion>,
    pub telemetry_hosts: Vec<String>,
    pub paths: GamePaths,
    pub assets: VersionAssets,
    pub extra: GameExtras
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameVersion {
    pub metadata: VersionMetadata,
    pub assets: VersionAssets,
    pub game: VersionGameFiles,
    pub audio: VersionAudioFiles
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GamePaths {
    pub audio_pkg_res_dir: String,
    pub exe_filename: String,
    pub installation_dir: String,
    pub screenshot_dir: String,
    pub screenshot_dir_relative_to: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VersionMetadata {
    pub versioned_name: String,
    pub version: String,
    pub download_mode: String,
    pub game_hash: String,
    pub index_file: String,
    pub res_list_url: String,
    pub diff_list_url: DiffUrls
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DiffUrls {
    pub game: String,
    pub en_us: String,
    pub zh_cn: String,
    pub ja_jp: String,
    pub ko_kr: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VersionAssets {
    pub game_icon: String,
    pub game_background: String,
    pub game_live_background: Option<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VersionGameFiles {
    pub full: Vec<FullGameFile>,
    pub diff: Vec<DiffGameFile>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FullGameFile {
    pub file_url: String,
    pub compressed_size: String,
    pub decompressed_size: String,
    pub file_hash: String,
    pub file_path: String,
    pub region_code: Option<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DiffGameFile {
    pub file_url: String,
    pub compressed_size: String,
    pub decompressed_size: String,
    pub file_hash: String,
    pub diff_type: String,
    pub original_version: String,
    pub delete_files: Vec<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VersionAudioFiles {
    pub full: Vec<FullAudioFile>,
    pub diff: Vec<DiffAudioFile>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FullAudioFile {
    pub file_url: String,
    pub compressed_size: String,
    pub decompressed_size: String,
    pub file_hash: String,
    pub language: String,
    pub region_code: Option<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DiffAudioFile {
    pub file_url: String,
    pub compressed_size: String,
    pub decompressed_size: String,
    pub file_hash: String,
    pub diff_type: String,
    pub original_version: String,
    pub language: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GamePreload {
    pub metadata: Option<VersionMetadata>,
    pub index_file: Option<String>,
    pub res_list_url: Option<String>,
    pub game: Option<VersionGameFiles>,
    pub audio: Option<VersionAudioFiles>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameTweakSwitches {
    pub fps_unlocker: bool,
    pub jadeite: bool,
    pub xxmi: bool
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CompatRunnerOverrides {
    pub enabled: bool,
    pub runner_version: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CompatPlatformOverrides {
    pub linux: CompatRunnerOverrides,
    pub macos: CompatRunnerOverrides
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameCompatOverrides {
    pub install_to_prefix: bool,
    pub disable_protonfixes: bool,
    pub protonfixes_id: String,
    pub protonfixes_store: String,
    pub stub_wintrust: bool,
    pub block_first_req: bool,
    pub proton_compat_config: Vec<String>,
    pub override_runner: CompatPlatformOverrides,
    pub min_runner_versions: Vec<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameExtras {
    pub preload: Option<GamePreload>,
    pub switches: GameTweakSwitches,
    pub compat_overrides: GameCompatOverrides,
    pub fps_unlock_options: Vec<String>,
}