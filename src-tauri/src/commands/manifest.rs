use crate::utils::LinkedHashMap;
use tauri::{AppHandle,Manager};
use crate::utils::db_manager::{get_manifest_info_by_filename,get_manifest_info_by_id,get_manifests_by_repository_id,update_manifest_enabled_by_id};
use crate::utils::repo_manager::{get_manifest,get_manifests,ManifestLoaders};
use crate::utils::models::{GameManifest};

#[cfg(target_os = "linux")]
use crate::utils::repo_manager::{get_compatibilities,get_compatibility};
#[cfg(target_os = "linux")]
use crate::utils::models::{RunnerManifest};

#[tauri::command]
pub fn get_manifest_by_id(app: AppHandle, id: String) -> Option<String> {
    let manifest = get_manifest_info_by_id(&app, id);

    if manifest.is_some() {
        let m = manifest.unwrap();
        let stringified = serde_json::to_string(&m).unwrap();
        Some(stringified)
    } else {
        None
    }
}

#[tauri::command]
pub fn get_manifest_by_filename(app: AppHandle, filename: String) -> Option<String> {
    let manifest = get_manifest_info_by_filename(&app, filename);

    if manifest.is_some() {
        let m = manifest.unwrap();
        let stringified = serde_json::to_string(&m).unwrap();
        Some(stringified)
    } else {
        None
    }
}

#[tauri::command]
pub fn list_manifests_by_repository_id(app: AppHandle, repository_id: String) -> Option<String> {
    let manifests = get_manifests_by_repository_id(&app, repository_id);

    if manifests.is_some() {
        let manifest = manifests.unwrap();
        let stringified = serde_json::to_string(&manifest).unwrap();
        Some(stringified)
    } else {
        None
    }
}

#[tauri::command]
pub fn list_game_manifests(app: AppHandle) -> Option<String> {
    let manifestss: LinkedHashMap<String, GameManifest> = get_manifests(&app);
    let mut manifests: Vec<GameManifest> = Vec::new();

    for value in manifestss.clone().into_iter().map(|(_, value)| value) { manifests.push(value); }

    if manifests.is_empty() {
        None
    } else {
        let stringified = serde_json::to_string(&manifests).unwrap();
        Some(stringified)
    }
}

#[tauri::command]
pub fn get_game_manifest_by_filename(app: AppHandle, filename: String) -> Option<String> {
    let manifest = get_manifest(&app, filename.clone());
    let db_manifest = get_manifest_info_by_filename(&app, filename.clone());

    if manifest.is_some() && db_manifest.is_some() {
        let dbm = db_manifest.unwrap();

        if dbm.enabled {
            let m = manifest.unwrap();
            let stringified = serde_json::to_string(&m).unwrap();
            Some(stringified)
        } else {
            None
        }
    } else {
        None
    }
}

#[tauri::command]
pub fn get_game_manifest_by_manifest_id(app: AppHandle, id: String) -> Option<String> {
    let db_manifest = get_manifest_info_by_id(&app, id.clone());

    if db_manifest.is_some() {
        let dbm = db_manifest.unwrap();
        let manifest = get_manifest(&app, dbm.filename);

        if dbm.enabled {
            let m = manifest.unwrap();
            let stringified = serde_json::to_string(&m).unwrap();
            Some(stringified)
        } else {
            None
        }
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
pub fn list_compatibility_manifests(app: AppHandle, biz: Option<String>) -> Option<String> {
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

    if manifests.is_empty() { None } else { let stringified = serde_json::to_string(&manifests).unwrap(); Some(stringified) }
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub fn list_compatibility_manifests(_app: AppHandle, _biz: Option<String>) -> Option<String> { None }

#[cfg(target_os = "linux")]
#[tauri::command]
pub fn get_compatibility_manifest_by_manifest_id(app: AppHandle, id: String) -> Option<String> {
    let db_manifest = get_manifest_info_by_id(&app, id.clone());

    if db_manifest.is_some() {
        let dbm = db_manifest.unwrap();
        let manifest = get_compatibility(&app, &dbm.filename);

        if dbm.enabled {
            #[allow(unused_mut)] let mut m = manifest.unwrap();
            #[cfg(target_arch = "aarch64")]
            { if !m.aarch64_supported { return None; } m.versions = m.versions.into_iter().filter(|v| v.urls.as_ref().map(|u| !u.aarch64.is_empty()).unwrap_or(false)).collect(); }
            let stringified = serde_json::to_string(&m).unwrap();
            Some(stringified)
        } else {
            None
        }
    } else {
        None
    }
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub fn get_compatibility_manifest_by_manifest_id(_app: AppHandle, _id: String) -> Option<String> { None }

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