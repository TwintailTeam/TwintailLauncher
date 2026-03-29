use crate::utils::models::{GameManifest, GlobalSettings, LauncherInstall};
use crate::utils::{apply_xxmi_tweaks,get_mi_path_from_game,prevent_system_idle,show_dialog};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Error};
use crate::utils::db_manager::{update_install_last_played_by_id,update_install_total_playtime_by_id};
use crate::utils::discord_rpc;
use fischl::utils::is_process_running;

#[cfg(target_os = "linux")]
use crate::utils::{get_steam_appid, get_steam_tool_appid, is_runner_lower, is_using_overriden_runner, runner_from_runner_version, update_steam_compat_config, repo_manager::get_compatibility};
#[cfg(target_os = "linux")]
use std::os::unix::process::CommandExt;
#[cfg(target_os = "linux")]
use tauri::Manager;

#[cfg(target_os = "linux")]
pub fn launch(app: &AppHandle, install: LauncherInstall, gm: GameManifest, gs: GlobalSettings) -> Result<bool, Error> {
    let Some(rm) = get_compatibility(&app, &runner_from_runner_version(app, install.runner_version.clone()).unwrap_or_default()) else { return Ok(false); };
    let is_proton = rm.display_name.to_ascii_lowercase().contains("proton") && !rm.display_name.to_ascii_lowercase().contains("wine");
    let mut compat_config = update_steam_compat_config(vec![]);
    let cpo = gm.extra.compat_overrides.clone();

    let dirp = Path::new(install.directory.as_str());
    let dir = dirp.to_str().unwrap().to_string();
    let prefixp = Path::new(install.runner_prefix.as_str()).to_path_buf();
    let prefix = prefixp.to_str().unwrap().to_string();
    let runnerp = Path::new(gs.default_runner_path.as_str()).to_path_buf();
    let runnerpi = Path::new(install.runner_path.as_str()).to_path_buf();
    let runner = runnerpi.to_str().unwrap().to_string();
    let game = gm.paths.exe_filename.clone();
    let exe = gm.paths.exe_filename.clone().split('/').last().unwrap().to_string();
    let toolid = get_steam_tool_appid(runnerpi);
    let steamrtpp = runnerp.join("steamrt/").join(toolid.clone());
    let steamrt_path = steamrtpp.to_str().unwrap().to_string();
    let steamrtp = steamrtpp.join("_v2-entry-point");
    let steamrt = steamrtp.to_str().unwrap().to_string();
    #[cfg(not(debug_assertions))]
    let reaper = if crate::utils::is_flatpak() { app.path().resource_dir()?.join("resources/reaper").to_str().unwrap().to_string().replace("/app/lib/", "/run/parent/app/lib/") } else { app.path().resource_dir()?.join("resources/reaper").to_str().unwrap().to_string().replace("/usr/lib/", "/run/host/usr/lib/") };
    #[cfg(debug_assertions)]
    let reaper = app.path().resource_dir()?.join("resources/reaper").to_str().unwrap().to_string();
    let appid = get_steam_appid();

    if is_runner_lower(cpo.min_runner_versions.clone(), install.clone().runner_version, ) && !cpo.min_runner_versions.is_empty() {
        log::info!("Attempted to launch {} with runner version {} which is lower than the minimum required runner version(s) of {}!", install.name, install.runner_version, cpo.min_runner_versions.join(", "));
        show_dialog(app, "warning", "TwintailLauncher", &format!("Launching {} with {} could lead to various unexpected behaviors.\nPlease download one of the supported minimum runner versions or higher!\nSupported minimum runner version(s): {}", install.name, install.runner_version, cpo.min_runner_versions.join(", ")), Some(vec!["I understand"]));
        return Ok(false);
    }

    if cpo.override_runner.linux.enabled && !cpo.override_runner.linux.runner_version.is_empty() && is_using_overriden_runner(install.runner_version.clone(), cpo.override_runner.linux.runner_version.clone(), ) {
        log::info!("Attempted to launch {} with runner version {} while compatibility override is set to {}!", install.name, install.runner_version, cpo.override_runner.linux.runner_version);
        show_dialog(app, "warning", "TwintailLauncher", &format!("Launching {} without using {} could lead to various issues.\nPlease change your runner to at minimum {} and try again!", install.name, cpo.override_runner.linux.runner_version, cpo.override_runner.linux.runner_version), Some(vec!["I understand"]));
        return Ok(false);
    }

    // If prefix folder somehow does not exist remake it
    if !prefixp.exists() {
        if let Err(e) = fs::create_dir_all(&prefixp) {
            log::error!("Failed to create missing runner prefix folder at {}! Error: {}", prefixp.to_str().unwrap(), e.to_string());
            show_dialog(app, "warning", "TwintailLauncher", &format!("Encountered an error while trying to reinitialize your runner prefix! - {err}!", err = e.to_string()), Some(vec!["I understand"]));
            return Ok(false);
        };
    }

    let pre_launch = install.pre_launch_command.clone();
    let wine64 = if rm.paths.wine64.is_empty() { rm.paths.wine32.clone() } else { rm.paths.wine64.clone() };
    let can_game_launch: Option<std::thread::JoinHandle<bool>> = if !prefixp.join("pfx").join("drive_c").exists() && !cpo.winetricks_verbs.is_empty() { Some(run_winetricks(app, install.clone(), steamrt.clone(), reaper.clone(), appid, runner.clone(), wine64.clone(), prefix.clone(), dir.clone(), cpo.winetricks_verbs.clone())) } else { None };

    // Wait for winetricks to fully exit before proceeding to game launch
    if let Some(handle) = can_game_launch { if !handle.join().unwrap_or(false) { return Ok(false); } }

    if !pre_launch.is_empty() {
        let command = format!("{pre_launch}").replace("%appid%", appid.clone().to_string().as_str()).replace("%reaper%", reaper.clone().as_str()).replace("%steamrt_path%", steamrt_path.clone().as_str()).replace("%steamrt%", steamrt.clone().as_str()).replace("%prefix%", prefix.clone().as_str()).replace("%runner_dir%", runner.clone().as_str()).replace("%runner%", &*(runner.clone() + "/" + wine64.as_str())).replace("%install_dir%", dir.clone().as_str()).replace("%game_exe%", &*(dir.clone() + "/" + exe.clone().as_str()));

        let mut cmd = Command::new("bash");
        cmd.arg("-c");
        cmd.arg(&command);

        cmd.env("WINEARCH", "win64");
        cmd.env("WINEPREFIX", prefix.clone() + "/pfx");
        cmd.env("STEAM_COMPAT_APP_ID", "0");
        cmd.env("STEAM_COMPAT_DATA_PATH", prefix.clone());
        cmd.env("STEAM_COMPAT_INSTALL_PATH", dir.clone());
        cmd.env("STEAM_COMPAT_CLIENT_INSTALL_PATH", "");
        cmd.env("STEAM_COMPAT_TOOL_PATHS", runner.clone());
        cmd.env("STEAM_COMPAT_SHADER_PATH", prefix.clone() + "/shadercache");
        cmd.env("WINEDLLOVERRIDES", "lsteamclient=d;KRSDKExternal.exe=d");
        if cpo.disable_protonfixes { cmd.env("PROTONFIXES_DISABLE", "1"); }
        if !cpo.protonfixes_store.is_empty() { cmd.env("STORE", cpo.protonfixes_store.clone()); }
        if !cpo.protonfixes_id.is_empty() { cmd.env("UMU_ID", cpo.protonfixes_id.clone()); }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.current_dir(dir.clone());
        cmd.process_group(0);

        match cmd.spawn() {
            Ok(mut child) => match child.try_wait() {
                Ok(Some(status)) => {
                    if !status.success() { log::info!("Executing prelaunch command: \"{}\" failed with status: {}", command, status.code().unwrap()); show_dialog(&app, "error", "TwintailLauncher", "Failed to execute prelaunch command! Please try again or check game settings.", None); }
                }
                Ok(None) => {
                    log::info!("Executing prelaunch command: \"{}\" detailed output of the command is available at {}", command, Path::new(&dir).join("pre_launch.log").to_str().unwrap());
                    write_log(app, Path::new(&dir).to_path_buf(), child, "pre_launch.log".parse().unwrap());
                }
                Err(_) => { show_dialog(&app, "error", "TwintailLauncher", "Failed to execute prelaunch command! Please try again or check the command correctness.", None); }
            },
            Err(_) => { log::error!("Executing prelaunch command \"{}\" failed catastrophically!", command); show_dialog(&app, "error", "TwintailLauncher", "Failed to execute prelaunch command! Something serious is wrong.", None); }
        }
    }

    let verb = if install.use_xxmi || install.use_fps_unlock { "run" } else { "waitforexitandrun" };
    let drive = if cpo.proton_compat_config.contains(&"gamedrive".to_string()) { format!("s:\\{game}") } else { format!("z:\\{dir}/{game}") };
    let rslt = if install.launch_command.is_empty() {
        let mut args = install.launch_args.clone();
        let xxmi_forced = install.use_xxmi && (gm.biz == "wuwa_global" || gm.biz == "endfield_global");
        if install.use_xxmi && gm.biz == "wuwa_global" { args = args.split_whitespace().filter(|a| gm.extra.graphics_api_options.options.iter().all(|o| o.value.as_str() != *a)).collect::<Vec<_>>().join(" "); if !args.is_empty() { args += " "; } args += "-dx11"; }
        if install.use_xxmi && gm.biz == "endfield_global" { args = args.split_whitespace().filter(|a| gm.extra.graphics_api_options.options.iter().all(|o| o.value.as_str() != *a)).collect::<Vec<_>>().join(" "); if !args.is_empty() { args += " "; } args += "-force-d3d11"; }
        if gm.extra.switches.graphics_api && !xxmi_forced && !args.split_whitespace().any(|a| gm.extra.graphics_api_options.options.iter().any(|o| o.value.as_str() == a)) && !install.graphics_api.is_empty() { if !args.is_empty() { args += " "; } args += &install.graphics_api; }

        let mut command = if is_proton {
            let steamrt_run = format!("'{steamrt}' --verb={verb} -- '{reaper}' SteamLaunch AppId={appid} -- '{runner}/{wine64}' {verb} '{drive}' {args}");
            if install.use_gamemode { format!("gamemoderun {steamrt_run}") } else { format!("{steamrt_run}") }
        } else {
            if install.use_gamemode { format!("gamemoderun '{runner}/{wine64}' '{dir}/{game}' {args}") } else { format!("'{runner}/{wine64}' '{dir}/{game}' {args}") }
        };

        if install.use_jadeite {
            let jadeite_path = gs.jadeite_path.clone();
            command = if is_proton {
                let steamrt_run = format!("'{steamrt}' --verb={verb} -- '{reaper}' SteamLaunch AppId={appid} -- '{runner}/{wine64}' {verb} 'z:\\{jadeite_path}/jadeite.exe' '{dir}/{game}' -- {args}");
                if install.use_gamemode { format!("gamemoderun {steamrt_run}") } else { format!("{steamrt_run}") }
            } else {
                if install.use_gamemode { format!("gamemoderun '{runner}/{wine64}' '{jadeite_path}/jadeite.exe' '{dir}/{game}' -- {args}") } else { format!("'{runner}/{wine64}' '{jadeite_path}/jadeite.exe' '{dir}/{game}' -- {args}") }
            };
        }

        let mut cmd = Command::new("bash");
        cmd.arg("-c");
        cmd.arg(&command);

        cmd.env("SteamOS", "1");
        cmd.env("WINEARCH", "win64");
        cmd.env("WINEPREFIX", prefix.clone() + "/pfx");
        cmd.env("STEAM_COMPAT_APP_ID", "0");
        cmd.env("STEAM_COMPAT_DATA_PATH", prefix.clone());
        cmd.env("STEAM_COMPAT_INSTALL_PATH", dir.clone());
        cmd.env("STEAM_COMPAT_CLIENT_INSTALL_PATH", "");
        cmd.env("STEAM_COMPAT_TOOL_PATHS", runner.clone());
        cmd.env("STEAM_COMPAT_LIBRARY_PATHS", format!("{dir}:{prefix}/pfx"));
        cmd.env("STEAM_COMPAT_SHADER_PATH", prefix.clone() + "/shadercache");
        cmd.env("WINEDLLOVERRIDES", "lsteamclient=d;KRSDKExternal.exe=d");
        if cpo.stub_wintrust { cmd.env("STUB_WINTRUST", "1"); }
        if cpo.block_first_req { cmd.env("BLOCK_FIRST_REQ", "1"); }
        if cpo.disable_protonfixes { cmd.env("PROTONFIXES_DISABLE", "1"); }
        if !cpo.protonfixes_store.is_empty() { cmd.env("STORE", cpo.protonfixes_store); }
        if !cpo.protonfixes_id.is_empty() { cmd.env("UMU_ID", cpo.protonfixes_id); }
        if !cpo.proton_compat_config.is_empty() { compat_config = update_steam_compat_config(cpo.proton_compat_config.iter().map(String::as_str).collect()); }
        if cpo.stub_wintrust || cpo.block_first_req { cmd.env("WINEDLLOVERRIDES", "lsteamclient=d;KRSDKExternal.exe=d;jsproxy=n,b"); crate::utils::apply_patch(app, Path::new(&dir.clone()).to_str().unwrap().to_string(), "sparkle".to_string(), "add".to_string()); } else if !cpo.stub_wintrust && !cpo.block_first_req { crate::utils::apply_patch(app, Path::new(&dir.clone()).to_str().unwrap().to_string(), "sparkle".to_string(), "remove".to_string()); }
        cmd.env("STEAM_COMPAT_CONFIG", compat_config);
        if install.use_mangohud {
            cmd.env("MANGOHUD", "1");
            if install.mangohud_config_path != "" { cmd.env("MANGOHUD_CONFIGFILE", format!("{}", install.clone().mangohud_config_path).as_str()); }
        }
        /*if gm.biz == "wuwa_global" {
            if install.use_xxmi {
                let engine_file = dirp.join("Client/Saved/Config/WindowsNoEditor/Engine.ini");
                let device_profiles_file = dirp.join("Client/Saved/Config/WindowsNoEditor/DeviceProfiles.ini");
                edit_wuwa_configs_xxmi(engine_file.to_str().unwrap().to_string(), device_profiles_file.to_str().unwrap().to_string());
            }
        }*/

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.current_dir(dir.clone());
        cmd.process_group(0);

        if !install.env_vars.is_empty() {
            let envs = install.env_vars.clone();
            let splitted = envs.split(";").collect::<Vec<&str>>();
            let parsed: Option<Vec<(&str, String)>> = splitted.iter().map(|env| {
                    if env.is_empty() { return Some(None); }
                    let mut tmp = env.splitn(2, "=");
                    match (tmp.next(), tmp.next()) { (Some(k), Some(v)) if !k.is_empty() => Some(Some((k, v.replace("\"", "")))), _ => None }
                }).collect::<Option<Vec<_>>>().and_then(|vec| Some(vec.into_iter().flatten().collect()));
            if let Some(env_vars) = parsed { for (k, v) in env_vars { cmd.env(k, v); } }
        }

        // Load before we spawn the game
        load_xxmi(app, install.clone(), prefix.clone(), gs.xxmi_path.clone(), runner.clone(), wine64.clone(), exe.clone(), is_proton);
        load_fps_unlock(app, install.clone(), gm.biz.clone(), prefix.clone(), gs.fps_unlock_path.clone(), dir.clone(), runner.clone(), wine64.clone(), is_proton);

        match cmd.spawn() {
            Ok(mut child) => match child.try_wait() {
                Ok(Some(status)) => {
                    if !status.success() { log::info!("Executing launch command: \"{}\" failed with status: {}", command, status.code().unwrap()); show_dialog(&app, "error", "TwintailLauncher", "Failed to execute launch command! Please try again or check game settings.", None); }
                }
                Ok(None) => {
                    let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs().to_string();
                    update_install_last_played_by_id(app, install.id.clone(), time);
                    start_playtime_tracker(app, install.clone(), gm.clone(), exe.clone());
                    log::info!("Executing launch command: \"{}\" detailed output of the command is available at {}", command, Path::new(&dir).join("game.log").to_str().unwrap());
                    write_log(app, Path::new(&dir).to_path_buf(), child, "game.log".parse().unwrap());
                }
                Err(_) => { log::error!("Executing launch command: \"{}\" failed! Is command correct?", command); show_dialog(&app, "error", "TwintailLauncher", "Failed to execute launch command! Please try again or check the command correctness.", None); }
            },
            Err(_) => { log::error!("Executing launch command \"{}\" failed catastrophically!", command); show_dialog(&app, "error", "TwintailLauncher", "Failed to execute launch command! Something serious is wrong.", None); }
        }
        true
    } else {
        // We assume user knows what he/she is doing so we just execute command that is configured without any checks
        let c = install.launch_command.clone();
        let mut args = install.launch_args.clone();
        let mut command = format!("{c}").replace("%appid%", appid.clone().to_string().as_str()).replace("%reaper%", reaper.clone().as_str()).replace("%steamrt_path%", steamrt_path.clone().as_str()).replace("%steamrt%", steamrt.clone().as_str()).replace("%prefix%", prefix.clone().as_str()).replace("%runner_dir%", runner.clone().as_str()).replace("%runner%", &*(runner.clone() + "/" + wine64.as_str())).replace("%install_dir%", dir.clone().as_str()).replace("%game_exe%", &*(dir.clone() + "/" + exe.clone().as_str()));

        let xxmi_forced = install.use_xxmi && (gm.biz == "wuwa_global" || gm.biz == "endfield_global");
        if install.use_xxmi && gm.biz == "wuwa_global" { args = args.split_whitespace().filter(|a| gm.extra.graphics_api_options.options.iter().all(|o| o.value.as_str() != *a)).collect::<Vec<_>>().join(" "); if !args.is_empty() { args += " "; } args += "-dx11"; }
        if install.use_xxmi && gm.biz == "endfield_global" { args = args.split_whitespace().filter(|a| gm.extra.graphics_api_options.options.iter().all(|o| o.value.as_str() != *a)).collect::<Vec<_>>().join(" "); if !args.is_empty() { args += " "; } args += "-force-d3d11"; }
        if gm.extra.switches.graphics_api && !xxmi_forced && !args.split_whitespace().any(|a| gm.extra.graphics_api_options.options.iter().any(|o| o.value.as_str() == a)) && !install.graphics_api.is_empty() { if !args.is_empty() { args += " "; } args += &install.graphics_api; }
        if !args.is_empty() { command = format!("{c} {args}").replace("%appid%", appid.clone().to_string().as_str()).replace("%reaper%", reaper.clone().as_str()).replace("%steamrt_path%", steamrt_path.clone().as_str()).replace("%steamrt%", steamrt.clone().as_str()).replace("%prefix%", prefix.clone().as_str()).replace("%runner_dir%", runner.clone().as_str()).replace("%runner%", &*(runner.clone() + "/" + wine64.as_str())).replace("%install_dir%", dir.clone().as_str()).replace("%game_exe%", &*(dir.clone() + "/" + exe.clone().as_str())); }

        let mut cmd = Command::new("bash");
        cmd.arg("-c");
        cmd.arg(&command);

        cmd.env("SteamOS", "1");
        cmd.env("WINEARCH", "win64");
        cmd.env("WINEPREFIX", prefix.clone() + "/pfx");
        cmd.env("STEAM_COMPAT_APP_ID", "0");
        cmd.env("STEAM_COMPAT_DATA_PATH", prefix.clone());
        cmd.env("STEAM_COMPAT_INSTALL_PATH", dir.clone());
        cmd.env("STEAM_COMPAT_CLIENT_INSTALL_PATH", "");
        cmd.env("STEAM_COMPAT_TOOL_PATHS", runner.clone());
        cmd.env("STEAM_COMPAT_LIBRARY_PATHS", format!("{dir}:{prefix}/pfx"));
        cmd.env("STEAM_COMPAT_SHADER_PATH", prefix.clone() + "/shadercache");
        cmd.env("WINEDLLOVERRIDES", "lsteamclient=d;KRSDKExternal.exe=d");
        if cpo.stub_wintrust { cmd.env("STUB_WINTRUST", "1"); }
        if cpo.block_first_req { cmd.env("BLOCK_FIRST_REQ", "1"); }
        if cpo.disable_protonfixes { cmd.env("PROTONFIXES_DISABLE", "1"); }
        if !cpo.protonfixes_store.is_empty() { cmd.env("STORE", cpo.protonfixes_store); }
        if !cpo.protonfixes_id.is_empty() { cmd.env("UMU_ID", cpo.protonfixes_id); }
        if !cpo.proton_compat_config.is_empty() { compat_config = update_steam_compat_config(cpo.proton_compat_config.iter().map(String::as_str).collect()); }
        if cpo.stub_wintrust || cpo.block_first_req { cmd.env("WINEDLLOVERRIDES", "lsteamclient=d;KRSDKExternal.exe=d;jsproxy=n,b"); crate::utils::apply_patch(app, Path::new(&dir.clone()).to_str().unwrap().to_string(), "sparkle".to_string(), "add".to_string()); } else if !cpo.stub_wintrust && !cpo.block_first_req { crate::utils::apply_patch(app, Path::new(&dir.clone()).to_str().unwrap().to_string(), "sparkle".to_string(), "remove".to_string()); }
        cmd.env("STEAM_COMPAT_CONFIG", compat_config);
        if install.use_mangohud {
            cmd.env("MANGOHUD", "1");
            if install.mangohud_config_path != "" { cmd.env("MANGOHUD_CONFIGFILE", format!("{}", install.clone().mangohud_config_path).as_str()); }
        }
        /*if gm.biz == "wuwa_global" {
            if install.use_xxmi {
                let engine_file = dirp.join("Client/Saved/Config/WindowsNoEditor/Engine.ini");
                let device_profiles_file = dirp.join("Client/Saved/Config/WindowsNoEditor/DeviceProfiles.ini");
                edit_wuwa_configs_xxmi(engine_file.to_str().unwrap().to_string(), device_profiles_file.to_str().unwrap().to_string());
            }
        }*/

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.current_dir(dir.clone());
        cmd.process_group(0);

        if !install.env_vars.is_empty() {
            let envs = install.env_vars.clone();
            let splitted = envs.split(";").collect::<Vec<&str>>();
            let parsed: Option<Vec<(&str, String)>> = splitted.iter().map(|env| {
                    if env.is_empty() { return Some(None); }
                    let mut tmp = env.splitn(2, "=");
                    match (tmp.next(), tmp.next()) { (Some(k), Some(v)) if !k.is_empty() => Some(Some((k, v.replace("\"", "")))), _ => None }
                }).collect::<Option<Vec<_>>>().and_then(|vec| Some(vec.into_iter().flatten().collect()));
            if let Some(env_vars) = parsed { for (k, v) in env_vars { cmd.env(k, v); } }
        }

        // Load before we spawn the game
        load_xxmi(app, install.clone(), prefix.clone(), gs.xxmi_path.clone(), runner.clone(), wine64.clone(), exe.clone(), is_proton);
        load_fps_unlock(app, install.clone(), gm.biz.clone(), prefix.clone(), gs.fps_unlock_path.clone(), dir.clone(), runner.clone(), wine64.clone(), is_proton);

        match cmd.spawn() {
            Ok(mut child) => match child.try_wait() {
                Ok(Some(status)) => {
                    if !status.success() { log::info!("Executing launch command: \"{}\" failed with status: {}", command, status.code().unwrap()); show_dialog(&app, "error", "TwintailLauncher", "Failed to execute launch command! Please try again or check install settings.", None); }
                }
                Ok(None) => {
                    let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs().to_string();
                    update_install_last_played_by_id(app, install.id.clone(), time);
                    start_playtime_tracker(app, install.clone(), gm.clone(), exe.clone());
                    log::info!("Executing launch command: \"{}\" detailed output of the command is available at {}", command, Path::new(&dir).join("game.log").to_str().unwrap());
                    write_log(app, Path::new(&dir).to_path_buf(), child, "game.log".parse().unwrap());
                }
                Err(_) => { log::error!("Executing launch command: \"{}\" failed! Is command correct?", command); show_dialog(&app, "error", "TwintailLauncher", "Failed to execute launch command! Please try again or check the command correctness.", None); }
            },
            Err(_) => { log::error!("Executing launch command \"{}\" failed catastrophically!", command); show_dialog(&app, "error", "TwintailLauncher", "Failed to execute launch command! Something serious is wrong.", None); }
        }
        true
    };
    if rslt { Ok(true) } else { Ok(false) }
}

