use std::{fs, io};
use std::collections::HashMap;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use fischl::utils::{get_github_release};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Listener, Manager};
use tauri_plugin_notification::NotificationExt;
use crate::utils::db_manager::{get_installs, get_manifest_info_by_id, get_settings, update_install_after_update_by_id, update_settings_default_fps_unlock_location, update_settings_default_game_location, update_settings_default_xxmi_location};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use sqlx::types::Json;
use crate::utils::models::{GameVersion, XXMISettings};
use crate::utils::repo_manager::{get_manifest};

#[cfg(target_os = "linux")]
use crate::utils::{repo_manager::{get_manifests}, db_manager::{get_installed_runners, update_installed_runner_is_installed_by_version, create_installed_runner, get_installed_runner_info_by_version, update_install_use_jadeite_by_id, update_settings_default_jadeite_location, update_settings_default_prefix_location, update_settings_default_runner_location, update_settings_default_dxvk_location}};

pub mod db_manager;
pub mod repo_manager;
mod git_helpers;
pub mod game_launch_manager;
pub mod system_tray;
pub mod args;
pub mod shortcuts;
pub mod models;
#[cfg(target_os = "linux")]
pub mod gpu;

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

    prevent_exit(app, true);
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
            tracker.fetch_add(size, Ordering::SeqCst);

            let mut payload = HashMap::new();
            payload.insert("file", f.to_str().unwrap().to_string());
            payload.insert("install_id", install.clone());
            payload.insert("install_name", install_name.clone());
            payload.insert("install_type", install_type.clone());
            payload.insert("progress", tracker.load(Ordering::SeqCst).to_string());
            payload.insert("total", totalsize.to_string());

            if let Err(e) = fs::copy(ep.clone(), dst.as_ref().join(f)) { eprintln!("Failed to copy {}: {}", ep.clone().display(), e); }
            app.emit("move_progress", &payload).unwrap();
            files_to_remove.push(ep.clone());
        }
    }
    for file_path in files_to_remove {
        if file_path.is_file() || file_path.is_symlink() {
            if let Err(e) = fs::remove_file(file_path) { eprintln!("Failed to remove file: {}", e); }
        } else {
            if let Err(e) = fs::remove_dir_all(file_path) { eprintln!("Failed to remove directory: {}", e); }
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn block_telemetry(app: &AppHandle) {
    // For the time being just return if we are flatpak build will be fixed soon
    if is_flatpak() { send_notification(&app, r#"Telemetry block is currently impossible inside flatpak sandbox! Team is working on the workaround fix."#, None); return; }
    let app1 = Arc::new(Mutex::new(app.clone()));
    std::thread::spawn(move || {
        let app = app1.lock().unwrap().clone();
        let manifests = get_manifests(&app);
        let mut allhosts = String::new();
        let mut unique = Vec::new();

        manifests.values().for_each(|manifest| {
            let hosts = manifest.telemetry_hosts.iter().map(|server| format!("echo '0.0.0.0 {server}' >> /etc/hosts")).filter(|entry| { if unique.contains(entry) { false } else { unique.push(entry.clone());true } }).collect::<Vec<String>>().join(" ; ");
            if !hosts.is_empty() {
                if !allhosts.is_empty() { allhosts.push_str(" ; "); }
                allhosts.push_str(&hosts);
            }
        });

        let remove_block_cmd = "sed -i '/# TwintailLauncher telemetry block start/,/# TwintailLauncher telemetry block end/d' /etc/hosts";
        let shell_cmd = format!("{remove_block_cmd} ; \
         echo '# TwintailLauncher telemetry block start' >> /etc/hosts ; \
         {allhosts} ; \
         echo '# TwintailLauncher telemetry block end' >> /etc/hosts");

        let output = std::process::Command::new("pkexec").env("PKEXEC_DESCRIPTION", "TwintailLauncher wants to block game telemetry servers").arg("bash").arg("-c").arg(shell_cmd).spawn();
        match output.and_then(|child| child.wait_with_output()) {
            Ok(output) => if !output.status.success() { send_notification(&app, r#"Failed to block telemetry servers, Please press "Block telemetry" in launcher settings!"#, None);
            } else {
                let path = app.path().app_data_dir().unwrap().join(".telemetry_blocked");
                if !path.exists() {
                    send_notification(&app, "Successfully blocked telemetry servers.", None);
                    fs::write(&path, ".").unwrap();
                } else { send_notification(&app, "Telemetry servers already blocked.", None); }
            }
            Err(_err) => { send_notification(&app, r#"Failed to block telemetry servers, something seriously failed or we are running under flatpak!"#, None); }
        }
    });
}

#[cfg(target_os = "windows")]
pub fn block_telemetry(_app: &AppHandle) {}

pub fn register_listeners(app: &AppHandle) {
    let h1 = app.clone();
    app.listen("launcher_action_exit", move |_event| {
        let blocks = h1.state::<Mutex<ActionBlocks>>();
        let state = blocks.lock().unwrap();
        if state.action_exit { h1.get_window("main").unwrap().hide().unwrap(); h1.emit("sync_tray_toggle", "Show").unwrap(); } else { h1.get_window("main").unwrap().hide().unwrap(); h1.cleanup_before_exit(); h1.exit(0); std::process::exit(0); }
    });
    let h2 = app.clone();
    app.listen("launcher_action_minimize", move |_event| { h2.get_window("main").unwrap().minimize().unwrap(); });
}

pub fn send_notification(app: &AppHandle, body: &str, icon: Option<&str>) {
    if body.is_empty() { return; }
    if icon.is_some() { let i = icon.unwrap(); app.notification().builder().icon(i).title("TwintailLauncher").body(body).show().unwrap(); } else { app.notification().builder().title("TwintailLauncher").body(body).show().unwrap(); }
}

pub fn prevent_exit(app: &AppHandle, val: bool) {
    let blocks = app.state::<Mutex<ActionBlocks>>();
    let mut state = blocks.lock().unwrap();
    match val {
        true => {
            state.action_exit = true;
            drop(state);
        }
        false => {
            state.action_exit = false;
            drop(state);
        }
    }
}

#[cfg(target_os = "linux")]
pub fn runner_from_runner_version(runner_version: String) -> Option<String> {
    let mut rslt = String::new();

    if runner_version.is_empty() {
        None
    } else {
        if runner_version.to_lowercase().contains("vanilla") {
            rslt = "dxvk_vanilla.json".to_string();
        }
        if runner_version.to_lowercase().contains("async") {
            rslt = "dxvk_async.json".to_string();
        }
        if runner_version.to_lowercase().contains("gplasync") {
            rslt = "dxvk_gplasync.json".to_string();
        }
        if runner_version.to_lowercase().contains("wine-vanilla") {
            rslt = "wine_vanilla.json".to_string();
        }
        if runner_version.to_lowercase().contains("wine-staging") {
            rslt = "wine_staging.json".to_string();
        }
        if runner_version.to_lowercase().contains("wine-staging-tkg") {
            rslt = "wine_staging_tkg.json".to_string();
        }
        if runner_version.to_lowercase().contains("wine-vaniglia") {
            rslt = "wine_vaniglia.json".to_string();
        }
        if runner_version.to_lowercase().contains("wine-soda") {
            rslt = "wine_soda.json".to_string();
        }
        if runner_version.to_lowercase().contains("wine-lutris") {
            rslt = "wine_lutris.json".to_string();
        }
        if runner_version.to_lowercase().contains("wine-ge-proton") {
            rslt = "wine_ge_proton.json".to_string();
        }
        if runner_version.to_lowercase().contains("wine-caffe") {
            rslt = "wine_caffe.json".to_string();
        }
        if runner_version.to_lowercase().contains("proton-ge") || runner_version.to_lowercase().contains("ge-proton") {
            rslt = "proton_ge.json".to_string();
        }
        if runner_version.to_lowercase().contains("proton-cachyos") {
            rslt = "proton_cachyos.json".to_string();
        }
        if runner_version.to_lowercase().contains("proton-cachyos-spritz") {
            rslt = "proton_cachyos_spritz.json".to_string();
        }
        if runner_version.to_lowercase().contains("proton-umu") {
            rslt = "proton_umu.json".to_string();
        }
        if runner_version.to_lowercase().contains("proton-vanilla") {
            rslt = "proton_vanilla.json".to_string();
        }
        if runner_version.to_lowercase().contains("proton-em") {
            rslt = "proton_em.json".to_string();
        }
        Some(rslt)
    }
}

pub fn get_mi_path_from_game(exe_name: String) -> Option<String> {
    if exe_name.is_empty() { None } else {
        let exe = exe_name.split('/').last().unwrap().to_string();
        match exe.to_ascii_lowercase().as_str() {
            "genshinimpact.exe" => { Some("gimi".parse().unwrap()) },
            "starrail.exe" => { Some("srmi".parse().unwrap()) },
            "zenlesszonezero.exe" => { Some("zzmi".parse().unwrap()) },
            "bh3.exe" => { Some("himi".parse().unwrap()) },
            "client-win64-shipping.exe" => { Some("wwmi".parse().unwrap()) },
            _ => { None }
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
            if g.fps_unlock_path == "" { fs::create_dir_all(&fpsunlockpath).unwrap(); update_settings_default_fps_unlock_location(app, fpsunlockpath.to_str().unwrap().to_string()); }

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
                if !steamrtpath.exists() { fs::create_dir_all(&steamrtpath).unwrap(); }

                if g.jadeite_path == "" { fs::create_dir_all(&jadeitepath).unwrap(); update_settings_default_jadeite_location(app, jadeitepath.to_str().unwrap().to_string()); }
                if g.default_runner_path == "" { fs::create_dir_all(&wine).unwrap(); update_settings_default_runner_location(app, wine.to_str().unwrap().to_string()); }
                if g.default_dxvk_path == "" { fs::create_dir_all(&dxvk).unwrap(); update_settings_default_dxvk_location(app, dxvk.to_str().unwrap().to_string()); }
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
            if !steamrtpath.exists() { fs::create_dir_all(&steamrtpath).unwrap(); }

            if !mangohudcfg.exists() { db_manager::update_settings_default_mangohud_config_location(app, mangohudcfg.to_str().unwrap().to_string()); } else { db_manager::update_settings_default_mangohud_config_location(app, mangohudcfg.to_str().unwrap().to_string()); }
            if !jadeitepath.exists() { fs::create_dir_all(&jadeitepath).unwrap(); update_settings_default_jadeite_location(app, jadeitepath.to_str().unwrap().to_string()); }
            if !comppath.exists() {
                fs::create_dir_all(&wine).unwrap();
                fs::create_dir_all(&dxvk).unwrap();
                fs::create_dir_all(&prefixes).unwrap();
                fs::create_dir_all(&steamrtpath).unwrap();
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

        for e in fs::read_dir(runners).unwrap() {
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
                            },
                            Err(_) => {}
                        }
                    }
                },
                Err(_) => {}
            }
        }
    }
}

pub fn sync_install_backgrounds(app: &AppHandle) {
    let installs = get_installs(app);
    if let Some(is) = installs {
        for i in is {
            let repm = get_manifest_info_by_id(app, i.manifest_id).unwrap();
            let gm = get_manifest(&app, repm.filename);
            if let Some(g) = gm {
                let is_live = i.game_background.ends_with(".webm") || i.game_background.ends_with(".mp4");
                let ver = g.game_versions.iter().filter(|e| e.metadata.version == i.version).collect::<Vec<&GameVersion>>();
                if ver.is_empty() { return; }
                let cur = ver.get(0).unwrap();
                let comparator = if is_live { i.game_background != cur.assets.game_live_background.clone().unwrap() } else { i.game_background != cur.assets.game_background.clone() };
                if !i.ignore_updates && comparator {
                    let bg = if is_live { cur.assets.game_live_background.clone().unwrap() } else { cur.assets.game_background.clone() };
                    update_install_after_update_by_id(app, i.id, i.name, i.game_icon, bg, i.version);
                }
            }
        }
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
            }
        }
    }
}

#[cfg(target_os = "linux")]
#[allow(non_camel_case_types)]
pub fn raise_fd_limit(new_limit: i32) {
    type rlim_t = u64;
    #[repr(C)]
    struct rlimit { rlim_cur: rlim_t, rlim_max: rlim_t, }
    const RLIMIT_NOFILE: i32 = 7;
    unsafe extern "C" {
        fn getrlimit(resource: i32, rlp: *mut rlimit) -> i32;
        fn setrlimit(resource: i32, rlp: *const rlimit) -> i32;
    }
    let mut cur = rlimit { rlim_cur: 0, rlim_max: 0 };
    unsafe { getrlimit(RLIMIT_NOFILE, &mut cur); };
    if cur.rlim_cur >= cur.rlim_max { return; }
    let v = if new_limit == 999999 { cur.rlim_max } else { new_limit as rlim_t };
    let mut new = rlimit {rlim_cur: v, rlim_max: cur.rlim_max };
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
                    app.dialog().message("You are running outdated version of TwintailLauncher!\nWe recommend updating to the latest version for best experience.\nIf you are using Flatpak version on Linux updates are always delayed for some time, sit tight and relax.").title("TwintailLauncher")
                        .kind(MessageDialogKind::Warning)
                        .buttons(MessageDialogButtons::OkCustom("Continue anyway".to_string()))
                        //.buttons(MessageDialogButtons::OkCancelCustom("Continue anyway".to_string(), "Do not show again".to_string()))
                        .show(move |_action| { /*if action {  } else { fs::File::create(&suppressed).unwrap(); }*/ });
                }
                std::cmp::Ordering::Equal => {println!("You are running up to date version of TwintailLauncher!");}
                std::cmp::Ordering::Greater => {println!("You are running newer version of TwintailLauncher! Is it dev build?");}
            }
        }
    }
}

