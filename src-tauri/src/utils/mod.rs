use crate::utils::db_manager::{get_installs, get_manifest_info_by_id, get_settings, update_install_after_update_by_id, update_settings_default_fps_unlock_location, update_settings_default_game_location, update_settings_default_xxmi_location};
#[cfg(target_os = "linux")]
use crate::utils::db_manager::{
    create_installed_runner, get_installed_runner_info_by_version, get_installed_runners,
    update_install_use_jadeite_by_id, update_installed_runner_is_installed_by_version,
    update_settings_default_dxvk_location, update_settings_default_jadeite_location,
    update_settings_default_prefix_location, update_settings_default_runner_location,
};
use crate::utils::models::{DialogResponse,XXMISettings};
use crate::utils::repo_manager::get_manifest;
use fischl::utils::get_github_release;
use sqlx::types::Json;
use std::collections::HashMap;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc};
use std::{fs, io};
use std::hash::Hash;
use tauri::{AppHandle, Emitter, Listener, Manager};

#[cfg(target_os = "linux")]
use crate::utils::repo_manager::{get_compatibility, get_compatibilities};
#[cfg(target_os = "linux")]
use crate::DownloadState;
#[cfg(target_os = "linux")]
use crate::downloading::{QueueJobPayload, RunnerDownloadPayload, queue::QueueJobKind};

pub mod args;
pub mod db_manager;
pub mod game_launch_manager;
mod git_helpers;
#[cfg(target_os = "linux")]
pub mod gpu;
pub mod models;
pub mod repo_manager;
pub mod shortcuts;
pub mod system_tray;
pub mod discord_rpc;

pub fn generate_cuid() -> String {
    cuid2::create_id()
}

pub fn run_async_command<F: Future>(cmd: F) -> F::Output {
    if tokio::runtime::Handle::try_current().is_ok() { tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(cmd)) } else { tauri::async_runtime::block_on(cmd) }
}

pub fn copy_dir_all(app: &AppHandle, src: impl AsRef<Path>, dst: impl AsRef<Path>, install: String, install_name: String, install_type: String) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    let totalsize = dir_size(src.as_ref())?;
    let tracker = Arc::new(AtomicU64::new(0));
    let mut files_to_remove = Vec::new();

    for entry in fs::read_dir(src.as_ref())? {
        let entry = entry?;
        let f = entry.file_name();
        let ep = entry.path();
        if ep == dst.as_ref() { continue; }

        let meta = fs::symlink_metadata(ep.clone())?;
        if meta.file_type().is_symlink() {
            let target_path = fs::read_link(&ep)?;
            #[cfg(target_os = "linux")]
            std::os::unix::fs::symlink(target_path, dst.as_ref().join(&f))?;
            #[cfg(target_os = "windows")]
            if target_path.is_dir() { std::os::windows::fs::symlink_dir(target_path, dst.as_ref())?; } else { std::os::windows::fs::symlink_file(target_path, dst.as_ref().join(&f))?; }
            files_to_remove.push(ep.clone());
        } else if meta.is_dir() {
            copy_dir_all(&app, ep.clone(), dst.as_ref().join(f), install.clone(), install_name.clone(), install_type.clone())?;
            files_to_remove.push(ep.clone());
        } else {
            let size = entry.metadata()?.len();
            let cur = tracker.fetch_add(size, Ordering::SeqCst) + size;

            let mut payload = HashMap::new();
            payload.insert("file", f.to_str().unwrap().to_string());
            payload.insert("install_id", install.clone());
            payload.insert("install_name", install_name.clone());
            payload.insert("install_type", install_type.clone());
            payload.insert("phase", "5".to_string());
            payload.insert("install_progress", cur.to_string());
            payload.insert("install_total", totalsize.to_string());

            if let Err(e) = fs::copy(ep.clone(), dst.as_ref().join(f)) { eprintln!("Failed to copy {}: {}", ep.clone().display(), e); }
            app.emit("move_progress", &payload).unwrap();
            files_to_remove.push(ep.clone());
        }
    }
    for file_path in files_to_remove {
        if file_path.is_file() || file_path.is_symlink() { if let Err(e) = fs::remove_file(file_path) { eprintln!("Failed to remove file: {}", e); } } else { if let Err(e) = fs::remove_dir_all(file_path) { eprintln!("Failed to remove directory: {}", e); } }
    }
    Ok(())
}

pub fn register_listeners(app: &AppHandle) {
    let h1 = app.clone();
    app.listen("launcher_action_exit", move |_event| {
        h1.get_window("main").unwrap().hide().unwrap();
        h1.cleanup_before_exit();
        h1.exit(0);
        std::process::exit(0);
    });
    let h2 = app.clone();
    app.listen("launcher_action_minimize", move |_event| { h2.get_window("main").unwrap().minimize().unwrap(); });

    let h3 = app.clone();
    app.listen_any("dialog_response", move |event| {
        if let Ok(response) = serde_json::from_str::<DialogResponse>(event.payload()) {
            match response.callback_id.as_str() {
                "dialog_steamrt3_dl_fail" => {
                    let gs = get_settings(&h3).unwrap();
                    let runnerp = Path::new(gs.default_runner_path.as_str()).to_path_buf();
                    let steamrtpp = runnerp.join("steamrt/").join("steamrt3/");
                    let _ = empty_dir(&steamrtpp);
                }
                "dialog_steamrt4_dl_fail" => {
                    let gs = get_settings(&h3).unwrap();
                    let runnerp = Path::new(gs.default_runner_path.as_str()).to_path_buf();
                    let steamrtpp = runnerp.join("steamrt/").join("steamrt4/");
                    let _ = empty_dir(&steamrtpp);
                }
                "dialog_runner_dl_fail" => { /* Empties the directory in its handler not here */ }
                "dialog_extra_dl_fail" => { /* Handled in respective failure blocks */}
                _ => {}
            }
        }
    });
}

