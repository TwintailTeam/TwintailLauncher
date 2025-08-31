use std::{fs, io};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use fischl::utils::{extract_archive, get_github_release};
use fischl::download::Extras;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Listener, Manager};
use tauri_plugin_notification::NotificationExt;
use crate::utils::db_manager::{get_installs, get_manifest_info_by_id, get_settings, update_install_use_jadeite_by_id, update_settings_default_fps_unlock_location, update_settings_default_game_location, update_settings_default_xxmi_location};

#[cfg(target_os = "linux")]
use crate::utils::repo_manager::get_manifests;
#[cfg(target_os = "linux")]
use crate::utils::db_manager::{update_settings_default_jadeite_location, update_settings_default_prefix_location, update_settings_default_runner_location, update_settings_default_dxvk_location};
#[cfg(target_os = "linux")]
use libc::{getrlimit, rlim_t, rlimit, setrlimit, RLIMIT_NOFILE};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

pub mod db_manager;
pub mod repo_manager;
mod git_helpers;
pub mod game_launch_manager;
pub mod system_tray;
pub mod args;
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

    prevent_exit(app, true);
    for entry in fs::read_dir(src.as_ref())? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let f = entry.file_name();
        let ep = entry.path();

        if ep == dst.as_ref() { continue; }

        if ty.is_dir() {
            copy_dir_all(&app, ep.clone(), dst.as_ref().join(f), install.clone(), install_name.clone(), install_type.clone())?;
            fs::remove_dir_all(ep)?;
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

            fs::copy(ep.clone(), dst.as_ref().join(f))?;
            app.emit("move_progress", &payload).unwrap();
            fs::remove_file(ep)?;
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn block_telemetry(app: &AppHandle) {
    let app1 = Arc::new(Mutex::new(app.clone()));
        std::thread::spawn(move || {
            let app = app1.lock().unwrap().clone();
            let manifests = get_manifests(&app);
            let mut allhosts = String::new();

            manifests.values().for_each(|manifest| {
                let hosts = manifest.telemetry_hosts.iter().map(|server| format!("echo '0.0.0.0 {server}' >> /etc/hosts")).collect::<Vec<String>>().join(" ; ");
                allhosts.push_str(&hosts);
                allhosts.push_str(" ; ");
            });

            if !allhosts.is_empty() { allhosts = allhosts.trim_end_matches(" ; ").to_string(); }

            let output = std::process::Command::new("pkexec").env("PKEXEC_DESCRIPTION", "TwintailLauncher wants to block game telemetry servers")
                .arg("bash").arg("-c").arg(format!("echo '' >> /etc/hosts ; echo '# TwintailLauncher telemetry block start' >> /etc/hosts ; {allhosts} ; echo '# TwintailLauncher telemetry block end' >> /etc/hosts")).spawn();

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
        if state.action_exit { h1.get_window("main").unwrap().hide().unwrap(); } else { h1.cleanup_before_exit();h1.exit(0);std::process::exit(0); }
    });

    let h2 = app.clone();
    app.listen("launcher_action_minimize", move |_event| { h2.get_window("main").unwrap().hide().unwrap(); });
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
        if runner_version.contains("proton-cachyos-spritz") {
            rslt = "proton_cachyos_spritz.json".to_string();
        }
        if runner_version.contains("proton-umu") {
            rslt = "proton_umu.json".to_string();
        }
        if runner_version.contains("proton-vanilla") {
            rslt = "proton_vanilla.json".to_string();
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
    let defgpath = path.join("games").follow_symlink().unwrap();
    let xxmipath = path.join("extras").join("xxmi").follow_symlink().unwrap();
    let fpsunlockpath = path.join("extras").join("fps_unlock").follow_symlink().unwrap();

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
                let comppath = path.join("compatibility").follow_symlink().unwrap();
                let wine = comppath.join("runners").follow_symlink().unwrap();
                let dxvk = comppath.join("dxvk").follow_symlink().unwrap();
                let prefixes = comppath.join("prefixes").follow_symlink().unwrap();
                let jadeitepath = path.join("extras").join("jadeite").follow_symlink().unwrap();
                let mangohudcfg = path.join("mangohud_default.conf").follow_symlink().unwrap();

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
            let comppath = path.join("compatibility").follow_symlink().unwrap();
            let wine = comppath.join("runners").follow_symlink().unwrap();
            let dxvk = comppath.join("dxvk").follow_symlink().unwrap();
            let prefixes = comppath.join("prefixes").follow_symlink().unwrap();
            let jadeitepath = path.join("extras").join("jadeite").follow_symlink().unwrap();
            let mangohudcfg = path.join("mangohud_default.conf").follow_symlink().unwrap();

            if !mangohudcfg.exists() { db_manager::update_settings_default_mangohud_config_location(app, mangohudcfg.to_str().unwrap().to_string()); }
            if !jadeitepath.exists() { fs::create_dir_all(&jadeitepath).unwrap(); update_settings_default_jadeite_location(app, jadeitepath.to_str().unwrap().to_string()); }
            if !comppath.exists() {
                fs::create_dir_all(&wine).unwrap();
                fs::create_dir_all(&dxvk).unwrap();
                fs::create_dir_all(&prefixes).unwrap();
                update_settings_default_runner_location(app, wine.to_str().unwrap().to_string());
                update_settings_default_dxvk_location(app, dxvk.to_str().unwrap().to_string());
                update_settings_default_prefix_location(app, prefixes.to_str().unwrap().to_string());
            }
        }
    }
}

