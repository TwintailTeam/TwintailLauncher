use crate::utils::LinkedHashMap;
use tauri::{AppHandle,Manager};
use crate::utils::db_manager::{get_manifest_info_by_filename,get_manifest_info_by_id,get_manifests_by_repository_id,update_manifest_enabled_by_id};
use crate::utils::repo_manager::{get_manifest,get_manifests,ManifestLoaders};
use crate::utils::models::{GameManifest,LauncherManifest};

use crate::utils::models::{RunnerManifest};
#[cfg(target_os = "linux")]
use crate::utils::repo_manager::{get_compatibilities,get_compatibility};

#[tauri::command]
pub fn get_manifest_by_id(app: AppHandle, id: String) -> Option<LauncherManifest> {
    get_manifest_info_by_id(&app, id)
}

#[tauri::command]
pub fn get_manifest_by_filename(app: AppHandle, filename: String) -> Option<LauncherManifest> {
    get_manifest_info_by_filename(&app, filename)
}

#[tauri::command]
pub fn list_manifests_by_repository_id(app: AppHandle, repository_id: String) -> Option<Vec<LauncherManifest>> {
    get_manifests_by_repository_id(&app, repository_id)
}

#[tauri::command]
pub fn list_game_manifests(app: AppHandle) -> Option<Vec<GameManifest>> {
    let manifestss: LinkedHashMap<String, GameManifest> = get_manifests(&app);
    let mut manifests: Vec<GameManifest> = Vec::new();

    for value in manifestss.clone().into_iter().map(|(_, value)| value) { manifests.push(value); }

    if manifests.is_empty() { None } else { Some(manifests) }
}

#[tauri::command]
pub fn get_game_manifest_by_filename(app: AppHandle, filename: String) -> Option<GameManifest> {
    let manifest = get_manifest(&app, filename.clone());
    let db_manifest = get_manifest_info_by_filename(&app, filename.clone());

    if manifest.is_some() && db_manifest.is_some() {
        let dbm = db_manifest.unwrap();

        if dbm.enabled { manifest } else { None }
    } else {
        None
    }
}

#[tauri::command]
pub fn get_game_manifest_by_manifest_id(app: AppHandle, id: String) -> Option<GameManifest> {
    let db_manifest = get_manifest_info_by_id(&app, id.clone());

    if db_manifest.is_some() {
        let dbm = db_manifest.unwrap();
        let manifest = get_manifest(&app, dbm.filename);

        if dbm.enabled { manifest } else { None }
    } else {
        None
    }
}

#[tauri::command]
pub fn update_manifest_enabled(app: AppHandle, id: String, enabled: bool) -> Option<bool> {
    let manifest = get_manifest_info_by_id(&app, id);

    if manifest.is_some() {
        let m = manifest.unwrap();
        update_manifest_enabled_by_id(&app, m.id, enabled);
        Some(true)
    } else {
        None
    }
}

#[cfg(target_os = "linux")]
#[tauri::command]
pub fn list_compatibility_manifests(app: AppHandle, biz: Option<String>) -> Option<Vec<RunnerManifest>> {
    let manifestss: LinkedHashMap<String, RunnerManifest> = get_compatibilities(&app);
    let mut manifests: Vec<RunnerManifest> = Vec::new();

    let override_version: Option<String> = if let Some(ref b) = biz {
        if let Some(gm) = get_manifest(&app, format!("{}.json", b)) {
            let ovr = &gm.extra.compat_overrides.override_runner.linux;
            if ovr.enabled && !ovr.runner_version.is_empty() { Some(ovr.runner_version.clone()) } else { None }
        } else { None }
    } else { None };

    for value in manifestss.clone().into_iter().map(|(_, value)| value) {
        #[cfg(target_arch = "aarch64")]
        { if !value.aarch64_supported { continue; } }
        #[allow(unused_mut)] let mut v = value;
        #[cfg(target_arch = "aarch64")]
        { v.versions = v.versions.into_iter().filter(|ver| ver.urls.as_ref().map(|u| !u.aarch64.is_empty()).unwrap_or(false)).collect(); }
        if let Some(ref ov) = override_version { v.versions = v.versions.into_iter().filter(|ver| crate::utils::is_using_overriden_runner(ver.version.clone(), ov.clone())).collect(); }
        manifests.push(v);
    }

    if manifests.is_empty() { None } else { Some(manifests) }
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub fn list_compatibility_manifests(_app: AppHandle, _biz: Option<String>) -> Option<Vec<RunnerManifest>> { None }

#[cfg(target_os = "linux")]
#[tauri::command]
pub fn get_compatibility_manifest_by_manifest_id(app: AppHandle, id: String) -> Option<RunnerManifest> {
    let db_manifest = get_manifest_info_by_id(&app, id.clone());

    if db_manifest.is_some() {
        let dbm = db_manifest.unwrap();
        let manifest = get_compatibility(&app, &dbm.filename);

        if dbm.enabled {
            #[allow(unused_mut)] let mut m = manifest.unwrap();
            #[cfg(target_arch = "aarch64")]
            { if !m.aarch64_supported { return None; } m.versions = m.versions.into_iter().filter(|v| v.urls.as_ref().map(|u| !u.aarch64.is_empty()).unwrap_or(false)).collect(); }
            Some(m)
        } else {
            None
        }
    } else {
        None
    }
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub fn get_compatibility_manifest_by_manifest_id(_app: AppHandle, _id: String) -> Option<RunnerManifest> { None }

#[allow(unused_variables)]
#[tauri::command]
pub async fn override_manifest_url(app: AppHandle, filename: String, url: String) -> Option<bool> {
    // Moved reqwest calls fully to fischl so this is dirty disabled
    let text = "";
    let manifest: GameManifest = serde_json::from_str(&text).ok()?;
    let ml = app.state::<ManifestLoaders>();
    let mut loader = ml.game.0.write().unwrap();
    loader.insert(filename, manifest);
    Some(true)
}

#[tauri::command]
pub fn clear_manifest_override(app: AppHandle, filename: String) -> Option<bool> {
    let data_path = app.path().app_data_dir().unwrap();
    let manifests_path = data_path.join("manifests");
    for d in std::fs::read_dir(&manifests_path).ok()? {
        let p = d.ok()?.path();
        if !p.is_dir() { continue; }
        for pp in std::fs::read_dir(&p).ok()? {
            let repo_dir = pp.ok()?.path();
            let target = repo_dir.join(&filename);
            if target.exists() {
                let file = std::fs::File::open(&target).ok()?;
                let reader = std::io::BufReader::new(file);
                let manifest: GameManifest = serde_json::from_reader(reader).ok()?;
                let ml = app.state::<ManifestLoaders>();
                let mut loader = ml.game.0.write().unwrap();
                loader.insert(filename, manifest);
                return Some(true);
            }
        }
    }
    None
}