#[cfg(target_os = "linux")]
fn load_xxmi(app: &AppHandle, install: LauncherInstall, prefix: String, xxmi_path: String, runner: String, wine64: String, game: String, is_proton: bool) {
    if install.use_xxmi {
        let appc = app.clone();
        // Prevent "App is not responding" by waiting in a separate thread
        std::thread::spawn(move || {
            let app = appc.clone();
            let xxmi_path = xxmi_path.clone();
            let mipath = get_mi_path_from_game(game.clone()).unwrap();
            let mi_pathbuf = Path::new(&xxmi_path).join(&mipath);
            let command = if is_proton { format!("'{runner}/{wine64}' run 'z:\\{xxmi_path}/3dmloader.exe' {mipath}") } else { format!("'{runner}/{wine64}' 'z:\\{xxmi_path}/3dmloader.exe' {mipath}") };

            // Apply the installation tweaks
            let data = apply_xxmi_tweaks(mi_pathbuf, install.xxmi_config);
            crate::utils::db_manager::update_install_xxmi_config_by_id(&app, install.id, data);

            let mut cmd = Command::new("bash");
            cmd.arg("-c");
            cmd.arg(&command);

            let loader_mode = if mipath == "efmi" { "inject" } else { "hook" };
            cmd.env("LOADER_MODE", loader_mode);
            cmd.env("WINEARCH", "win64");
            cmd.env("WINEPREFIX", prefix.clone() + "/pfx");
            cmd.env("STEAM_COMPAT_APP_ID", "0");
            cmd.env("STEAM_COMPAT_DATA_PATH", prefix.clone());
            cmd.env("STEAM_COMPAT_CLIENT_INSTALL_PATH", "");
            cmd.env("STEAM_COMPAT_TOOL_PATHS", runner.clone());
            cmd.env("PROTONFIXES_DISABLE", "1");
            cmd.env("PROTON_USE_XALIA", "0");
            cmd.env("WINEDLLOVERRIDES", "lsteamclient=d;KRSDKExternal.exe=d");

            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());
            cmd.current_dir(xxmi_path.clone());
            cmd.process_group(0);

            if !install.env_vars.is_empty() {
                let envs = install.env_vars.clone();
                let splitted = envs.split(";").collect::<Vec<&str>>();
                let parsed: Option<Vec<(&str, String)>> = splitted.iter().map(|env| {
                    if env.is_empty() { return Some(None); }
                    let mut tmp = env.splitn(2, "=");
                    match (tmp.next(), tmp.next()) { (Some(k), Some(v)) if !k.is_empty() => Some(Some((k, v.replace("\"", "")))), _ => None }
                }).collect::<Option<Vec<_>>>().and_then(|vec| Some(vec.into_iter().flatten().collect()));
                if let Some(env_vars) = parsed { for (k, v) in env_vars { cmd.env(k, v); } }
            }

            match cmd.spawn() {
                Ok(mut child) => match child.try_wait() {
                    Ok(Some(status)) => {
                        if !status.success() { log::info!("Executing XXMI command: \"{}\" failed with status: {}", command, status.code().unwrap()); show_dialog(&app, "error", "TwintailLauncher", "Failed to run XXMI! Please try again and make sure \"Inject XXMI\" is enabled!", None); }
                    }
                    Ok(None) => {
                        log::info!("Executing XXMI command: \"{}\" detailed output of the command is available at {}", command, Path::new(&xxmi_path).join("xxmi.log").to_str().unwrap());
                        write_log(&app, Path::new(&xxmi_path).to_path_buf(), child, "xxmi.log".parse().unwrap());
                    }
                    Err(_) => { log::error!("Executing XXMI command: \"{}\" failed! Is command correct?", command); show_dialog(&app, "error", "TwintailLauncher", "Failed to run XXMI! Please try again later!", None); }
                },
                Err(_) => { log::error!("Executing XXMI command \"{}\" failed catastrophically!", command); show_dialog(&app, "error", "TwintailLauncher", "Failed to run XXMI! Something serious is wrong.", None); }
            }
        });
    }
}