pub fn show_dialog_with_callback(app: &AppHandle, dialog_type: &str, title: &str, message: &str, buttons: Option<Vec<&str>>, callback_id: Option<&str>) {
    #[derive(serde::Serialize, Clone)]
    struct DialogPayload { dialog_type: String, title: String, message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        buttons: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        callback_id: Option<String>,
    }
    let payload = DialogPayload { dialog_type: dialog_type.to_string(), title: title.to_string(), message: message.to_string(), buttons: buttons.map(|btns| btns.into_iter().map(|b| b.to_string()).collect::<Vec<String>>()), callback_id: callback_id.map(|id| id.to_string()), };
    app.emit("show_dialog", payload).unwrap();
}

#[cfg(target_os = "linux")]
pub fn runner_from_runner_version(app: &AppHandle, runner_version: String) -> Option<String> {
    if runner_version.is_empty() { return None; }
    let loader = get_compatibilities(app);
    for (filename, manifest) in &loader { if manifest.versions.iter().any(|v| v.version == runner_version) { return Some(filename.clone()); } }
    let rl = runner_version.to_lowercase();
    let rslt = if rl.contains("proton-cachyos") { "proton_cachyos.json" }
        else if rl.contains("proton-ge") || rl.contains("ge-proton") { "proton_ge.json" }
        else if rl.contains("proton-twintail") { "proton_twintail.json" }
        else if rl.contains("proton-umu") { "proton_umu.json" }
        else if rl.contains("proton-vanilla") { "proton_vanilla.json" }
        else if rl.contains("proton-em") { "proton_em.json" }
        else if rl.contains("wine-staging-tkg") { "wine_staging_tkg.json" }
        else if rl.contains("wine-staging") { "wine_staging.json" }
        else if rl.contains("wine-ge-proton") { "wine_ge_proton.json" }
        else if rl.contains("wine-vaniglia") { "wine_vaniglia.json" }
        else if rl.contains("wine-vanilla") { "wine_vanilla.json" }
        else if rl.contains("wine-soda") { "wine_soda.json" }
        else if rl.contains("wine-lutris") { "wine_lutris.json" }
        else if rl.contains("wine-caffe") { "wine_caffe.json" }
        else if rl.contains("gplasync") { "dxvk_gplasync.json" }
        else if rl.contains("async") { "dxvk_async.json" }
        else if rl.contains("vanilla") { "dxvk_vanilla.json" }
        else { "proton_cachyos.json" };
    Some(rslt.to_string())
}

pub fn get_mi_path_from_game(exe_name: String) -> Option<String> {
    if exe_name.is_empty() {
        None
    } else {
        let exe = exe_name.split('/').last().unwrap().to_string();
        match exe.to_ascii_lowercase().as_str() {
            "genshinimpact.exe" => Some("gimi".parse().unwrap()),
            "starrail.exe" => Some("srmi".parse().unwrap()),
            "zenlesszonezero.exe" => Some("zzmi".parse().unwrap()),
            "bh3.exe" => Some("himi".parse().unwrap()),
            "client-win64-shipping.exe" => Some("wwmi".parse().unwrap()),
            "endfield.exe" => Some("efmi".parse().unwrap()),
            "stellasora.exe" => Some("ssmi".parse().unwrap()),
            _ => None,
        }
    }
}

fn dir_size(path: &Path) -> io::Result<u64> {
    let mut size = 0;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_dir() { size += dir_size(&entry.path())?; } else { size += metadata.len(); }
    }
    Ok(size)
}