pub fn download_or_update_jadeite(path: PathBuf, update_mode: bool) {
    if update_mode {
        if fs::read_dir(&path).unwrap().next().is_some() {
            std::thread::spawn(move || {
                let dl = Extras::download_jadeite("MrLGamer/jadeite".parse().unwrap(), path.as_path().to_str().unwrap().parse().unwrap(), move |_current, _total| {});
                if dl { extract_archive("".to_string(), path.join("jadeite.zip").as_path().to_str().unwrap().parse().unwrap(), path.as_path().to_str().unwrap().parse().unwrap(), false); }
            });
        }
    } else {
        if fs::read_dir(&path).unwrap().next().is_none() {
            std::thread::spawn(move || {
                let dl = Extras::download_jadeite("MrLGamer/jadeite".parse().unwrap(), path.as_path().to_str().unwrap().parse().unwrap(), move |_current, _total| {});
                if dl { extract_archive("".to_string(), path.join("jadeite.zip").as_path().to_str().unwrap().parse().unwrap(), path.as_path().to_str().unwrap().parse().unwrap(), false); }
            });
        }
    }
}

pub fn download_or_update_fps_unlock(path: PathBuf, update_mode: bool) {
    if update_mode {
        if fs::read_dir(&path).unwrap().next().is_some() {
            std::thread::spawn(move || { Extras::download_fps_unlock("TwintailTeam/KeqingUnlock".parse().unwrap(), path.as_path().to_str().unwrap().parse().unwrap(), move |_current, _total| {}); });
        }
    } else {
        if fs::read_dir(&path).unwrap().next().is_none() {
            std::thread::spawn(move || { Extras::download_fps_unlock("TwintailTeam/KeqingUnlock".parse().unwrap(), path.as_path().to_str().unwrap().parse().unwrap(), move |_current, _total| {}); });
        }
    }
}