#[cfg(target_os = "linux")]
fn load_fps_unlock(app: &AppHandle, install: LauncherInstall, biz: String, prefix: String, fpsunlock_path: String, game_path: String, runner: String, wine64: String, is_proton: bool) {
    if install.use_fps_unlock {
        let appc = app.clone();
        // Prevent "App is not responding" by waiting in a separate thread
        std::thread::spawn(move || {
            let app = appc.clone();
            let fpsunlock_path = fpsunlock_path.clone();
            let fpsv = install.fps_value.clone();
            let command = if is_proton { format!("'{runner}/{wine64}' run 'z:\\{fpsunlock_path}/keqing_unlock.exe' run {biz} {fpsv} 2000 600 '{game_path}'") } else { format!("'{runner}/{wine64}' 'z:\\{fpsunlock_path}/keqing_unlock.exe' run {biz} {fpsv} 2000 600 '{game_path}'") };

            let mut cmd = Command::new("bash");
            cmd.arg("-c");
            cmd.arg(&command);

            cmd.env("WINEARCH", "win64");
            cmd.env("WINEPREFIX", prefix.clone() + "/pfx");
            cmd.env("STEAM_COMPAT_APP_ID", "0");
            cmd.env("STEAM_COMPAT_DATA_PATH", prefix.clone());
            cmd.env("STEAM_COMPAT_CLIENT_INSTALL_PATH", "");
            cmd.env("STEAM_COMPAT_TOOL_PATHS", runner.clone());
            cmd.env("PROTONFIXES_DISABLE", "1");
            cmd.env("PROTON_USE_XALIA", "0");
            cmd.env("WINEDLLOVERRIDES", "lsteamclient=d;KRSDKExternal.exe=d");

            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());
            cmd.current_dir(fpsunlock_path.clone());
            cmd.process_group(0);

            if !install.env_vars.is_empty() {
                let envs = install.env_vars.clone();
                let splitted = envs.split(";").collect::<Vec<&str>>();
                let parsed: Option<Vec<(&str, String)>> = splitted.iter().map(|env| {
                    if env.is_empty() { return Some(None); }
                    let mut tmp = env.splitn(2, "=");
                    match (tmp.next(), tmp.next()) { (Some(k), Some(v)) if !k.is_empty() => Some(Some((k, v.replace("\"", "")))), _ => None }
                }).collect::<Option<Vec<_>>>().and_then(|vec| Some(vec.into_iter().flatten().collect()));
                if let Some(env_vars) = parsed { for (k, v) in env_vars { cmd.env(k, v); } }
            }

            match cmd.spawn() {
                Ok(mut child) => match child.try_wait() {
                    Ok(Some(status)) => {
                        if !status.success() { log::info!("Executing FPS Unlocker command: \"{}\" failed with status: {}", command, status.code().unwrap()); show_dialog(&app, "error", "TwintailLauncher", "Failed to run FPS Unlocker! Please try again and make sure FPS Unlocker is enabled!", None); }
                    }
                    Ok(None) => {
                        log::info!("Executing FPS Unlocker command: \"{}\" detailed output of the command is available at {}", command, Path::new(&fpsunlock_path).join("fps_unlocker.log").to_str().unwrap());
                        write_log(&app, Path::new(&fpsunlock_path).to_path_buf(), child, "fps_unlocker.log".parse().unwrap());
                    }
                    Err(_) => { log::error!("Executing FPS Unlocker command: \"{}\" failed! Is command correct?", command); show_dialog(&app, "error", "TwintailLauncher", "Failed to run FPS Unlocker! Please try again later!", None); }
                },
                Err(_) => { log::error!("Executing FPS Unlocker command \"{}\" failed catastrophically!", command); show_dialog(&app, "error", "TwintailLauncher", "Failed to run FPS Unlocker! Something serious is wrong.", None); }
            }
        });
    }
}