pub fn setup_or_fix_default_paths(app: &AppHandle, path: PathBuf, fix_mode: bool) {
    let defgpath = path.join("games");
    let xxmipath = path.join("extras").join("xxmi");
    let fpsunlockpath = path.join("extras").join("fps_unlock");

    if fix_mode {
        // Fix empty db entries and remake dirs
        let gs = get_settings(app);
        if gs.is_some() {
            let g = gs.unwrap();
            if g.default_game_path == "" { fs::create_dir_all(&defgpath).unwrap(); update_settings_default_game_location(app, defgpath.to_str().unwrap().to_string()); }
            if g.xxmi_path == "" { fs::create_dir_all(&xxmipath).unwrap(); update_settings_default_xxmi_location(app, xxmipath.to_str().unwrap().to_string()); }
            if g.fps_unlock_path == "" { fs::create_dir_all(&fpsunlockpath).unwrap();update_settings_default_fps_unlock_location(app, fpsunlockpath.to_str().unwrap().to_string()); }

            #[cfg(target_os = "linux")]
            {
                let comppath = path.join("compatibility");
                let wine = comppath.join("runners");
                let dxvk = comppath.join("dxvk");
                let prefixes = comppath.join("prefixes");
                let jadeitepath = path.join("extras").join("jadeite");
                let mangohudcfg = app.path().home_dir().unwrap().join(".config/MangoHud/MangoHud.conf");

                // steamrt setup
                let steamrtpath = wine.join("steamrt");
                let steamrt3 = steamrtpath.join("steamrt3");
                let steamrt4 = steamrtpath.join("steamrt4");
                // Ensure base steamrt folder exists, then create versioned subfolders
                if !steamrtpath.exists() { fs::create_dir_all(&steamrtpath).unwrap(); }
                if !steamrt3.exists() { fs::create_dir_all(&steamrt3).unwrap(); }
                if !steamrt4.exists() { fs::create_dir_all(&steamrt4).unwrap(); }

                // steamrt migration code
                for entry in fs::read_dir(&steamrtpath).unwrap().flatten() {
                    let name = entry.file_name().into_string().unwrap();
                    if name != "steamrt3" && name != "steamrt4" {
                        let p = entry.path();
                        (if p.is_dir() { fs::remove_dir_all } else { fs::remove_file as fn(_) -> _ })(&p).ok();
                    }
                }

                if g.jadeite_path == "" { fs::create_dir_all(&jadeitepath).unwrap(); update_settings_default_jadeite_location(app, jadeitepath.to_str().unwrap().to_string()); }
                if g.default_runner_path == "" { fs::create_dir_all(&wine).unwrap(); update_settings_default_runner_location(app, wine.to_str().unwrap().to_string()); }
                if g.default_dxvk_path == "" { fs::create_dir_all(&dxvk).unwrap();update_settings_default_dxvk_location(app, dxvk.to_str().unwrap().to_string()); }
                if g.default_runner_prefix_path == "" { fs::create_dir_all(&prefixes).unwrap(); update_settings_default_prefix_location(app, prefixes.to_str().unwrap().to_string()); }
                if g.default_mangohud_config_path == "" { db_manager::update_settings_default_mangohud_config_location(app, mangohudcfg.to_str().unwrap().to_string()); }
            }
        }
    } else {
        if !defgpath.exists() { fs::create_dir_all(&defgpath).unwrap(); update_settings_default_game_location(app, defgpath.to_str().unwrap().to_string()); }
        if !xxmipath.exists() { fs::create_dir_all(&xxmipath).unwrap(); update_settings_default_xxmi_location(app, xxmipath.to_str().unwrap().to_string()); }
        if !fpsunlockpath.exists() { fs::create_dir_all(&fpsunlockpath).unwrap(); update_settings_default_fps_unlock_location(app, fpsunlockpath.to_str().unwrap().to_string()); }
        #[cfg(target_os = "linux")]
        {
            let comppath = path.join("compatibility");
            let wine = comppath.join("runners");
            let dxvk = comppath.join("dxvk");
            let prefixes = comppath.join("prefixes");
            let jadeitepath = path.join("extras").join("jadeite");
            let mangohudcfg = app.path().home_dir().unwrap().join(".config/MangoHud/MangoHud.conf");

            // steamrt setup
            let steamrtpath = wine.join("steamrt");
            let steamrt3 = steamrtpath.join("steamrt3");
            let steamrt4 = steamrtpath.join("steamrt4");
            // Ensure base steamrt folder exists, then create versioned subfolders
            if !steamrtpath.exists() { fs::create_dir_all(&steamrtpath).unwrap(); }
            if !steamrt3.exists() { fs::create_dir_all(&steamrt3).unwrap(); }
            if !steamrt4.exists() { fs::create_dir_all(&steamrt4).unwrap(); }

            // steamrt migration code
            for entry in fs::read_dir(&steamrtpath).unwrap().flatten() {
                let name = entry.file_name().into_string().unwrap();
                if name != "steamrt3" && name != "steamrt4" {
                    let p = entry.path();
                    (if p.is_dir() { fs::remove_dir_all } else { fs::remove_file as fn(_) -> _ })(&p).ok();
                }
            }

            if !mangohudcfg.exists() { db_manager::update_settings_default_mangohud_config_location(app, mangohudcfg.to_str().unwrap().to_string()); } else { db_manager::update_settings_default_mangohud_config_location(app, mangohudcfg.to_str().unwrap().to_string()); }
            if !jadeitepath.exists() { fs::create_dir_all(&jadeitepath).unwrap();update_settings_default_jadeite_location(app, jadeitepath.to_str().unwrap().to_string()); }
            if !comppath.exists() {
                fs::create_dir_all(&wine).unwrap();
                fs::create_dir_all(&dxvk).unwrap();
                fs::create_dir_all(&prefixes).unwrap();
                fs::create_dir_all(&steamrtpath).unwrap();
                fs::create_dir_all(&steamrt3).unwrap();
                fs::create_dir_all(&steamrt4).unwrap();
                update_settings_default_runner_location(app, wine.to_str().unwrap().to_string());
                update_settings_default_dxvk_location(app, dxvk.to_str().unwrap().to_string());
                update_settings_default_prefix_location(app, prefixes.to_str().unwrap().to_string());
            }
        }
    }
}

