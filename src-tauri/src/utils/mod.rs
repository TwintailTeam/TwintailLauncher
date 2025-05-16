use std::{fs, io};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};
use fischl::download::game::{Game, Hoyo, Kuro, Sophon};
use fischl::utils::game::VoiceLocale;
use fischl::utils::KuroFile;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Listener, Manager};
use crate::utils::db_manager::{get_install_info_by_id, get_manifest_info_by_id};
use crate::utils::repo_manager::{get_manifest, get_manifests, GameVersion};

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

#[cfg(target_os = "linux")]
pub fn block_telemetry(app: &AppHandle) {
    let app1 = Arc::new(Mutex::new(app.clone()));
        std::thread::spawn(move || {
            let app = app1.lock().unwrap().clone();
            let manifests = get_manifests(&app);
            let mut allhosts = String::new();

            manifests.values().for_each(|manifest| {
                // Thanks to certain anime team for some of this lol
                let hosts = manifest.telemetry_hosts.iter().map(|server| format!("echo '0.0.0.0 {server}' >> /etc/hosts")).collect::<Vec<String>>().join(" ; ");
                allhosts.push_str(&hosts);
                allhosts.push_str(" ; ");
            });

            if !allhosts.is_empty() {
                allhosts = allhosts.trim_end_matches(" ; ").to_string();
            }

            let output = Command::new("pkexec")
                .arg("bash").arg("-c").arg(format!("echo '' >> /etc/hosts ; echo '# KeqingLauncher telemetry block start' >> /etc/hosts ; {allhosts} ; echo '# KeqingLauncher telemetry block end' >> /etc/hosts")).spawn();

            match output.and_then(|child| child.wait_with_output()) {
                Ok(output) => if !output.status.success() {
                    app.emit("telemetry_block", 0).unwrap();
                } else {
                    let path = app.path().app_data_dir().unwrap().join(".telemetry_blocked");
                    if !path.exists() {
                        app.emit("telemetry_block", 1).unwrap();
                        fs::write(&path, ".").unwrap();
                    } else {
                        app.emit("telemetry_block", 2).unwrap();
                    }
                }
                Err(_err) => { app.emit("telemetry_block", 0).unwrap(); }
            }
        });
}