#[cfg(target_os = "linux")]
fn run_winetricks(app: &AppHandle, install: LauncherInstall, steamrt: String, reaper: String, appid: u32, runner: String, wine64: String, prefix: String, install_dir: String, verbs: Vec<String>) -> std::thread::JoinHandle<bool> {
    let appc = app.clone();
    // Prevent "App is not responding" by waiting in a separate thread
    std::thread::spawn(move || {
        let app = appc.clone();
        let install_dir = install_dir.clone();
        let winetricks_cache = app.path().app_cache_dir().unwrap().join("winetricks");
        let winetricks_cache_str = winetricks_cache.to_str().unwrap().to_string();
        if !winetricks_cache.exists() { let _ = fs::create_dir_all(&winetricks_cache); }

        #[cfg(not(debug_assertions))]
        let winetricks_bin = if crate::utils::is_flatpak() { app.path().resource_dir().unwrap().join("resources/winetricks").to_str().unwrap().to_string().replace("/app/lib/", "/run/parent/app/lib/") } else { app.path().resource_dir().unwrap().join("resources/winetricks").to_str().unwrap().to_string().replace("/usr/lib/", "/run/host/usr/lib/") };
        #[cfg(debug_assertions)]
        let winetricks_bin = app.path().resource_dir().unwrap().join("resources/winetricks").to_str().unwrap().to_string();

        if verbs.is_empty() { return true; }
        let verbs_str = verbs.join(" ");
        let command = format!("'{steamrt}' --verb=waitforexitandrun -- '{reaper}' SteamLaunch AppId={appid} -- '{runner}/{wine64}' waitforexitandrun '{winetricks_bin}' -q -f {verbs_str}");

        let mut cmd = Command::new("bash");
        cmd.arg("-c");
        cmd.arg(&command);

        cmd.env("WINE", format!("{runner}/files/bin/wine64"));
        cmd.env("WINESERVER", format!("{runner}/files/bin/wineserver"));
        cmd.env("WINE_BIN", format!("{runner}/files/bin/wine64"));
        cmd.env("WINESERVER_BIN", format!("{runner}/files/bin/wineserver"));
        cmd.env("WINEBOOT", format!("{runner}/files/bin/wineboot"));
        cmd.env("WINEBOOT_BIN", format!("{runner}/files/bin/wineboot"));
        cmd.env("W_OPT_UNATTENDED", "1");
        cmd.env("W_CACHE", winetricks_cache_str);
        cmd.env("WINEARCH", "win64");
        cmd.env("WINEPREFIX", format!("{prefix}/pfx"));
        cmd.env("STEAM_COMPAT_DATA_PATH", prefix);
        cmd.env("STEAM_COMPAT_TOOL_PATHS", runner.clone());
        cmd.env("STEAM_COMPAT_CLIENT_INSTALL_PATH", "");
        cmd.env("PROTONFIXES_DISABLE", "1");
        cmd.env("PROTON_USE_XALIA", "0");
        cmd.env("WINEDLLOVERRIDES", "lsteamclient=d;KRSDKExternal.exe=d");
        cmd.env("WINETRICKS_SUPER_QUIET", "1");

        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());
        cmd.current_dir(&install_dir);
        cmd.process_group(0);

        if !install.env_vars.is_empty() {
            let envs = install.env_vars.clone();
            let splitted = envs.split(";").collect::<Vec<&str>>();
            let parsed: Option<Vec<(&str, String)>> = splitted.iter().map(|env| {
                if env.is_empty() { return Some(None); }
                let mut tmp = env.splitn(2, "=");
                match (tmp.next(), tmp.next()) { (Some(k), Some(v)) if !k.is_empty() => Some(Some((k, v.replace("\"", "")))), _ => None }
            }).collect::<Option<Vec<_>>>().and_then(|vec| Some(vec.into_iter().flatten().collect()));
            if let Some(env_vars) = parsed { for (k, v) in env_vars { cmd.env(k, v); } }
        }

        match cmd.spawn() {
            Ok(mut child) => {
                let status = child.wait();
                match status {
                    Ok(s) => {
                        if !s.success() { log::info!("Executing WineTricks command: \"{}\" failed with status: {}", command, s.code().unwrap()); show_dialog(&app, "warning", "TwintailLauncher", "Winetricks setup failed! The game will still attempt to launch.", Some(vec!["I understand"])); }
                        s.success()
                    }
                    Err(_) => { log::error!("Executing WineTricks command: \"{}\" failed! Is command correct?", command); show_dialog(&app, "warning", "TwintailLauncher", "Winetricks setup failed! Please try again later!", Some(vec!["I understand"])); false }
                }
            }
            Err(_) => { log::error!("Executing WineTricks command \"{}\" failed catastrophically!", command); show_dialog(&app, "warning", "TwintailLauncher", "Failed to execute winetricks for setup! Something serious is wrong.", Some(vec!["I understand"])); false }
        }
    })
}