#[cfg(target_os = "linux")]
pub fn sync_installed_runners(app: &AppHandle) {
    let gs = get_settings(app);
    if gs.is_some() {
        let s = gs.unwrap();
        let runners = Path::new(&s.default_runner_path).to_path_buf();
        if !runners.exists() { return; }

        // Mark non-existing ones as uninstalled
        let all_runners = get_installed_runners(app);
        if all_runners.is_some() {
            let ar = all_runners.unwrap();
            for r in ar {
                let dir_path = runners.join(&r.version).to_path_buf();
                if !dir_path.exists() && dir_path.to_str().unwrap().to_string() != "steamrt" { update_installed_runner_is_installed_by_version(app, r.version.clone(), false); }
            }
        }

        for e in fs::read_dir(&runners).unwrap() {
            match e {
                Ok(d) => {
                    let path = d.path();
                    if path.is_dir() && path.exists() {
                        let dir_name = path.file_name().unwrap().to_str().unwrap();
                        let subdir_iter = fs::read_dir(&path);
                        match subdir_iter {
                            Ok(mut subdir) => {
                                if subdir.next().is_some() {
                                    let installed_runner = get_installed_runner_info_by_version(app, dir_name.to_string());
                                    if installed_runner.is_none() && dir_name != "steamrt" { create_installed_runner(app, dir_name.to_string(), true, path.to_str().unwrap().parse().unwrap()).unwrap(); } else if dir_name != "steamrt" { update_installed_runner_is_installed_by_version(app, dir_name.to_string(), true); }
                                }
                            }
                            Err(_) => {}
                        }
                    }
                }
                Err(_) => {}
            }
        }

        // Auto-redownload missing runners that game installs depend on
        let installs = get_installs(app);
        if let Some(insts) = installs {
            let mut queued_versions = std::collections::HashSet::new();
            for inst in &insts {
                let rv = &inst.runner_version;
                if rv.is_empty() || queued_versions.contains(rv) { continue; }

                // Check if this runner is already installed
                let runner_info = get_installed_runner_info_by_version(app, rv.clone());
                let is_installed = runner_info.as_ref().map_or(false, |r| r.is_installed);
                if is_installed { continue; }

                // Resolve the download URL from the compatibility manifest
                let manifest_file = match runner_from_runner_version(app, rv.clone()) { Some(f) if !f.is_empty() => f, _ => continue };
                let compat = match get_compatibility(app, &manifest_file) { Some(c) => c, None => continue };
                let matched: Vec<_> = compat.versions.into_iter().filter(|v| v.version == *rv).collect();
                let runner_ver = match matched.first() { Some(v) => v, None => continue };

                // Determine the download URL based on architecture
                let mut dl_url = runner_ver.url.clone();
                if let Some(ref urls) = runner_ver.urls {
                    #[cfg(target_arch = "x86_64")]
                    { dl_url = urls.x86_64.clone(); }
                    #[cfg(target_arch = "aarch64")]
                    { dl_url = if urls.aarch64.is_empty() { runner_ver.url.clone() } else { urls.aarch64.clone() }; }
                }

                let runner_path = runners.join(rv);
                if !runner_path.exists() { fs::create_dir_all(&runner_path).unwrap(); }
                if runner_info.is_none() { let _ = create_installed_runner(app, rv.clone(), false, runner_path.to_str().unwrap().to_string()); }

                // Enqueue the download
                let state = app.state::<DownloadState>();
                let q = state.queue.lock().unwrap().clone();
                if let Some(queue) = q {
                    queue.enqueue(QueueJobKind::RunnerDownload, QueueJobPayload::Runner(RunnerDownloadPayload {
                        runner_version: rv.clone(),
                        runner_url: dl_url,
                        runner_path: runner_path.to_str().unwrap().to_string(),
                    }));
                    queued_versions.insert(rv.clone());
                    log::debug!("Auto-redownloading missing runner: {}", rv);
                }
            }
        }
    }
}

pub fn sync_install_backgrounds(app: &AppHandle) {
    if let Some(is) = get_installs(app) {
        log::debug!("Started background sync for {} installs", is.len());
        for i in is {
            let repm = match get_manifest_info_by_id(app, i.manifest_id.clone()) { Some(r) => r, None => continue };
            let gm = get_manifest(app, repm.filename);
            if let Some(g) = gm {
                let cur = match g.game_versions.iter().find(|e| e.metadata.version == i.version) { Some(v) => v, None => continue };
                #[cfg(target_os = "linux")]
                let is_live = false;
                #[cfg(not(target_os = "linux"))]
                let is_live = i.game_background.ends_with(".webm") || i.game_background.ends_with(".mp4");
                let bg = if is_live { cur.assets.game_live_background.clone().unwrap_or(cur.assets.game_background.clone()) } else { cur.assets.game_background.clone() };
                if !i.ignore_updates && i.game_background != bg { update_install_after_update_by_id(app, i.id, i.name, i.game_icon, bg, i.version); }
            }
        }
        log::debug!("Finished background sync for all installs");
    }
}

#[cfg(target_os = "linux")]
pub fn deprecate_jadeite(app: &AppHandle) {
    let installs = get_installs(app);
    if installs.is_some() {
        let i = installs.unwrap();
        for ci in i {
            let im = get_manifest_info_by_id(&app, ci.manifest_id);
            if im.is_some() {
                let lm = im.unwrap();
                // Shit validation but will work
                if lm.display_name.to_ascii_lowercase().contains("wuthering") { update_install_use_jadeite_by_id(&app, ci.id.clone(), false); }
                if lm.display_name.to_ascii_lowercase().contains("starrail") { update_install_use_jadeite_by_id(&app, ci.id.clone(), false); }
                if lm.display_name.to_ascii_lowercase().contains("honkaiimpact") { update_install_use_jadeite_by_id(&app, ci.id.clone(), false); }
            }
        }
    }
}

#[cfg(target_os = "linux")]
#[allow(non_camel_case_types)]
pub fn raise_fd_limit(new_limit: i32) {
    type rlim_t = u64;
    #[repr(C)]
    struct rlimit { rlim_cur: rlim_t, rlim_max: rlim_t }
    const RLIMIT_NOFILE: i32 = 7;
    unsafe extern "C" {
        fn getrlimit(resource: i32, rlp: *mut rlimit) -> i32;
        fn setrlimit(resource: i32, rlp: *const rlimit) -> i32;
    }
    let mut cur = rlimit { rlim_cur: 0, rlim_max: 0 };
    unsafe { getrlimit(RLIMIT_NOFILE, &mut cur); };
    if cur.rlim_cur >= cur.rlim_max { return; }
    let v = if new_limit == 999999 { cur.rlim_max } else { new_limit as rlim_t };
    let mut new = rlimit { rlim_cur: v, rlim_max: cur.rlim_max };
    unsafe { setrlimit(RLIMIT_NOFILE, &mut new); };
}