#[cfg(target_os = "windows")]
pub fn block_telemetry(_app: &AppHandle) {

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

    // Start game download
    let h4 = app.clone();
    app.listen("start_game_download", move |event| {
        let h4 = h4.clone();
        std::thread::spawn(async move || {
            let payload: DownloadGamePayload = serde_json::from_str(event.payload()).unwrap();
            let install = get_install_info_by_id(&h4, payload.install).unwrap(); // Should exist by now, if not we FUCKED UP
            let gid = payload.biz.clone() + ".json";

            let mm = get_manifest(&h4, gid);
            if let Some(gm) = mm {
                let version = gm.game_versions.iter().filter(|e| e.metadata.version == install.version).collect::<Vec<&GameVersion>>();
                let picked = version.get(0).unwrap();
                let tmp = Arc::new(h4.clone());

                let instn = Arc::new(install.name.clone());
                let tracker = Arc::new(Mutex::new(0));
                let tc = Arc::clone(&tracker);

                h4.emit("download_progress", install.name.clone()).unwrap();

                match picked.metadata.download_mode.as_str() {
                    // Generic zipped mode, PS: Currently only hoyo for backwards compatibility
                    "DOWNLOAD_MODE_FILE" => {
                        /*let mut urls = picked.game.full.iter().map(|v| v.file_url.clone()).collect::<Vec<String>>();
                        if !picked.audio.full.is_empty() {
                            let faudio: Vec<_> = picked.audio.full.iter().filter(|v| v.language == install.audio_langs).collect();
                            urls.push(faudio.get(0).unwrap().file_url.clone());
                        }
                        <Game as Hoyo>::download(urls.clone(), install.directory.clone(), move |_, _| {
                            let mut tracker = tc.lock().unwrap();
                            *tracker += 1;
                            tmp.emit("download_progress", instn.as_ref()).unwrap();
                        });
                        if *tracker.lock().unwrap() == urls.clone().len() {
                            h4.emit("download_complete", install.name.clone()).unwrap();
                        }*/
                    }
                    // Sophon chunk mode, PS: Only hoyo supported as it is their literal format
                    "DOWNLOAD_MODE_CHUNK" => {
                        let urls = picked.game.full.iter().map(|v| v.file_url.clone()).collect::<Vec<String>>();
                        let manifest = urls.get(0).unwrap();
                        <Game as Sophon>::download(manifest.to_owned(), picked.metadata.res_list_url.clone(), install.directory.clone(), move |_, _| {
                            let mut tracker = tc.lock().unwrap();
                            *tracker += 1;
                            tmp.emit("download_progress", instn.as_ref()).unwrap();
                        }).await;
                        // Shitty way to validate but will work for the time being
                        if *tracker.lock().unwrap() <= 3000 || *tracker.lock().unwrap() >= 3000 {
                            h4.emit("download_complete", install.name.clone()).unwrap();
                        }
                    }
                    // Raw file mode, PS: Currently only wuwa supported! PGR soon???
                    "DOWNLOAD_MODE_RAW" => {
                        let urls = picked.game.full.iter().map(|v| KuroFile { url: v.file_url.clone(), path: v.file_path.clone(), hash: v.file_hash.clone(), size: v.decompressed_size.clone() }).collect::<Vec<KuroFile>>();
                        <Game as Kuro>::download(urls.clone(), install.directory.clone(), move |_, _| {
                            let mut tracker = tc.lock().unwrap();
                            *tracker += 1;
                            tmp.emit("download_progress", instn.as_ref()).unwrap();
                        });
                        if *tracker.lock().unwrap() == urls.clone().len() {
                            h4.emit("download_complete", install.name.clone()).unwrap();
                        }
                    }
                    // Fallback mode... NOT IMPLEMENTED AS I DID NOT WRITE ANY IN THE LIBRARY
                    _ => {}
                }
            } else {
                println!("Failed to download game!");
            }
        });
    });

    // Start game repair
    let h5 = app.clone();
    app.listen("start_game_repair", move |event| {
        let h5 = h5.clone();
        std::thread::spawn(move || {
            let payload: DownloadGamePayload = serde_json::from_str(event.payload()).unwrap();
            let install = get_install_info_by_id(&h5, payload.install); // Should exist by now, if not we FUCKED UP
            let lm = get_manifest_info_by_id(&h5, payload.biz).unwrap();
            let gm = get_manifest(&h5, lm.filename).unwrap();

            if install.is_some() { 
                let i = install.unwrap();
                let version = gm.game_versions.iter().filter(|e| e.metadata.version == i.version).collect::<Vec<&GameVersion>>();
                let picked = version.get(0).unwrap();

                let tmp = Arc::new(h5.clone());
                let instn = Arc::new(i.name.clone());

                h5.emit("repair_progress", instn.as_ref()).unwrap();

                match picked.metadata.download_mode.as_str() {
                    // General game repair, PS: Only hoyo games for backwards compatibility
                    "DOWNLOAD_MODE_FILE" => {
                        let rslt = <Game as Hoyo>::repair_game(picked.metadata.res_list_url.clone(), i.directory.clone(), i.skip_hash_check, move |_, _| {
                            tmp.emit("repair_progress", instn.as_ref()).unwrap();
                        });
                        if rslt {
                            if !gm.paths.audio_pkg_res_dir.clone().is_empty() {
                                let dir = Path::new(&i.directory.clone()).join(gm.paths.audio_pkg_res_dir.clone());

                                // Make this shit better as some folder names might be pulled as inaccurate
                                let locales = vec![
                                    (VoiceLocale::English, if gm.biz.contains("hk4e") { dir.join(&VoiceLocale::English.to_folder()) } else { dir.join(&VoiceLocale::English.to_name()) }),
                                    (VoiceLocale::Korean, dir.join(&VoiceLocale::Korean.to_name())),
                                    (VoiceLocale::Japanese, dir.join(&VoiceLocale::Japanese.to_name())),
                                    (VoiceLocale::Chinese, dir.join(&VoiceLocale::Chinese.to_name())),
                                ];

                                let instn1 = Arc::new(i.name.clone());
                                let tmp1 = Arc::new(h5.clone());
                                // Loop over all available locales and check if their Audio pkg folder exists, if it does start repair for the language
                                for (locale, path) in locales {
                                    if path.exists() {
                                        let instn1c = instn1.clone();
                                        let tmp1c = tmp1.clone();
                                        let l = if gm.biz.contains("hk4e") { locale.to_folder() } else { locale.to_name() };

                                        let rslt1 = <Game as Hoyo>::repair_audio(picked.metadata.res_list_url.clone(), l.to_string(), i.directory.clone(), i.skip_hash_check, move |_, _| {
                                            tmp1c.emit("repair_progress", instn1c.as_ref()).unwrap();
                                        });
                                        if rslt1 { h5.emit("repair_complete", i.name.clone()).unwrap(); }
                                    }
                                }
                            } else { h5.emit("repair_complete", i.name.clone()).unwrap(); };
                        }
                    }
                    // Sophon chunk repair, PS: Only hoyo games as it is their literal format
                    "DOWNLOAD_MODE_CHUNK" => {}
                    // Raw file repair, PS: Only wuwa currently
                    "DOWNLOAD_MODE_RAW" => {
                        let rslt = <Game as Kuro>::repair_game(picked.metadata.index_file.clone(), picked.metadata.res_list_url.clone(), i.directory, i.skip_hash_check, move |_, _| {
                            tmp.emit("repair_progress", instn.as_ref()).unwrap();
                        });
                        if rslt { h5.emit("repair_complete", i.name.clone()).unwrap(); }
                    }
                    // Fallback mode
                    _ => {}
                }
            } else { 
                println!("Failed to find installation for repair!");
            }
            
        });
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
        if runner_version.contains("proton-umu") {
            rslt = "proton_umu.json".to_string();
        }
        Some(rslt)
    }
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
pub struct DownloadGamePayload {
    pub install: String,
    pub biz: String,
    pub lang: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DownloadSizesRsp {
    pub game_decompressed_size: String,
    pub free_disk_space: String,
    pub game_decompressed_size_raw: u64,
    pub free_disk_space_raw: u64
}