#[cfg(target_os = "linux")]
pub fn is_flatpak() -> bool { std::env::var("FLATPAK_ID").is_ok() }

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
                full_arg = format!("{}{}", &full_arg[..full_arg.len()-1], next_part);
                remaining = new_remainder;
            }
            let arg = full_arg[14..].replace("\\\\", "\\");
            cmdline_appends.push(arg);
        } else if !cur.trim().is_empty() { compat_flags.push(cur.to_string()); }
        config = remainder;
    }
    for item in append_items.iter() {
        if item.starts_with("cmdlineappend:") {
            let arg = item[14..].replace("\\\\", "\\");cmdline_appends.push(arg); } else { compat_flags.push(item.to_string()); }
    }
    let mut new_parts: Vec<String> = Vec::new();
    for flag in &compat_flags { new_parts.push(flag.clone()); }
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
                match mode.as_str() {
                    "add" => {
                        let f = dir.join("Client/Binaries/Win64/ThirdParty/KrPcSdk_Global/KRSDKRes/KRSDK.bin");
                        if f.exists() {
                            let fp = fs::read_to_string(f.clone()).unwrap();
                            let patched = fp.lines().map(|line| {
                                if line.starts_with("KR_ChannelID=") { "KR_ChannelID=205" } else { line }
                            }).collect::<Vec<_>>().join("\n");
                            fs::write(f, patched).unwrap();
                        }
                    }
                    "remove" => {}
                    _ => {}
                }
            }
            "sparkle" => {
                let target_old = dir.join("dbghelp.dll");
                if target_old.exists() { fs::remove_file(&target_old).unwrap(); }
                match mode.as_str() {
                    "add" => {
                        let patch = app.path().resource_dir().unwrap().join("resources").join("hkrpg_patch.dll");
                        let target = dir.join("jsproxy.dll");
                        if patch.exists() { fs::copy(&patch, &target).unwrap(); }
                    }
                    "remove" => {
                        let target = dir.join("jsproxy.dll");
                        if target.exists() { fs::remove_file(&target).unwrap(); }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

pub fn edit_wuwa_configs_xxmi(engine_ini: String) {
    let file = Path::new(&engine_ini);
    if file.exists() {
        let mut ini = configparser::ini::Ini::new_cs();
        let f = ini.load(&file);
        match f {
            Ok(_) => {
                let perf_tweaks: HashMap<&str, HashMap<&str, String>> = HashMap::from([(
                    "SystemSettings",
                    HashMap::from([("r.Streaming.HLODStrategy", "2".to_string()), ("r.Streaming.PoolSizeForMeshes", "-1".to_string()), ("r.XGEShaderCompile", "0".to_string()),
                        ("FX.BatchAsync", "1".to_string()), ("FX.EarlyScheduleAsync", "1".to_string()), ("fx.Niagara.ForceAutoPooling", "1".to_string()),
                        ("wp.Runtime.KuroRuntimeStreamingRangeOverallScale", "0.5".to_string()),
                        ("tick.AllowAsyncTickCleanup", "1".to_string()), ("tick.AllowAsyncTickDispatch", "1".to_string())])
                )]);
                for (section_name, section_data) in perf_tweaks {
                    for (option_name, option_value) in section_data { ini.set(section_name, option_name, Some(option_value)); }
                }
                for section in ini.get_map_ref().keys().cloned().collect::<Vec<_>>() {
                    ini.remove_key(&section, "r.Streaming.UsingNewKuroStreaming"); // Ancient 3rd-party configs set it to 0 with bad results
                    ini.remove_key(&section, "r.Streaming.Boost"); // Replaced with r.Streaming.MinBoost
                }
                ini.set("ConsoleVariables", "r.Streaming.LimitPoolSizeToVRAM", Some("1".to_string()));
                ini.set("ConsoleVariables", "r.Streaming.PoolSize", Some("0".to_string()));
                ini.set("ConsoleVariables", "r.Streaming.UseAllMips", Some("1".to_string()));
                ini.set("ConsoleVariables", "r.Streaming.MinBoost", Some("20.0".to_string()));
                ini.set("ConsoleVariables", "r.Kuro.SkeletalMesh.LODDistanceScale", Some("24".to_string()));
                ini.set("ConsoleVariables", "r.Kuro.SkeletalMesh.LODDistanceScaleDeviceOffset", Some("-50".to_string()));
                let r = ini.write(&file);
                match r { Ok(_) => {} Err(_) => {} }
            }
            Err(_) => {}
        }
    }
}

#[allow(unused_mut)]
pub fn apply_xxmi_tweaks(package: PathBuf, mut data: Json<XXMISettings>) -> Json<XXMISettings> {
    if package.exists() {
        let cfg = package.join("d3dx.ini");
        if cfg.exists() {
            let mut ini = configparser::ini::Ini::new_cs();
            let f = ini.load(&cfg);
            match f {
                Ok(_) => {
                    // Why is ini parser fucking these lines?? Explicitly set them back...
                    ini.set("Include", "exclude_recursive", Some("DISABLED*".to_string()));

                    // Apply edits
                    ini.set("Hunting", "hunting", Some(data.hunting_mode.to_string()));
                    let actions = if data.dump_shaders { "clipboard hlsl asm regex" } else { "clipboard" };
                    ini.set("Hunting", "marking_actions", Some(actions.to_string()));
                    ini.set("Logging", "show_warnings", Some(data.show_warnings.to_string()));
                    #[cfg(target_os = "linux")]
                    {
                        if package.to_str().unwrap().contains("gimi") || package.to_str().unwrap().contains("zzmi") {
                            data.require_admin = false;
                            data.dll_init_delay = 500;
                            data.close_delay = 20;
                            ini.set("Loader", "require_admin", Some(data.require_admin.to_string()));
                            ini.set("Loader", "delay", Some(data.close_delay.to_string()));
                            ini.set("System", "dll_initialization_delay", Some(data.dll_init_delay.to_string()));
                        }
                    }
                    let r = ini.write(&cfg);
                    match r { Ok(_) => {} Err(_) => {} }
                }
                Err(_) => {}
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
                    if token.starts_with("3.") && token.matches('.').count() >= 3 && token.chars().all(|c| c.is_ascii_digit() || c == '.') { return Ok(token.to_string()); }
                }
            }
        }
        Err(_) => {eprintln!("Could not find VERSIONS.txt in steamrt directory!");}
    }
    Ok(String::new())
}

#[cfg(target_os = "linux")]
pub fn compare_steamrt_versions(v1: &str, v2: &str) -> bool {
    let parts1: Vec<u64> = v1.split('.').map(|v| v.parse().unwrap_or(0)).collect();
    let parts2: Vec<u64> = v2.split('.').map(|v| v.parse().unwrap_or(0)).collect();
    for (a, b) in parts1.iter().zip(parts2.iter()) {
        if a > b { return true; } else if a < b { return false; }
    }
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
    let installed_family = match family_suffix(&installed_runner) { Some(f) => f, None => return false};
    installed_family == override_family
}

#[allow(dead_code)]
pub fn empty_dir<P: AsRef<Path>>(dir: P) -> io::Result<()> {
    const EXCEPTIONS: &[&str] = &["Mods/", "ShaderCache/", "d3dx_user.ini", "gimi/", "srmi/", "zzmi/", "himi/", "wwmi/"];
    if dir.as_ref().exists() {
        for entry in fs::read_dir(dir.as_ref())? {
            let entry = entry?;
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                let is_dir = path.is_dir();
                let should_skip = EXCEPTIONS.iter().any(|&ex| {
                    if ex.ends_with('/') { is_dir && name.contains(&ex[..ex.len() - 1]) } else { !is_dir && name == ex }
                });
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
            if let Some(val) = last.as_os_str().to_str() {
                if let Ok(id) = val.parse::<u32>() { return id; }
            }
        }
    }
    if let Ok(path) = std::env::var("STEAM_COMPAT_MEDIA_PATH") {
        let parts: Vec<_> = Path::new(&path).components().collect();
        if parts.len() >= 2 {
            if let Some(val) = parts[parts.len() - 2].as_os_str().to_str() {
                if let Ok(id) = val.parse::<u32>() { return id; }
            }
        }
    }
    if let Ok(path) = std::env::var("STEAM_FOSSILIZE_DUMP_PATH") {
        let parts: Vec<_> = Path::new(&path).components().collect();
        if parts.len() >= 3 {
            if let Some(val) = parts[parts.len() - 3].as_os_str().to_str() {
                if let Ok(id) = val.parse::<u32>() { return id; }
            }
        }
    }
    if let Ok(path) = std::env::var("DXVK_STATE_CACHE_PATH") {
        let parts: Vec<_> = Path::new(&path).components().collect();
        if parts.len() >= 2 {
            if let Some(val) = parts[parts.len() - 2].as_os_str().to_str() {
                if let Ok(id) = val.parse::<u32>() { return id; }
            }
        }
    }
    if let Ok(id_str) = std::env::var("SteamGameId") {
        if let Ok(id) = id_str.parse::<u64>() { return (id >> 32) as u32; }
    }
    0
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

#[derive(Serialize, Deserialize, Debug)]
pub struct DownloadSizesRsp {
    pub game_decompressed_size: String,
    pub free_disk_space: String,
    pub game_decompressed_size_raw: u64,
    pub free_disk_space_raw: u64
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResumeStatesRsp {
    pub downloading: bool,
    pub updating: bool,
    pub preloading: bool,
    pub repairing: bool
}