#[cfg(target_os = "windows")]
pub fn launch(app: &AppHandle, install: LauncherInstall, gm: GameManifest, gs: GlobalSettings) -> Result<bool, Error> {
    let dirp = Path::new(&install.directory.clone()).to_path_buf();
    let dir = dirp.to_str().unwrap().to_string();
    let game = gm.paths.exe_filename.clone();
    let exe = gm.paths.exe_filename.clone().split('/').last().unwrap().to_string();

    let pre_launch = install.pre_launch_command.clone();
    if !pre_launch.is_empty() {
        let command = format!("Start-Process -FilePath '{pre_launch}' -WorkingDirectory '{dir}' -Verb RunAs");

        let mut cmd = Command::new("powershell");
        cmd.arg("-Command");
        cmd.arg(&command);

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.current_dir(dir.clone());

        match cmd.spawn() {
            Ok(mut child) => match child.try_wait() {
                Ok(Some(status)) => {
                    if !status.success() { log::info!("Executing prelaunch command: \"{}\" failed with status: {}", command, status.code().unwrap()); show_dialog(&app, "error", "TwintailLauncher", "Failed to execute prelaunch command! Please try again or check game settings.", None); }
                }
                Ok(None) => {
                    log::info!("Executing prelaunch command: \"{}\" detailed output of the command is available at {}", command, Path::new(&dir).join("pre_launch.log").to_str().unwrap());
                    write_log(app, Path::new(&dir).to_path_buf(), child, "pre_launch.log".parse().unwrap());
                }
                Err(_) => { log::error!("Executing prelaunch command: \"{}\" failed! Is command correct?", command); show_dialog(&app, "error", "TwintailLauncher", "Failed to execute prelaunch command! Please try again or check the command correctness.", None); }
            },
            Err(_) => { log::error!("Executing prelaunch command \"{}\" failed catastrophically!", command); show_dialog(&app, "error", "TwintailLauncher", "Failed to execute prelaunch command! Something serious is wrong.", None); }
        }
    }

    /*if gm.biz == "wuwa_global" {
        if install.use_xxmi {
            let engine_file = dirp.join("Client/Saved/Config/WindowsNoEditor/Engine.ini");
            let device_profiles_file = dirp.join("Client/Saved/Config/WindowsNoEditor/DeviceProfiles.ini");
            edit_wuwa_configs_xxmi(engine_file.to_str().unwrap().to_string(), device_profiles_file.to_str().unwrap().to_string());
        }
    }*/
    // Run xxmi first
    load_xxmi(app, install.clone(), gs.xxmi_path, exe.clone());
    load_fps_unlock(app, install.clone(), gm.biz.clone(), dir.clone(), gs.fps_unlock_path);

    let rslt = if install.launch_command.is_empty() {
        let mut args= install.launch_args.clone();
        let dir = dir.trim_matches('\\');
        let game = game.trim_matches('\\');
        let tmp = game.replace("/", "\\");

        let full_path = Path::new(dir).join(&tmp);
        let full_path_str = full_path.to_str().unwrap().replace("/", "\\");
        let mut command = format!("Start-Process -FilePath '{full_path_str}' -WorkingDirectory '{dir}' -Verb RunAs");

        let xxmi_forced = install.use_xxmi && (gm.biz == "wuwa_global" || gm.biz == "endfield_global");
        if install.use_xxmi && gm.biz == "wuwa_global" { args = args.split_whitespace().filter(|a| gm.extra.graphics_api_options.options.iter().all(|o| o.value.as_str() != *a)).collect::<Vec<_>>().join(" "); if !args.is_empty() { args += " "; } args += "-dx11"; }
        if install.use_xxmi && gm.biz == "endfield_global" { args = args.split_whitespace().filter(|a| gm.extra.graphics_api_options.options.iter().all(|o| o.value.as_str() != *a)).collect::<Vec<_>>().join(" "); if !args.is_empty() { args += " "; } args += "-force-d3d11"; }
        if gm.extra.switches.graphics_api && !xxmi_forced && !args.split_whitespace().any(|a| gm.extra.graphics_api_options.options.iter().any(|o| o.value.as_str() == a)) && !install.graphics_api.is_empty() { if !args.is_empty() { args += " "; } args += &install.graphics_api; }
        if !args.is_empty() { command = format!("Start-Process -FilePath '{full_path_str}' -ArgumentList '{args}' -WorkingDirectory '{dir}' -Verb RunAs"); }

        let mut cmd = Command::new("powershell");
        cmd.arg("-Command");
        cmd.arg(&command);

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.current_dir(dir);

        if !install.env_vars.is_empty() {
            let envs = install.env_vars.clone();
            let splitted = envs.split(";").collect::<Vec<&str>>();
            let parsed: Option<Vec<(&str, String)>> = splitted.iter().map(|env| {
                    if env.is_empty() { return Some(None); }
                    let mut tmp = env.splitn(2, "=");
                    match (tmp.next(), tmp.next()) { (Some(k), Some(v)) if !k.is_empty() => Some(Some((k, v.replace("\"", "")))), _ => None }
                }).collect::<Option<Vec<_>>>().and_then(|vec| Some(vec.into_iter().flatten().collect()));

            if let Some(env_vars) = parsed { for (k, v) in env_vars { cmd.env(k, v); } }
        }

        match cmd.spawn() {
            Ok(mut child) => match child.try_wait() {
                Ok(Some(status)) => {
                    if !status.success() { log::info!("Executing launch command: \"{}\" failed with status: {}", command, status.code().unwrap()); show_dialog(&app, "error", "TwintailLauncher", "Failed to run launch command! Please try again or check game settings.", None); }
                }
                Ok(None) => {
                    let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs().to_string();
                    update_install_last_played_by_id(app, install.id.clone(), time);
                    start_playtime_tracker(app, install.clone(), gm.clone(), exe.clone());
                    log::info!("Executing launch command: \"{}\" detailed output of the command is available at {}", command, Path::new(&dir).join("game.log").to_str().unwrap());
                    write_log(app, Path::new(&dir).to_path_buf(), child, "game.log".parse().unwrap());
                }
                Err(_) => { log::error!("Executing launch command: \"{}\" failed! Is command correct?", command); show_dialog(&app, "error", "TwintailLauncher", "Failed to run launch command! Please try again or check the command correctness.", None); }
            },
            Err(_) => { log::error!("Executing launch command \"{}\" failed catastrophically!", command); show_dialog(&app, "error", "TwintailLauncher", "Failed to execute launch command! Something serious is wrong.", None); }
        }
        true
    } else {
        // We assume user knows what he/she is doing so we just execute command that is configured without any checks
        let dir = dir.trim_matches('\\');
        let game = game.trim_matches('\\');
        let tmp = game.replace("/", "\\");

        let full_path = Path::new(dir).join(&tmp);
        let full_path_str = full_path.to_str().unwrap().replace("/", "\\");
        let c = install.launch_command.clone();
        let mut args= install.launch_args.clone();
        let mut command = format!("Start-Process -FilePath '{c}' -WorkingDirectory '{dir}' -Verb RunAs").replace("%install_dir%", dir).replace("%game_exe%", full_path_str.as_str());

        let xxmi_forced = install.use_xxmi && (gm.biz == "wuwa_global" || gm.biz == "endfield_global");
        if install.use_xxmi && gm.biz == "wuwa_global" { args = args.split_whitespace().filter(|a| gm.extra.graphics_api_options.options.iter().all(|o| o.value.as_str() != *a)).collect::<Vec<_>>().join(" "); if !args.is_empty() { args += " "; } args += "-dx11"; }
        if install.use_xxmi && gm.biz == "endfield_global" { args = args.split_whitespace().filter(|a| gm.extra.graphics_api_options.options.iter().all(|o| o.value.as_str() != *a)).collect::<Vec<_>>().join(" "); if !args.is_empty() { args += " "; } args += "-force-d3d11"; }
        if gm.extra.switches.graphics_api && !xxmi_forced && !args.split_whitespace().any(|a| gm.extra.graphics_api_options.options.iter().any(|o| o.value.as_str() == a)) && !install.graphics_api.is_empty() { if !args.is_empty() { args += " "; } args += &install.graphics_api; }
        if !args.is_empty() { command = format!("Start-Process -FilePath '{c}' -ArgumentList '{args}' -WorkingDirectory '{dir}' -Verb RunAs").replace("%install_dir%", dir).replace("%game_exe%", full_path_str.as_str()); }

        let mut cmd = Command::new("powershell");
        cmd.arg("-Command");
        cmd.arg(&command);

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.current_dir(dir);

        if !install.env_vars.is_empty() {
            let envs = install.env_vars.clone();
            let splitted = envs.split(";").collect::<Vec<&str>>();
            let parsed: Option<Vec<(&str, String)>> = splitted.iter().map(|env| {
                    if env.is_empty() { return Some(None); }
                    let mut tmp = env.splitn(2, "=");
                    match (tmp.next(), tmp.next()) { (Some(k), Some(v)) if !k.is_empty() => Some(Some((k, v.replace("\"", "")))), _ => None }
                }).collect::<Option<Vec<_>>>().and_then(|vec| Some(vec.into_iter().flatten().collect()));

            if let Some(env_vars) = parsed { for (k, v) in env_vars { cmd.env(k, v); } }
        }

        match cmd.spawn() {
            Ok(mut child) => match child.try_wait() {
                Ok(Some(status)) => {
                    if !status.success() { log::info!("Executing launch command: \"{}\" failed with status: {}", command, status.code().unwrap()); show_dialog(&app, "error", "TwintailLauncher", "Failed to execute launch command! Please try again or check game settings.", None); }
                }
                Ok(None) => {
                    let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs().to_string();
                    update_install_last_played_by_id(app, install.id.clone(), time);
                    start_playtime_tracker(app, install.clone(), gm.clone(), exe.clone());
                    log::info!("Executing launch command: \"{}\" detailed output of the command is available at {}", command, Path::new(&dir).join("game.log").to_str().unwrap());
                    write_log(app, Path::new(&dir).to_path_buf(), child, "game.log".parse().unwrap());
                }
                Err(_) => { log::error!("Executing launch command: \"{}\" failed! Is command correct?", command); show_dialog(&app, "error", "TwintailLauncher", "Failed to execute launch command! Please try again or check the command correctness.", None); }
            },
            Err(_) => { log::error!("Executing launch command \"{}\" failed catastrophically!", command); show_dialog(&app, "error", "TwintailLauncher", "Failed to execute launch command! Something serious is wrong.", None); }
        }
        true
    };
    Ok(rslt)
}