pub fn notify_update(app: &AppHandle) {
    let ttl = get_github_release("TwintailTeam/TwintailLauncher".to_string());
    if ttl.is_some() {
        let r = ttl.unwrap();
        let v = r.tag_name.unwrap().replace("ttl-v", "");
        let suppressed = app.path().app_data_dir().unwrap().join(".updatenaghide");
        if !suppressed.exists() {
            let cfg = app.config();
            match compare_version(cfg.version.clone().unwrap().as_str(), v.as_str()) {
                std::cmp::Ordering::Less => {
                    show_dialog_with_callback(&app, "warning", "TwintailLauncher", "You are running outdated version of TwintailLauncher!\nWe recommend updating to the latest version for best experience.\nIf you are using Flatpak version on Linux updates are always delayed for some time, sit tight and relax.", Some(vec!["Continue anyway"]), None);
                    log::info!("You are running outdated version of TwintailLauncher!");
                }
                std::cmp::Ordering::Equal => {
                    log::info!("You are running up to date version of TwintailLauncher!");
                    #[cfg(debug_assertions)]
                    println!("You are running up to date version of TwintailLauncher!");
                }
                std::cmp::Ordering::Greater => {
                    log::info!("You are running newer version of TwintailLauncher! Is it dev build?");
                    #[cfg(debug_assertions)]
                    println!("You are running newer version of TwintailLauncher! Is it dev build?");
                }
            }
        }
    }
}

pub fn prevent_system_idle(is_idle: bool) -> Option<keepawake::KeepAwake> {
    keepawake::Builder::default().display(false).sleep(false).idle(is_idle).reason("TwintailLauncher requested action").app_name("TwintailLauncher").app_reverse_domain("app.twintaillauncher.ttl").create().ok()
}

#[cfg(target_os = "linux")]
pub fn is_flatpak() -> bool {
    std::env::var("FLATPAK_ID").is_ok()
}

#[cfg(target_os = "linux")]
pub fn fix_window_decorations(app: &AppHandle) {
    let ssd = vec!["hyprland", "i3", "sway", "bspwm", "awesome", "dwm", "xmonad", "qtile", "niri", "mango", "mangowc"];
    match std::env::var("XDG_SESSION_DESKTOP") {
        Ok(val) => { if ssd.contains(&&**&val.to_ascii_lowercase()) { app.get_window("main").unwrap().set_decorations(false).unwrap(); } else { app.get_window("main").unwrap().set_decorations(true).unwrap(); } },
        Err(_e) => {},
    }
}

#[cfg(target_os = "linux")]
pub fn update_steam_compat_config(append_items: Vec<&str>) -> String {
    let existing = std::env::var("STEAM_COMPAT_CONFIG").unwrap_or_default();
    let mut compat_flags: Vec<String> = Vec::new();
    let mut cmdline_appends: Vec<String> = Vec::new();
    let mut config = existing.as_str();
    while !config.is_empty() {
        let (cur, remainder) = match config.split_once(',') {
            Some((c, r)) => (c, r),
            None => (config, ""),
        };
        if cur.starts_with("cmdlineappend:") {
            let mut full_arg = cur.to_string();
            let mut remaining = remainder;
            while full_arg.ends_with('\\') && !remaining.is_empty() {
                let (next_part, new_remainder) = match remaining.split_once(',') {
                    Some((n, r)) => (n, r),
                    None => (remaining, ""),
                };
                full_arg = format!("{}{}", &full_arg[..full_arg.len() - 1], next_part);
                remaining = new_remainder;
            }
            let arg = full_arg[14..].replace("\\\\", "\\");
            cmdline_appends.push(arg);
        } else if !cur.trim().is_empty() {
            compat_flags.push(cur.to_string());
        }
        config = remainder;
    }
    for item in append_items.iter() {
        if item.starts_with("cmdlineappend:") {
            let arg = item[14..].replace("\\\\", "\\");
            cmdline_appends.push(arg);
        } else {
            compat_flags.push(item.to_string());
        }
    }
    let mut new_parts: Vec<String> = Vec::new();
    for flag in &compat_flags {
        new_parts.push(flag.clone());
    }
    for append_arg in &cmdline_appends {
        let mut escaped_arg = append_arg.replace('\\', "\\\\");
        escaped_arg = escaped_arg.replace(',', "\\,");
        new_parts.push(format!("cmdlineappend:{}", escaped_arg));
    }
    let new_config = new_parts.join(",");
    new_config
}