pub fn download_or_update_xxmi(app: &AppHandle, path: PathBuf, update_mode: bool) {
    if update_mode {
        if fs::read_dir(&path).unwrap().next().is_some() {
            std::thread::spawn(move || {
                let dl = Extras::download_xxmi("SpectrumQT/XXMI-Libs-Package".parse().unwrap(), path.as_path().to_str().unwrap().parse().unwrap(), false, move |_current, _total| {});
                if dl {
                    extract_archive("".to_string(), path.join("xxmi.zip").as_path().to_str().unwrap().parse().unwrap(), path.as_path().to_str().unwrap().parse().unwrap(), false);
                    let gimi = String::from("SilentNightSound/GIMI-Package");
                    let srmi = String::from("SpectrumQT/SRMI-Package");
                    let zzmi = String::from("leotorrez/ZZMI-Package");
                    let wwmi = String::from("SpectrumQT/WWMI-Package");
                    let himi = String::from("leotorrez/HIMI-Package");

                    let dl1 = Extras::download_xxmi_packages(gimi, srmi, zzmi, wwmi, himi, path.as_path().to_str().unwrap().parse().unwrap());
                    if dl1 {
                        for mi in ["gimi", "srmi", "zzmi", "wwmi", "himi"] {
                            extract_archive("".to_string(), path.join(format!("{mi}.zip")).as_path().to_str().unwrap().parse().unwrap(), path.join(mi).as_path().to_str().unwrap().parse().unwrap(), false);
                            for lib in ["d3d11.dll", "d3dcompiler_47.dll"] {
                                let linkedpath = path.join(mi).join(lib);
                                if !linkedpath.exists() {
                                    #[cfg(target_os = "linux")]
                                    std::os::unix::fs::symlink(path.join(lib), linkedpath).unwrap();
                                    #[cfg(target_os = "windows")]
                                    fs::copy(path.join(lib), linkedpath).unwrap();
                                }
                            }
                        }
                    }
                }
            });
        }
    } else {
        if fs::read_dir(&path).unwrap().next().is_none() {
            let app = app.clone();
            std::thread::spawn(move || {
                let app = app.clone();
                let mut dlpayload = HashMap::new();
                dlpayload.insert("name", String::from("XXMI Modding tool"));
                dlpayload.insert("progress", "0".to_string());
                dlpayload.insert("total", "1000".to_string());
                app.emit("download_progress", dlpayload.clone()).unwrap();
                prevent_exit(&app, true);
                let dl = Extras::download_xxmi("SpectrumQT/XXMI-Libs-Package".parse().unwrap(), path.as_path().to_str().unwrap().parse().unwrap(), true, {
                    let app = app.clone();
                    let dlpayload = dlpayload.clone();
                    move |current, total| {
                        let mut dlpayload = dlpayload.clone();
                        dlpayload.insert("name", "XXMI Modding tool".to_string());
                        dlpayload.insert("progress", current.to_string());
                        dlpayload.insert("total", total.to_string());
                        app.emit("download_progress", dlpayload.clone()).unwrap();
                    }
                });
                if dl {
                    extract_archive("".to_string(), path.join("xxmi.zip").as_path().to_str().unwrap().parse().unwrap(), path.as_path().to_str().unwrap().parse().unwrap(), false);
                    let gimi = String::from("SilentNightSound/GIMI-Package");
                    let srmi = String::from("SpectrumQT/SRMI-Package");
                    let zzmi = String::from("leotorrez/ZZMI-Package");
                    let wwmi = String::from("SpectrumQT/WWMI-Package");
                    let himi = String::from("leotorrez/HIMI-Package");

                    let dl1 = Extras::download_xxmi_packages(gimi, srmi, zzmi, wwmi, himi, path.as_path().to_str().unwrap().parse().unwrap());
                    if dl1 {
                        for mi in ["gimi", "srmi", "zzmi", "wwmi", "himi"] {
                            extract_archive("".to_string(), path.join(format!("{mi}.zip")).as_path().to_str().unwrap().parse().unwrap(), path.join(mi).as_path().to_str().unwrap().parse().unwrap(), false);
                            for lib in ["d3d11.dll", "d3dcompiler_47.dll"] {
                                let linkedpath = path.join(mi).join(lib);
                                if !linkedpath.exists() {
                                    #[cfg(target_os = "linux")]
                                    std::os::unix::fs::symlink(path.join(lib), linkedpath).unwrap();
                                    #[cfg(target_os = "windows")]
                                    fs::copy(path.join(lib), linkedpath).unwrap();
                                }
                            }
                        }
                        app.emit("download_complete", String::from("XXMI Modding tool")).unwrap();
                        prevent_exit(&app, false);
                    }
                }
            });
        }
    }
}

pub fn deprecate_jadeite(app: &AppHandle) {
    let installs = get_installs(app);
    if installs.is_some() {
        let i = installs.unwrap();
        for ci in i {
            let im = get_manifest_info_by_id(&app, ci.manifest_id);
            if im.is_some() {
                let lm = im.unwrap();
                // Shit validation but will work
                if lm.display_name.to_ascii_lowercase().contains("wuthering") { update_install_use_jadeite_by_id(&app, ci.id, false); }
            }
        }
    }
}

#[cfg(target_os = "linux")]
pub fn raise_fd_limit(new_limit: i32) {
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
        let v = r.tag_name.replace("ttl-v", "");
        let suppressed = app.path().app_data_dir().unwrap().join(".updatenaghide");
        if !suppressed.exists() {
            let cfg = app.config();
            if cfg.version.clone().unwrap() < v {
                app.dialog().message("You are running outdated version of TwintailLauncher!\nYou can still continue to use currently installed version, however we recommend updating to the latest version for best experience.").title("Update available")
                    .kind(MessageDialogKind::Info)
                    .buttons(MessageDialogButtons::OkCustom("Continue anyway".to_string()))
                    //.buttons(MessageDialogButtons::OkCancelCustom("Continue anyway".to_string(), "Do not show again".to_string()))
                    .show(move |_action| {
                        //if action {  } else { fs::File::create(&suppressed).unwrap(); }
                    });
            }
        }
    }
}

#[cfg(target_os = "linux")]
pub fn is_flatpak() -> bool { std::env::var("FLATPAK_ID").is_ok() }

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

pub trait PathResolve {
    fn follow_symlink(&self) -> io::Result<PathBuf>;
}

impl PathResolve for Path {
    fn follow_symlink(&self) -> io::Result<PathBuf> {
        #[cfg(target_os = "linux")]
        return if self.is_symlink() { self.read_link() } else { Ok(self.to_path_buf()) };
        #[cfg(target_os = "windows")]
        return Ok(self.to_path_buf())
    }
}