#[cfg(target_os = "windows")]
fn load_xxmi(app: &AppHandle, install: LauncherInstall, xxmi_path: String, game: String) {
    if install.use_xxmi {
        let xxmi_path = xxmi_path.trim_matches('\\');
        let mipath = get_mi_path_from_game(game.clone()).unwrap();
        let mi_pathbuf = Path::new(&xxmi_path).join(&mipath);
        let loader_path = Path::new(xxmi_path).join("3dmloader.exe");
        let loader_path_str = loader_path.to_str().unwrap().replace("/", "\\");
        let command = format!("Start-Process -FilePath '{}' -ArgumentList '{}' -WorkingDirectory '{}' -Verb RunAs", loader_path_str, mipath, xxmi_path);

        // Apply the installation tweaks
        let data = apply_xxmi_tweaks(mi_pathbuf, install.xxmi_config);
        crate::utils::db_manager::update_install_xxmi_config_by_id(&app, install.id, data);

        let mut cmd = Command::new("powershell");
        cmd.arg("-Command");
        cmd.arg(&command);

        let loader_mode = if mipath == "efmi" { "inject" } else { "hook" };
        cmd.env("LOADER_MODE", loader_mode);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.current_dir(xxmi_path);

        if !install.env_vars.is_empty() {
            let envs = install.env_vars.clone();
            let splitted = envs.split(";").collect::<Vec<&str>>();
            let parsed: Option<Vec<(&str, String)>> = splitted.iter().map(|env| {
                if env.is_empty() { return Some(None); }
                let mut tmp = env.splitn(2, "=");
                match (tmp.next(), tmp.next()) { (Some(k), Some(v)) if !k.is_empty() => Some(Some((k, v.replace("\"", "")))), _ => None }
            }).collect::<Option<Vec<_>>>().and_then(|vec| Some(vec.into_iter().flatten().collect()));

            if let Some(env_vars) = parsed { for (k, v) in env_vars { cmd.env(k, v); } }
        }

        let spawned = cmd.spawn();
        if spawned.is_ok() {
            log::info!("Executing XXMI command: \"{}\" detailed output of the command is available at {}", command, Path::new(&xxmi_path).join("xxmi.log").to_str().unwrap());
            let process = spawned.unwrap();
            write_log(app, Path::new(&xxmi_path).to_path_buf(), process, "xxmi.log".parse().unwrap());
        }
    }
}