#[cfg(target_os = "linux")]
pub fn apply_patch(app: &AppHandle, dir: String, patch_type: String, mode: String) {
    let dir = Path::new(&dir);
    if dir.exists() {
        match patch_type.as_str() {
            "aki" => {
                let f = dir.join("Client/Binaries/Win64/ThirdParty/KrPcSdk_Global/KRSDKRes/KRSDK.bin");
                if f.exists() {
                    match mode.as_str() {
                        "add" => {
                            let mut data = fs::read(&f).unwrap();
                            let from = b"KR_ChannelID=240";
                            let to   = b"KR_ChannelID=205";
                            if let Some(pos) = data.windows(from.len()).position(|w| w == from) { data[pos..pos+to.len()].copy_from_slice(to); fs::write(&f, &data).unwrap(); log::debug!("Applied AKI patch to {}", dir.display()); }
                        }
                        "remove" => {
                            let mut data = fs::read(&f).unwrap();
                            let from = b"KR_ChannelID=205";
                            let to   = b"KR_ChannelID=240";
                            if let Some(pos) = data.windows(from.len()).position(|w| w == from) { data[pos..pos+to.len()].copy_from_slice(to); fs::write(&f, &data).unwrap(); log::debug!("Removed AKI patch from {}", dir.display()); }
                        }
                        _ => {}
                    }
                }
            },
            "sparkle" => {
                let target_old = dir.join("dbghelp.dll");
                if target_old.exists() { fs::remove_file(&target_old).unwrap(); }
                match mode.as_str() {
                    "add" => {
                        let patch = app.path().resource_dir().unwrap().join("resources").join("hkrpg_patch.dll");
                        let target = dir.join("jsproxy.dll");
                        if patch.exists() { fs::copy(&patch, &target).unwrap(); log::debug!("Applied Sparkle patch to {}", dir.display()); }
                    }
                    "remove" => {
                        let target = dir.join("jsproxy.dll");
                        if target.exists() { fs::remove_file(&target).unwrap(); log::debug!("Removed Sparkle patch from {}", dir.display()); }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

#[allow(unused_mut)]
pub fn apply_xxmi_tweaks(package: PathBuf, mut data: Json<XXMISettings>) -> Json<XXMISettings> {
    if package.exists() {
        let cfg = package.join("d3dx.ini");
        if cfg.exists() {
            let actions = if data.dump_shaders { "clipboard hlsl asm regex" } else { "clipboard" };
            let mut managed: Vec<(&str, &str, String)> = vec![("Hunting","hunting",data.hunting_mode.to_string()), ("Hunting","marking_actions",actions.to_string()), ("Logging","show_warnings",data.show_warnings.to_string()), ];

            #[cfg(target_os = "linux")]
            {
                if package.to_str().unwrap().contains("gimi") || package.to_str().unwrap().contains("zzmi") {
                    data.require_admin = false;
                    data.dll_init_delay = 500;
                    data.close_delay = 20;
                    managed.push(("Loader","require_admin",data.require_admin.to_string()));
                    managed.push(("Loader","delay",data.close_delay.to_string()));
                    managed.push(("System","dll_initialization_delay",data.dll_init_delay.to_string()));
                }
            }
            // Fuck us deeply if we even understand how tf this parsing even works... but it does SO tukan will kill you if you touch it
            if let Ok(content) = fs::read_to_string(&cfg) {
                let mut out: Vec<String> = Vec::new();
                let mut cur = String::new();
                let mut written = vec![false; managed.len()];
                for line in content.lines() {
                    let tr = line.trim();
                    if tr.starts_with('[') && tr.ends_with(']') {
                        for (i,(s,k,v)) in managed.iter().enumerate() { if !written[i] && *s == cur.as_str() { out.push(format!("{} = {}",k,v)); written[i]=true; } }
                        cur = tr[1..tr.len()-1].to_string();
                        out.push(line.to_string());
                    } else if !tr.starts_with(';') && !tr.starts_with('#') && tr.contains('=') {
                        let key = tr.splitn(2,'=').next().unwrap_or("").trim();
                        if let Some(i) = managed.iter().position(|(s,k,_)| *s == cur.as_str() && *k == key) { out.push(format!("{} = {}",managed[i].1,managed[i].2)); written[i]=true; } else { out.push(line.to_string()); }
                    } else { out.push(line.to_string()); }
                }
                for (i,(s,k,v)) in managed.iter().enumerate() { if !written[i] && *s == cur.as_str() { out.push(format!("{} = {}",k,v)); } }
                let _ = fs::write(&cfg, out.join("\n"));
                log::debug!("Edited d3dx.ini at {} with values: {}", cfg.display(), managed.iter().map(|(s,k,v)| format!("{}->{}={}",s,k,v)).collect::<Vec<_>>().join(", "));
            }
            data
        } else { data }
    } else { data }
}

#[cfg(target_os = "linux")]
pub fn find_steamrt_version(file_path: PathBuf) -> io::Result<String> {
    let file = fs::File::open(file_path);
    match file {
        Ok(file) => {
            let reader = io::BufReader::new(file);
            for line in reader.lines() {
                let line = line?;
                for token in line.split_whitespace() {
                    if token.starts_with("3.") || token.starts_with("4.") && token.matches('.').count() >= 3 && token.chars().all(|c| c.is_ascii_digit() || c == '.') { return Ok(token.to_string()); }
                }
            }
        }
        Err(_) => { log::debug!("Could not find VERSIONS.txt in steamrt directory!"); }
    }
    Ok(String::new())
}

#[cfg(target_os = "linux")]
pub fn compare_steamrt_versions(v1: &str, v2: &str) -> bool {
    let parts1: Vec<u64> = v1.split('.').map(|v| v.parse().unwrap_or(0)).collect();
    let parts2: Vec<u64> = v2.split('.').map(|v| v.parse().unwrap_or(0)).collect();
    for (a, b) in parts1.iter().zip(parts2.iter()) { if a > b { return true; } else if a < b { return false; } }
    parts1.len() > parts2.len()
}

pub fn compare_version(a: &str, b: &str) -> std::cmp::Ordering {
    fn parse(s: &str) -> (u64, u64, u64) {
        let ss = s.replace("-", ".");
        let mut it = ss.split('.');
        let major = it.next().unwrap_or("1").parse().unwrap_or(1);
        let minor = it.next().unwrap_or("0").parse().unwrap_or(0);
        let patch = it.next().unwrap_or("0").parse().unwrap_or(0);
        (major, minor, patch)
    }
    let va = parse(a);
    let vb = parse(b);
    va.cmp(&vb)
}

pub fn find_package_version(file_path: PathBuf, package_name: &str) -> Option<String> {
    let file = fs::File::open(file_path).ok()?;
    let reader = io::BufReader::new(file);
    for line in reader.lines().flatten() {
        if let Some((key, value)) = line.split_once('=') {
            if key.trim().to_ascii_uppercase() == package_name.to_ascii_uppercase() { return Some(value.trim().to_string()); }
        }
    }
    None
}

#[cfg(target_os = "linux")]
pub fn is_runner_lower(min_runner_versions: Vec<String>, runner_version: String) -> bool {
    let idx = match runner_version.find("proton-") { Some(i) => i, None => return false };
    let (left, right) = runner_version.split_at(idx);
    let cand_ver = left.strip_suffix('-').unwrap_or(left).replace("-", ".");
    for s in min_runner_versions {
        let idx = match s.find("proton-") { Some(i) => i, None => continue };
        let (l, r) = s.split_at(idx);
        let ver = l.strip_suffix('-').unwrap_or(l).replace("-", ".");
        if r == right && compare_version(cand_ver.as_str(), ver.as_str()).is_lt() { return true; }
    }
    false
}

#[cfg(target_os = "linux")]
pub fn is_using_overriden_runner(installed_runner: String, override_runner: String) -> bool {
    fn family_suffix(s: &str) -> Option<&str> {
        let idx = s.rfind("proton-")?;
        Some(&s[idx..])
    }
    let override_family = match family_suffix(&override_runner) { Some(f) => f, None => return false };
    let installed_family = match family_suffix(&installed_runner) { Some(f) => f, None => return false };
    installed_family == override_family
}

#[allow(dead_code)]
pub fn empty_dir<P: AsRef<Path>>(dir: P) -> io::Result<()> {
    const EXCEPTIONS: &[&str] = &["Mods/", "ShaderCache/", "ShaderFixes/", "d3dx_user.ini", "gimi/", "srmi/", "zzmi/", "himi/", "wwmi/", "ssmi/", "efmi/"];
    if dir.as_ref().exists() {
        for entry in fs::read_dir(dir.as_ref())? {
            let entry = entry?;
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                let is_dir = path.is_dir();
                let should_skip = EXCEPTIONS.iter().any(|&ex| { if ex.ends_with('/') { is_dir && name.contains(&ex[..ex.len() - 1]) } else { !is_dir && name == ex } });
                if should_skip { continue; }
                if is_dir { fs::remove_dir_all(&path)?; } else { fs::remove_file(&path)?; }
            }
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn get_steam_appid() -> u32 {
    if let Ok(path) = std::env::var("STEAM_COMPAT_TRANSCODED_MEDIA_PATH") {
        if let Some(last) = Path::new(&path).components().last() {
            if let Some(val) = last.as_os_str().to_str() { if let Ok(id) = val.parse::<u32>() { return id; } }
        }
    }
    if let Ok(path) = std::env::var("STEAM_COMPAT_MEDIA_PATH") {
        let parts: Vec<_> = Path::new(&path).components().collect();
        if parts.len() >= 2 {
            if let Some(val) = parts[parts.len() - 2].as_os_str().to_str() { if let Ok(id) = val.parse::<u32>() { return id; } }
        }
    }
    if let Ok(path) = std::env::var("STEAM_FOSSILIZE_DUMP_PATH") {
        let parts: Vec<_> = Path::new(&path).components().collect();
        if parts.len() >= 3 {
            if let Some(val) = parts[parts.len() - 3].as_os_str().to_str() { if let Ok(id) = val.parse::<u32>() { return id; } }
        }
    }
    if let Ok(path) = std::env::var("DXVK_STATE_CACHE_PATH") {
        let parts: Vec<_> = Path::new(&path).components().collect();
        if parts.len() >= 2 {
            if let Some(val) = parts[parts.len() - 2].as_os_str().to_str() { if let Ok(id) = val.parse::<u32>() { return id; } }
        }
    }
    if let Ok(id_str) = std::env::var("SteamGameId") { if let Ok(id) = id_str.parse::<u64>() { return (id >> 32) as u32; } }
    0
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Copy)]
pub enum SteamRTType {
    Soldier,
    SteamRT3,
    SteamRT3Arm64,
    SteamRT4,
    SteamRT4Arm64,
}

#[cfg(target_os = "linux")]
impl SteamRTType {
    pub fn runtime_name(&self) -> &'static str {
        match self {
            SteamRTType::Soldier => "soldier",
            SteamRTType::SteamRT3 => "steamrt3",
            SteamRTType::SteamRT3Arm64 => "steamrt3",
            SteamRTType::SteamRT4 => "steamrt4",
            SteamRTType::SteamRT4Arm64 => "steamrt4",
        }
    }

    #[allow(unused)]
    pub fn tool_appid(&self) -> &'static str {
        match self {
            SteamRTType::Soldier => "1391110",
            SteamRTType::SteamRT3 => "1628350",
            SteamRTType::SteamRT3Arm64 => "3810310",
            SteamRTType::SteamRT4 => "4183110",
            SteamRTType::SteamRT4Arm64 => "4185400",
        }
    }

    pub fn from_tool_appid(appid: &str) -> Option<Self> {
        match appid {
            "1391110" => Some(SteamRTType::Soldier),
            "1628350" => Some(SteamRTType::SteamRT3),
            "3810310" => Some(SteamRTType::SteamRT3Arm64),
            "4183110" => Some(SteamRTType::SteamRT4),
            "4185400" => Some(SteamRTType::SteamRT4Arm64),
            _ => None,
        }
    }
}

#[cfg(target_os = "linux")]
pub fn get_steam_tool_appid(path: PathBuf) -> String {
    let manifest_path = path.join("toolmanifest.vdf");
    if let Ok(manifest_str) = fs::read_to_string(&manifest_path) {
        for line in manifest_str.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("\"require_tool_appid\"") {
                if let Some(start) = trimmed.rfind('"') {
                    if let Some(value_start) = trimmed[..start].rfind('"') {
                        let appid = &trimmed[value_start + 1..start];
                        if let Some(runtime) = SteamRTType::from_tool_appid(appid) { return runtime.runtime_name().to_string(); }
                    }
                }
            }
        }
    }
    String::new()
}

fn collect_authkey_urls(content: &str) -> Vec<&str> {
    let mut rslt = Vec::<&str>::new();
    let mut offset: usize = 0;
    while offset < content.len() {
        let next = content[offset..].find("https://");
        if next.is_none() { break; }
        let start = offset + next.unwrap();
        let sliced = &content[start..];
        let end = sliced.find(|c: char| c.is_whitespace() || c == '"' || c == '\'' || c == '<' || c == '>').unwrap_or(sliced.len());
        let url = &sliced[..end];
        if url.contains("authkey=") { rslt.push(url); }
        offset = start + "https://".len();
    }
    rslt
}

pub fn extract_authkey_from_content(content: &str) -> Option<String> {
    let urls = collect_authkey_urls(content);
    let hints = vec!["webview_gacha"];
    for url in urls.into_iter().rev() {
        let lowered = url.to_ascii_lowercase();
        if !hints.iter().any(|h| lowered.contains(h)) { continue; }
        if let Ok(uri) = fischl::utils::parse_url(url.parse().unwrap()) {
            for (key, value) in uri.query_pairs() { if key.eq_ignore_ascii_case("authkey") && value.len() > 10 { return Some(value.into_owned()); } }
        }
    }
    None
}

pub fn get_engine_log_from_game(base: String, game_name: String, region_code: String) -> String {
    if game_name.to_ascii_lowercase().contains("genshin") { return "miHoYo/Genshin Impact/output_log.txt".to_string() }
    if game_name.to_ascii_lowercase().contains("starrail") { return "Cognosphere/Star Rail/Player.log".to_string() }
    if game_name.to_ascii_lowercase().contains("zenless") { return "miHoYo/ZenlessZoneZero/Player.log".to_string() }
    if game_name.to_ascii_lowercase().contains("honkai") {
        if region_code.to_ascii_lowercase().contains("glb_official") { return "miHoYo/Honkai Impact 3rd/Player.log".to_string() }
        if region_code.to_ascii_lowercase().contains("overseas_official") { return "miHoYo/Honkai Impact 3/Player.log".to_string() }
        if region_code.to_ascii_lowercase().contains("kr_official") { return "miHoYo/붕괴3rd/Player.log".to_string() }
        if region_code.to_ascii_lowercase().contains("asia_offcial") { return "miHoYo/崩壊3rd/Player.log".to_string() }
        if region_code.to_ascii_lowercase().contains("jp_official") { return "miHoYo/崩壊3rd/Player.log".to_string() }
        return "miHoYo/Honkai Impact 3rd/Player.log".to_string()
    }
    if game_name.to_ascii_lowercase().contains("punishing") { return fs::read_dir(PathBuf::from(&base).join("kurogame/PGR/log")).ok().and_then(|e| e.filter_map(|e| e.ok()).max_by_key(|e| e.file_name()).map(|e| format!("kurogame/PGR/log/{}", e.file_name().to_string_lossy()))).unwrap_or_default(); }
    if game_name.to_ascii_lowercase().contains("endfield") { return "Gryphline/Endfield/Player.log".to_string() }
    "".to_string()
}

// === LinkedHashMap ===

#[derive(Debug, Clone)]
pub struct LinkedHashMap<K, V> { order: Vec<K>, map: HashMap<K, V>, }
impl<K, V> Default for LinkedHashMap<K, V> {
    fn default() -> Self { Self { order: Vec::new(), map: HashMap::new() } }
}
impl<K: Eq + Hash + Clone, V: Clone> LinkedHashMap<K, V> {
    pub fn new() -> Self { Self { order: Vec::new(), map: HashMap::new() } }
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if !self.map.contains_key(&key) { self.order.push(key.clone()); }
        self.map.insert(key, value)
    }
    pub fn get(&self, key: &K) -> Option<&V> { self.map.get(key) }
    pub fn contains_key(&self, key: &K) -> bool { self.map.contains_key(key) }
    pub fn len(&self) -> usize { self.order.len() }
    pub fn is_empty(&self) -> bool { self.order.is_empty() }
}
impl<K: Eq + Hash + Clone, V: Clone> IntoIterator for LinkedHashMap<K, V> {
    type Item = (K, V);
    type IntoIter = LinkedHashMapIter<K, V>;
    fn into_iter(mut self) -> Self::IntoIter {
        let items: Vec<(K, V)> = self.order.into_iter().filter_map(|k| { self.map.remove(&k).map(|v| (k, v)) }).collect();
        LinkedHashMapIter { inner: items.into_iter() }
    }
}
impl<'a, K: Eq + Hash + Clone, V: Clone> IntoIterator for &'a LinkedHashMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter { Iter { inner: self.order.iter().map(|k| (k, self.map.get(k).unwrap())).collect::<Vec<_>>().into_iter() } }
}
pub struct LinkedHashMapIter<K, V> { inner: std::vec::IntoIter<(K, V)>, }
impl<K, V> Iterator for LinkedHashMapIter<K, V> {
    type Item = (K, V);
    fn next(&mut self) -> Option<Self::Item> { self.inner.next() }
}
pub struct Iter<'a, K, V> { inner: std::vec::IntoIter<(&'a K, &'a V)>, }
impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> { self.inner.next() }
}