#[cfg(target_os = "windows")]
fn load_fps_unlock(app: &AppHandle, install: LauncherInstall, biz: String, game_path: String, fpsunlock_path: String) {
    if install.use_fps_unlock {
        let fpsunlock_path = fpsunlock_path.trim_matches('\\');
        let loader_path = Path::new(fpsunlock_path).join("keqing_unlock.exe");
        let loader_path_str = loader_path.to_str().unwrap().replace("/", "\\");
        let fpsv = install.fps_value.clone();
        let args = format!("run {} {} 2000 0 \"{}\"", biz, fpsv, game_path);
        let command = format!("Start-Process -FilePath '{}' -ArgumentList '{}' -WorkingDirectory '{}' -Verb RunAs", loader_path_str, args, fpsunlock_path);

        let mut cmd = Command::new("powershell");
        cmd.arg("-Command");
        cmd.arg(&command);

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.current_dir(fpsunlock_path);

        if !install.env_vars.is_empty() {
            let envs = install.env_vars.clone();
            let splitted = envs.split(";").collect::<Vec<&str>>();
            let parsed: Option<Vec<(&str, String)>> = splitted.iter().map(|env| {
                if env.is_empty() { return Some(None); }
                let mut tmp = env.splitn(2, "=");
                match (tmp.next(), tmp.next()) { (Some(k), Some(v)) if !k.is_empty() => Some(Some((k, v.replace("\"", "")))), _ => None }
            }).collect::<Option<Vec<_>>>().and_then(|vec| Some(vec.into_iter().flatten().collect()));

            if let Some(env_vars) = parsed { for (k, v) in env_vars { cmd.env(k, v); } }
        }

        let spawned = cmd.spawn();
        if spawned.is_ok() {
            log::info!("Executing FPS Unlocker command: \"{}\" detailed output of the command is available at {}", command, Path::new(&fpsunlock_path).join("fps_unlocker.log").to_str().unwrap());
            let process = spawned.unwrap();
            write_log(app, Path::new(&fpsunlock_path).to_path_buf(), process, "fps_unlocker.log".parse().unwrap());
        }
    }
}

fn start_playtime_tracker(app: &AppHandle, install: LauncherInstall, gm: GameManifest, exe_name: String) {
    let app = app.clone();
    let install_id = install.id.clone();
    let base_playtime = install.total_playtime as u64;
    #[cfg(target_os = "linux")]
    let exe_name = { let stem = exe_name.split('.').next().unwrap_or(&exe_name); stem[..stem.len().min(15)].to_string() };
    std::thread::spawn(move || {
        let poll_interval = std::time::Duration::from_secs(5);
        let db_write_interval = 30u64;
        let mut last_db_write_elapsed: u64 = 0;
        std::thread::sleep(std::time::Duration::from_secs(5));
        if !is_process_running(&exe_name) { return; }
        let mut rpc_client = None;
        if install.show_discord_rpc { rpc_client = discord_rpc::init(&app, install.clone(), gm.clone()); }
        let mut keepawake = None;
        if install.disable_system_idle { keepawake = prevent_system_idle(true); }
        let started = std::time::Instant::now();
        loop {
            std::thread::sleep(poll_interval);
            let elapsed = started.elapsed().as_secs();
            let running = is_process_running(&exe_name);
            if !running || elapsed - last_db_write_elapsed >= db_write_interval {
                let new_total = base_playtime + elapsed;
                update_install_total_playtime_by_id(&app, install_id.clone(), new_total.to_string());
                if !running {
                    if install.show_discord_rpc { if let Some(ref mut client) = rpc_client { discord_rpc::terminate(client); } }
                    if install.disable_system_idle { drop(keepawake); }
                    app.emit("game_closed", install_id.clone()).unwrap();
                    return;
                }
                last_db_write_elapsed = elapsed;
            }
        }
    });
}

fn write_log(app: &AppHandle, log_dir: PathBuf, child: Child, file: String) {
    let ld1 = Arc::new(Mutex::new(log_dir.clone()));
    let c1 = Arc::new(Mutex::new(child));
    let ac = Arc::new(app.clone());
    std::thread::spawn(move || {
        let log_dir = ld1.lock().unwrap().clone();
        let mut child = c1.lock().unwrap();
        let log_file_size = 8 * 1024 * 1024; // 8 MiB

        let game_output = Arc::new(Mutex::new(fs::File::create(log_dir.join(file)).unwrap()));
        let written = Arc::new(AtomicUsize::new(0));
        let mut stdout_join = None;
        let mut stderr_join = None;

        if let Some(mut stdout) = child.stdout.take() {
            let game_output = game_output.clone();
            let written = written.clone();

            stdout_join = Some(std::thread::spawn(move || -> std::io::Result<()> {
                let mut buf = [0; 1024];
                while let Ok(read) = stdout.read(&mut buf) {
                    if read == 0 { break; }
                    let Ok(mut game_output) = game_output.lock() else { break; };

                    for line in buf[..read].split(|c| c == &b'\n') {
                        game_output.write_all(b"    [stdout] ")?;
                        game_output.write_all(line)?;
                        game_output.write_all(b"\n")?;
                        written.fetch_add(line.len() + 14, Ordering::Relaxed);
                    }
                    if written.load(Ordering::Relaxed) > log_file_size { break; }
                }
                Ok(())
            }));
        }

        if let Some(mut stderr) = child.stderr.take() {
            let game_output = game_output.clone();
            let written = written.clone();

            stderr_join = Some(std::thread::spawn(move || -> std::io::Result<()> {
                let mut buf = [0; 1024];
                while let Ok(read) = stderr.read(&mut buf) {
                    if read == 0 { break; }
                    let Ok(mut game_output) = game_output.lock() else { break; };

                    for line in buf[..read].split(|c| c == &b'\n') {
                        game_output.write_all(b"[!] [stderr] ")?;
                        game_output.write_all(line)?;
                        game_output.write_all(b"\n")?;
                        written.fetch_add(line.len() + 14, Ordering::Relaxed);
                    }
                    if written.load(Ordering::Relaxed) > log_file_size { break; }
                }
                Ok(())
            }));
        }

        // Send notify if we fail to execute any command
        let status = child.wait().unwrap();
        let mut stderr_output = String::new();
        if let Some(mut stderr) = child.stderr.take() { let _ = stderr.read_to_string(&mut stderr_output); }

        if !status.success() || !stderr_output.trim().is_empty() {
            let message = if !stderr_output.trim().is_empty() { format!("Failed to run command: {}", stderr_output.trim()) } else { "Failed to run command! Please try again or check logs available in game directory or respective tool's directory.".to_string() };
            show_dialog(&ac, "error", "TwintailLauncher", &message, None);
        }

        if let Ok(mut file) = game_output.lock() { file.flush().unwrap(); }
        drop(game_output);
        if let Some(join) = stdout_join { join.join().map_err(|err| format!("Failed to join stdout reader thread: {err:?}")).unwrap().unwrap(); }
        if let Some(join) = stderr_join { join.join().map_err(|err| format!("Failed to join stderr reader thread: {err:?}")).unwrap().unwrap(); }
    });
}
