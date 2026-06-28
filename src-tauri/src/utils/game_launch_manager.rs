use crate::utils::models::{GameManifest, GlobalSettings, LauncherInstall};
use crate::utils::{apply_xxmi_tweaks,get_mi_path_from_game,prevent_system_idle,show_dialog_with_callback};
use std::process::{Command, Stdio};
use tauri::{AppHandle, Runtime, Emitter, Error};
use crate::utils::db_manager::{update_install_last_played_by_id,update_install_total_playtime_by_id};
use crate::utils::discord_rpc;
use fischl::utils::is_process_running;

#[cfg(target_os = "linux")]
use crate::utils::{get_steam_appid, get_steam_tool_appid, is_runner_lower, is_using_overriden_runner, runner_from_runner_version, update_steam_compat_config, repo_manager::get_compatibility};
#[cfg(target_os = "linux")]
use std::os::unix::process::CommandExt;
#[cfg(target_os = "linux")]
use tauri::Manager;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "linux")]
pub fn launch<R: Runtime>(app: &AppHandle<R>, install: LauncherInstall, gm: GameManifest, gs: GlobalSettings) -> Result<bool, Error> {
    let Some(rm) = get_compatibility(&app, &runner_from_runner_version(app, install.runner_version.clone()).unwrap_or_default()) else { return Ok(false); };
    let is_proton = rm.display_name.to_ascii_lowercase().contains("proton") && !rm.display_name.to_ascii_lowercase().contains("wine");
    let mut compat_config = update_steam_compat_config(vec![]);
    let cpo = gm.extra.compat_overrides.clone();

    let dirp = std::path::Path::new(install.directory.as_str());
    let dir = dirp.to_str().unwrap().to_string();
    let prefixp = std::path::Path::new(install.runner_prefix.as_str()).to_path_buf();
    let prefix = prefixp.to_str().unwrap().to_string();
    let runnerp = std::path::Path::new(gs.default_runner_path.as_str()).to_path_buf();
    let runnerpi = std::path::Path::new(install.runner_path.as_str()).to_path_buf();
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

    if !steamrtp.exists() {
        log::info!("Attempted to launch {} with broken SteamRT (ToolID: {})! Pressing Repair SteamLinuxRuntime button in application settings is recommended.", install.name, toolid);
        show_dialog_with_callback(app, "error", "TwintailLauncher", "dialogs.launch_steamrt_broken", Some(vec!["dialogs.buttons.i_understand"]), None, Some(std::collections::HashMap::from([("install_name", install.name.as_str())])));
        return Ok(false);
    }

    if is_runner_lower(cpo.min_runner_versions.clone(), install.clone().runner_version) && !cpo.min_runner_versions.is_empty() {
        log::info!("Attempted to launch {} with runner version {} which is lower than the minimum required runner version(s) of {}!", install.name, install.runner_version, cpo.min_runner_versions.join(", "));
        let min_vers = cpo.min_runner_versions.join(", "); show_dialog_with_callback(app, "warning", "TwintailLauncher", "dialogs.launch_runner_unsupported", Some(vec!["dialogs.buttons.i_understand"]), None, Some(std::collections::HashMap::from([("install_name", install.name.as_str()), ("runner_version", install.runner_version.as_str()), ("min_runner_versions", min_vers.as_str())])));
        return Ok(false);
    }

    if cpo.override_runner.linux.enabled && !cpo.override_runner.linux.runner_version.is_empty() && !is_using_overriden_runner(install.runner_version.clone(), cpo.override_runner.linux.runner_version.clone()) {
        log::info!("Attempted to launch {} with runner version {} while compatibility override is set to {}!", install.name, install.runner_version, cpo.override_runner.linux.runner_version);
        show_dialog_with_callback(app, "warning", "TwintailLauncher", "dialogs.launch_runner_version_required", Some(vec!["dialogs.buttons.i_understand"]), None, Some(std::collections::HashMap::from([("install_name", install.name.as_str()), ("runner_version", install.runner_version.as_str()), ("required_runner", cpo.override_runner.linux.runner_version.as_str())])));
        return Ok(false);
    }

    // If prefix folder somehow does not exist remake it
    if !prefixp.exists() {
        if let Err(e) = std::fs::create_dir_all(&prefixp) {
            log::error!("Failed to create missing runner prefix folder at {}! Error: {}", prefixp.to_str().unwrap(), e.to_string());
            let err_str = e.to_string(); show_dialog_with_callback(app, "warning", "TwintailLauncher", "dialogs.prefix_reinit_failed", Some(vec!["dialogs.buttons.i_understand"]), None, Some(std::collections::HashMap::from([("error", err_str.as_str())])));
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

        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());
        cmd.current_dir(dir.clone());
        cmd.process_group(0);

        match cmd.spawn() {
            Ok(mut child) => match child.try_wait() {
                Ok(Some(status)) => {
                    if !status.success() { log::info!("Executing prelaunch command: \"{}\" failed with status: {}", command, status.code().unwrap()); show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.prelaunch_cmd_failed", None, None, None); }
                }
                Ok(None) => { log::info!("Executing prelaunch command: \"{}\"", command); }
                Err(_) => { show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.prelaunch_cmd_incorrect", None, None, None); }
            },
            Err(_) => { log::error!("Executing prelaunch command \"{}\" failed catastrophically!", command); show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.prelaunch_cmd_critical", None, None, None); }
        }
    }

    let verb = if install.use_xxmi || install.use_fps_unlock { "run" } else { "waitforexitandrun" };
    let drive = if cpo.proton_compat_config.contains(&"gamedrive".to_string()) { format!("s:\\{game}") } else { format!("z:\\{dir}/{game}") };

    let mut args = install.launch_args.clone();
    let xxmi_forced = install.use_xxmi && (gm.biz == "wuwa_global" || gm.biz == "endfield_global" || gm.biz == "nap_global");
    if install.use_xxmi && gm.biz == "wuwa_global" { args = args.split_whitespace().filter(|a| gm.extra.graphics_api_options.options.iter().all(|o| o.value.as_str() != *a)).collect::<Vec<_>>().join(" "); if !args.is_empty() { args += " "; } args += "-dx11"; }
    if install.use_xxmi && gm.biz == "endfield_global" { args = args.split_whitespace().filter(|a| gm.extra.graphics_api_options.options.iter().all(|o| o.value.as_str() != *a)).collect::<Vec<_>>().join(" "); if !args.is_empty() { args += " "; } args += "-force-d3d11"; }
    if install.use_xxmi && gm.biz == "nap_global" { args = args.split_whitespace().filter(|a| gm.extra.graphics_api_options.options.iter().all(|o| o.value.as_str() != *a)).collect::<Vec<_>>().join(" "); if !args.is_empty() { args += " "; } args += "-use-d3d11"; }
    if gm.extra.switches.graphics_api && !xxmi_forced && !args.split_whitespace().any(|a| gm.extra.graphics_api_options.options.iter().any(|o| o.value.as_str() == a)) && !install.graphics_api.is_empty() { if !args.is_empty() { args += " "; } args += &install.graphics_api; }

    let gamemode_ok = if install.use_gamemode && !crate::utils::is_flatpak() {
        let found = std::env::var("PATH").unwrap_or_default().split(':').any(|dir| std::path::Path::new(dir).join("gamemoderun").exists());
        if !found { show_dialog_with_callback(app, "warning", "TwintailLauncher", "dialogs.gamemode_not_found", Some(vec!["dialogs.buttons.i_understand"]), None, None); }
        found
    } else { install.use_gamemode };

    let default_command = if is_proton {
        let steamrt_run = format!("'{steamrt}' --verb={verb} -- '{reaper}' SteamLaunch AppId={appid} -- '{runner}/{wine64}' {verb} '{drive}' {args}");
        if gamemode_ok { format!("gamemoderun {steamrt_run}") } else { format!("{steamrt_run}") }
    } else {
        if gamemode_ok { format!("gamemoderun '{runner}/{wine64}' '{dir}/{game}' {args}") } else { format!("'{runner}/{wine64}' '{dir}/{game}' {args}") }
    };

    let rslt = if install.launch_command.is_empty() {
        let mut cmd = Command::new("bash");
        cmd.arg("-c");
        cmd.arg(&default_command);

        cmd.env("SteamGameId", if gm.biz == "wuwa_global" { "3513350".to_string() } else { appid.clone().to_string() });
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
        if cpo.stub_wintrust || cpo.block_first_req { cmd.env("WINEDLLOVERRIDES", "lsteamclient=d;KRSDKExternal.exe=d;jsproxy=n,b"); crate::utils::apply_patch(app, std::path::Path::new(&dir.clone()).to_str().unwrap().to_string(), "sparkle".to_string(), "add".to_string()); } else if !cpo.stub_wintrust && !cpo.block_first_req { crate::utils::apply_patch(app, std::path::Path::new(&dir.clone()).to_str().unwrap().to_string(), "sparkle".to_string(), "remove".to_string()); }
        cmd.env("STEAM_COMPAT_CONFIG", compat_config);
        if install.use_mangohud {
            cmd.env("MANGOHUD", "1");
            if install.mangohud_config_path != "" { cmd.env("MANGOHUD_CONFIGFILE", format!("{}", install.clone().mangohud_config_path).as_str()); }
        }

        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());
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
                    if !status.success() { log::info!("Executing launch command: \"{}\" failed with status: {}", default_command, status.code().unwrap()); show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.launch_cmd_failed", None, None, None); }
                }
                Ok(None) => {
                    let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs().to_string();
                    update_install_last_played_by_id(app, install.id.clone(), time);
                    start_playtime_tracker(app, install.clone(), gm.clone(), exe.clone());
                    log::info!("Executing launch command: \"{}\"", default_command);
                }
                Err(_) => { log::error!("Executing launch command: \"{}\" failed! Is command correct?", default_command); show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.launch_cmd_incorrect", None, None, None); }
            },
            Err(_) => { log::error!("Executing launch command \"{}\" failed catastrophically!", default_command); show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.launch_cmd_critical", None, None, None); }
        }
        true
    } else {
        // We assume user knows what he/she is doing so we just execute command that is configured without any checks
        let c = install.launch_command.clone();
        let mut args = install.launch_args.clone();
        let mut command = format!("{c}").replace("%command%", default_command.clone().to_string().as_str()).replace("%appid%", appid.clone().to_string().as_str()).replace("%reaper%", reaper.clone().as_str()).replace("%steamrt_path%", steamrt_path.clone().as_str()).replace("%steamrt%", steamrt.clone().as_str()).replace("%prefix%", prefix.clone().as_str()).replace("%runner_dir%", runner.clone().as_str()).replace("%runner%", &*(runner.clone() + "/" + wine64.as_str())).replace("%install_dir%", dir.clone().as_str()).replace("%game_exe%", &*(dir.clone() + "/" + exe.clone().as_str()));

        let xxmi_forced = install.use_xxmi && (gm.biz == "wuwa_global" || gm.biz == "endfield_global" || gm.biz == "nap_global");
        if install.use_xxmi && gm.biz == "wuwa_global" { args = args.split_whitespace().filter(|a| gm.extra.graphics_api_options.options.iter().all(|o| o.value.as_str() != *a)).collect::<Vec<_>>().join(" "); if !args.is_empty() { args += " "; } args += "-dx11"; }
        if install.use_xxmi && gm.biz == "endfield_global" { args = args.split_whitespace().filter(|a| gm.extra.graphics_api_options.options.iter().all(|o| o.value.as_str() != *a)).collect::<Vec<_>>().join(" "); if !args.is_empty() { args += " "; } args += "-force-d3d11"; }
        if install.use_xxmi && gm.biz == "nap_global" { args = args.split_whitespace().filter(|a| gm.extra.graphics_api_options.options.iter().all(|o| o.value.as_str() != *a)).collect::<Vec<_>>().join(" "); if !args.is_empty() { args += " "; } args += "-use-d3d11"; }
        if gm.extra.switches.graphics_api && !xxmi_forced && !args.split_whitespace().any(|a| gm.extra.graphics_api_options.options.iter().any(|o| o.value.as_str() == a)) && !install.graphics_api.is_empty() { if !args.is_empty() { args += " "; } args += &install.graphics_api; }
        if !args.is_empty() { command = format!("{c} {args}").replace("%command%", default_command.clone().to_string().as_str()).replace("%appid%", appid.clone().to_string().as_str()).replace("%reaper%", reaper.clone().as_str()).replace("%steamrt_path%", steamrt_path.clone().as_str()).replace("%steamrt%", steamrt.clone().as_str()).replace("%prefix%", prefix.clone().as_str()).replace("%runner_dir%", runner.clone().as_str()).replace("%runner%", &*(runner.clone() + "/" + wine64.as_str())).replace("%install_dir%", dir.clone().as_str()).replace("%game_exe%", &*(dir.clone() + "/" + exe.clone().as_str())); }

        let mut cmd = Command::new("bash");
        cmd.arg("-c");
        cmd.arg(&command);

        cmd.env("SteamGameId", if gm.biz == "wuwa_global" { "3513350".to_string() } else { appid.clone().to_string() });
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
        if cpo.stub_wintrust || cpo.block_first_req { cmd.env("WINEDLLOVERRIDES", "lsteamclient=d;KRSDKExternal.exe=d;jsproxy=n,b"); crate::utils::apply_patch(app, std::path::Path::new(&dir.clone()).to_str().unwrap().to_string(), "sparkle".to_string(), "add".to_string()); } else if !cpo.stub_wintrust && !cpo.block_first_req { crate::utils::apply_patch(app, std::path::Path::new(&dir.clone()).to_str().unwrap().to_string(), "sparkle".to_string(), "remove".to_string()); }
        cmd.env("STEAM_COMPAT_CONFIG", compat_config);
        if install.use_mangohud {
            cmd.env("MANGOHUD", "1");
            if install.mangohud_config_path != "" { cmd.env("MANGOHUD_CONFIGFILE", format!("{}", install.clone().mangohud_config_path).as_str()); }
        }

        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());
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
                    if !status.success() { log::info!("Executing launch command: \"{}\" failed with status: {}", command, status.code().unwrap()); show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.launch_cmd_failed", None, None, None); }
                }
                Ok(None) => {
                    let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs().to_string();
                    update_install_last_played_by_id(app, install.id.clone(), time);
                    start_playtime_tracker(app, install.clone(), gm.clone(), exe.clone());
                    log::info!("Executing launch command: \"{}\"", command);
                }
                Err(_) => { log::error!("Executing launch command: \"{}\" failed! Is command correct?", command); show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.launch_cmd_incorrect", None, None, None); }
            },
            Err(_) => { log::error!("Executing launch command \"{}\" failed catastrophically!", command); show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.launch_cmd_critical", None, None, None); }
        }
        true
    };
    if rslt { Ok(true) } else { Ok(false) }
}

#[cfg(target_os = "linux")]
fn load_xxmi<R: Runtime>(app: &AppHandle<R>, install: LauncherInstall, prefix: String, xxmi_path: String, runner: String, wine64: String, game: String, is_proton: bool) {
    if install.use_xxmi {
        let appc = app.clone();
        // Prevent "App is not responding" by waiting in a separate thread
        std::thread::spawn(move || {
            let app = appc.clone();
            let xxmi_path = xxmi_path.clone();
            let mipath = get_mi_path_from_game(game.clone()).unwrap();
            let mi_pathbuf = std::path::Path::new(&xxmi_path).join(&mipath);
            let game_dir = std::path::PathBuf::from(&install.directory);
            let command = if is_proton { format!("'{runner}/{wine64}' run 'z:\\{xxmi_path}/3dmloader.exe' {mipath}") } else { format!("'{runner}/{wine64}' 'z:\\{xxmi_path}/3dmloader.exe' {mipath}") };

            // Apply the installation tweaks
            let data = apply_xxmi_tweaks(mi_pathbuf, install.xxmi_config);
            crate::utils::db_manager::update_install_xxmi_config_by_id(&app, install.id, data);
            if mipath.to_ascii_lowercase().as_str() == "wwmi" { crate::utils::apply_wwmi_tweaks(game_dir.to_path_buf(), xxmi_path.clone()); }

            let mut cmd = Command::new("bash");
            cmd.arg("-c");
            cmd.arg(&command);

            let loader_mode = if mipath == "efmi" || mipath == "wwmi" { "inject" } else { "hook" };
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

            cmd.stdout(Stdio::null());
            cmd.stderr(Stdio::null());
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
                        if !status.success() { log::info!("Executing XXMI command: \"{}\" failed with status: {}", command, status.code().unwrap()); show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.xxmi_run_failed", None, None, None); }
                    }
                    Ok(None) => { log::info!("Executing XXMI command: \"{}\"", command); }
                    Err(_) => { log::error!("Executing XXMI command: \"{}\" failed! Is command correct?", command); show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.xxmi_run_retry", None, None, None); }
                },
                Err(_) => { log::error!("Executing XXMI command \"{}\" failed catastrophically!", command); show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.xxmi_run_critical", None, None, None); }
            }
        });
    }
}

#[cfg(target_os = "linux")]
fn load_fps_unlock<R: Runtime>(app: &AppHandle<R>, install: LauncherInstall, biz: String, prefix: String, fpsunlock_path: String, game_path: String, runner: String, wine64: String, is_proton: bool) {
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

            cmd.stdout(Stdio::null());
            cmd.stderr(Stdio::null());
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
                        if !status.success() { log::info!("Executing FPS Unlocker command: \"{}\" failed with status: {}", command, status.code().unwrap()); show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.fps_unlock_run_failed", None, None, None); }
                    }
                    Ok(None) => { log::info!("Executing FPS Unlocker command: \"{}\"", command); }
                    Err(_) => { log::error!("Executing FPS Unlocker command: \"{}\" failed! Is command correct?", command); show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.fps_unlock_run_retry", None, None, None); }
                },
                Err(_) => { log::error!("Executing FPS Unlocker command \"{}\" failed catastrophically!", command); show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.fps_unlock_run_critical", None, None, None); }
            }
        });
    }
}

#[cfg(target_os = "linux")]
fn run_winetricks<R: Runtime>(app: &AppHandle<R>, install: LauncherInstall, steamrt: String, reaper: String, appid: u32, runner: String, wine64: String, prefix: String, install_dir: String, verbs: Vec<String>) -> std::thread::JoinHandle<bool> {
    let appc = app.clone();
    // Prevent "App is not responding" by waiting in a separate thread
    std::thread::spawn(move || {
        let app = appc.clone();
        let install_dir = install_dir.clone();
        let winetricks_cache = app.path().app_cache_dir().unwrap().join("winetricks");
        let winetricks_cache_str = winetricks_cache.to_str().unwrap().to_string();
        if !winetricks_cache.exists() { let _ = std::fs::create_dir_all(&winetricks_cache); }

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
                        if !s.success() { log::info!("Executing WineTricks command: \"{}\" failed with status: {}", command, s.code().unwrap()); show_dialog_with_callback(&app, "warning", "TwintailLauncher", "dialogs.winetricks_setup_failed", Some(vec!["dialogs.buttons.i_understand"]), None, None); }
                        s.success()
                    }
                    Err(_) => { log::error!("Executing WineTricks command: \"{}\" failed! Is command correct?", command); show_dialog_with_callback(&app, "warning", "TwintailLauncher", "dialogs.winetricks_setup_retry", Some(vec!["dialogs.buttons.i_understand"]), None, None); false }
                }
            }
            Err(_) => { log::error!("Executing WineTricks command \"{}\" failed catastrophically!", command); show_dialog_with_callback(&app, "warning", "TwintailLauncher", "dialogs.winetricks_exec_critical", Some(vec!["dialogs.buttons.i_understand"]), None, None); false }
        }
    })
}

#[cfg(target_os = "windows")]
pub fn launch<R: Runtime>(app: &AppHandle<R>, install: LauncherInstall, gm: GameManifest, gs: GlobalSettings) -> Result<bool, Error> {
    let dirp = std::path::Path::new(&install.directory.clone()).to_path_buf();
    let dir = dirp.to_str().unwrap().to_string();
    let game = gm.paths.exe_filename.clone();
    let exe = gm.paths.exe_filename.clone().split('/').last().unwrap().to_string();

    let pre_launch = install.pre_launch_command.clone();
    if !pre_launch.is_empty() {
        let mut cmd = Command::new(&pre_launch);
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());
        cmd.current_dir(dir.clone());

        match cmd.spawn() {
            Ok(mut child) => match child.try_wait() {
                Ok(Some(status)) => {
                    if !status.success() { log::info!("Executing prelaunch command: \"{}\" failed with status: {}", pre_launch, status.code().unwrap()); show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.prelaunch_cmd_failed", None, None, None); }
                }
                Ok(None) => { log::info!("Executing prelaunch command: \"{}\"", pre_launch); }
                Err(_) => { log::error!("Executing prelaunch command: \"{}\" failed! Is command correct?", pre_launch); show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.prelaunch_cmd_incorrect", None, None, None); }
            },
            Err(_) => { log::error!("Executing prelaunch command \"{}\" failed catastrophically!", pre_launch); show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.prelaunch_cmd_critical", None, None, None); }
        }
    }

    // Run xxmi first
    load_xxmi(app, install.clone(), gs.xxmi_path, exe.clone());
    load_fps_unlock(install.clone(), gm.biz.clone(), dir.clone(), gs.fps_unlock_path);

    let rslt = if install.launch_command.is_empty() {
        let mut args = install.launch_args.clone();
        let dir = dir.trim_matches('\\');
        let game = game.trim_matches('\\');
        let tmp = game.replace("/", "\\");

        let full_path = std::path::Path::new(dir).join(&tmp);
        let full_path_str = full_path.to_str().unwrap().replace("/", "\\");

        let xxmi_forced = install.use_xxmi && (gm.biz == "wuwa_global" || gm.biz == "endfield_global" || gm.biz == "nap_global");
        if install.use_xxmi && gm.biz == "wuwa_global" { args = args.split_whitespace().filter(|a| gm.extra.graphics_api_options.options.iter().all(|o| o.value.as_str() != *a)).collect::<Vec<_>>().join(" "); if !args.is_empty() { args += " "; } args += "-dx11"; }
        if install.use_xxmi && gm.biz == "endfield_global" { args = args.split_whitespace().filter(|a| gm.extra.graphics_api_options.options.iter().all(|o| o.value.as_str() != *a)).collect::<Vec<_>>().join(" "); if !args.is_empty() { args += " "; } args += "-force-d3d11"; }
        if install.use_xxmi && gm.biz == "nap_global" { args = args.split_whitespace().filter(|a| gm.extra.graphics_api_options.options.iter().all(|o| o.value.as_str() != *a)).collect::<Vec<_>>().join(" "); if !args.is_empty() { args += " "; } args += "-use-d3d11"; }
        if gm.extra.switches.graphics_api && !xxmi_forced && !args.split_whitespace().any(|a| gm.extra.graphics_api_options.options.iter().any(|o| o.value.as_str() == a)) && !install.graphics_api.is_empty() { if !args.is_empty() { args += " "; } args += &install.graphics_api; }

        let mut cmd = Command::new(&full_path_str);
        if !args.is_empty() { cmd.args(args.split_whitespace().collect::<Vec<_>>()); }
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());
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
                    if !status.success() { log::info!("Executing launch command: \"{}\" failed with status: {}", full_path_str, status.code().unwrap()); show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.launch_run_cmd_failed", None, None, None); }
                }
                Ok(None) => {
                    let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs().to_string();
                    update_install_last_played_by_id(app, install.id.clone(), time);
                    start_playtime_tracker(app, install.clone(), gm.clone(), exe.clone());
                    log::info!("Executing launch command: \"{}\"", full_path_str);
                }
                Err(_) => { log::error!("Executing launch command: \"{}\" failed! Is command correct?", full_path_str); show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.launch_run_cmd_incorrect", None, None, None); }
            },
            Err(_) => { log::error!("Executing launch command \"{}\" failed catastrophically!", full_path_str); show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.launch_cmd_critical", None, None, None); }
        }
        true
    } else {
        // We assume user knows what he/she is doing so we just execute command that is configured without any checks
        let dir = dir.trim_matches('\\');
        let game = game.trim_matches('\\');
        let tmp = game.replace("/", "\\");

        let full_path = std::path::Path::new(dir).join(&tmp);
        let full_path_str = full_path.to_str().unwrap().replace("/", "\\");
        let c = install.launch_command.clone().replace("%install_dir%", dir).replace("%game_exe%", full_path_str.as_str());
        let mut args = install.launch_args.clone();

        let xxmi_forced = install.use_xxmi && (gm.biz == "wuwa_global" || gm.biz == "endfield_global" || gm.biz == "nap_global");
        if install.use_xxmi && gm.biz == "wuwa_global" { args = args.split_whitespace().filter(|a| gm.extra.graphics_api_options.options.iter().all(|o| o.value.as_str() != *a)).collect::<Vec<_>>().join(" "); if !args.is_empty() { args += " "; } args += "-dx11"; }
        if install.use_xxmi && gm.biz == "endfield_global" { args = args.split_whitespace().filter(|a| gm.extra.graphics_api_options.options.iter().all(|o| o.value.as_str() != *a)).collect::<Vec<_>>().join(" "); if !args.is_empty() { args += " "; } args += "-force-d3d11"; }
        if install.use_xxmi && gm.biz == "nap_global" { args = args.split_whitespace().filter(|a| gm.extra.graphics_api_options.options.iter().all(|o| o.value.as_str() != *a)).collect::<Vec<_>>().join(" "); if !args.is_empty() { args += " "; } args += "-use-d3d11"; }
        if gm.extra.switches.graphics_api && !xxmi_forced && !args.split_whitespace().any(|a| gm.extra.graphics_api_options.options.iter().any(|o| o.value.as_str() == a)) && !install.graphics_api.is_empty() { if !args.is_empty() { args += " "; } args += &install.graphics_api; }

        let mut cmd = Command::new(&c);
        if !args.is_empty() { cmd.args(args.split_whitespace().collect::<Vec<_>>()); }
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());
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
                    if !status.success() { log::info!("Executing launch command: \"{}\" failed with status: {}", c, status.code().unwrap()); show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.launch_cmd_failed", None, None, None); }
                }
                Ok(None) => {
                    let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs().to_string();
                    update_install_last_played_by_id(app, install.id.clone(), time);
                    start_playtime_tracker(app, install.clone(), gm.clone(), exe.clone());
                    log::info!("Executing launch command: \"{}\"", c);
                }
                Err(_) => { log::error!("Executing launch command: \"{}\" failed! Is command correct?", c); show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.launch_cmd_incorrect", None, None, None); }
            },
            Err(_) => { log::error!("Executing launch command \"{}\" failed catastrophically!", c); show_dialog_with_callback(&app, "error", "TwintailLauncher", "dialogs.launch_cmd_critical", None, None, None); }
        }
        true
    };
    Ok(rslt)
}

#[cfg(target_os = "windows")]
fn load_xxmi<R: Runtime>(app: &AppHandle<R>, install: LauncherInstall, xxmi_path: String, game: String) {
    if install.use_xxmi {
        let xxmi_path = xxmi_path.trim_matches('\\');
        let mipath = get_mi_path_from_game(game.clone()).unwrap();
        let mi_pathbuf = std::path::Path::new(&xxmi_path).join(&mipath);
        let game_dir = std::path::PathBuf::from(&install.directory);
        let loader_path = std::path::Path::new(xxmi_path).join("3dmloader.exe");
        let loader_path_str = loader_path.to_str().unwrap().replace("/", "\\");

        // Apply the installation tweaks
        let data = apply_xxmi_tweaks(mi_pathbuf, install.xxmi_config);
        crate::utils::db_manager::update_install_xxmi_config_by_id(&app, install.id, data);
        if mipath.to_ascii_lowercase().as_str() == "wwmi" { crate::utils::apply_wwmi_tweaks(game_dir.to_path_buf(), xxmi_path.to_string()); }

        let loader_mode = if mipath == "efmi" || mipath == "wwmi" { "inject" } else { "hook" };
        let mut cmd = Command::new(&loader_path_str);
        cmd.arg(&mipath);
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

        cmd.env("LOADER_MODE", loader_mode);
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());
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
            log::info!("Executing XXMI command: \"{}\"", loader_path_str);
            spawned.unwrap();
        }
    }
}

#[cfg(target_os = "windows")]
fn load_fps_unlock(install: LauncherInstall, biz: String, game_path: String, fpsunlock_path: String) {
    if install.use_fps_unlock {
        let fpsunlock_path = fpsunlock_path.trim_matches('\\');
        let loader_path = std::path::Path::new(fpsunlock_path).join("keqing_unlock.exe");
        let loader_path_str = loader_path.to_str().unwrap().replace("/", "\\");
        let fpsv = install.fps_value.clone();

        let mut cmd = Command::new(&loader_path_str);
        cmd.arg("run").arg(&biz).arg(&fpsv).arg("2000").arg("0").arg(&game_path);
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());
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
            log::info!("Executing FPS Unlocker command: \"{}\"", loader_path_str);
            spawned.unwrap();
        }
    }
}

fn start_playtime_tracker<R: Runtime>(app: &AppHandle<R>, install: LauncherInstall, gm: GameManifest, exe_name: String) {
    let app = app.clone();
    let install_id = install.id.clone();
    let base_playtime = install.total_playtime as u64;
    #[cfg(target_os = "linux")]
    let exe_name = { let stem = exe_name.split('.').next().unwrap_or(&exe_name); stem[..stem.len().min(15)].to_string() };
    std::thread::spawn(move || {
        let mut last_db_write_elapsed: u64 = 0;
        const POLL_MS: u64 = 500;
        const MAX_WAIT_POLLS: u64 = 240;
        let mut appeared = false;
        for _ in 0..MAX_WAIT_POLLS {
            if is_process_running(&exe_name) { appeared = true; break; }
            std::thread::sleep(std::time::Duration::from_millis(POLL_MS));
        }
        if !appeared {
            if cfg!(target_os = "linux") && gm.biz != "wuwa_global" {
                if install.use_xxmi && is_process_running("3dmloader.exe") { let _ = Command::new("bash").args(["-c", "for pid in $(pgrep -f 3dmloader.exe); do kill -9 -$pid; done"]).spawn(); log::debug!("Killing 3dmloader.exe as game crashed!"); }
                if install.use_fps_unlock && is_process_running("keqing_unlock.exe") { let _ = Command::new("bash").args(["-c", "for pid in $(pgrep -f keqing_unlock.exe); do kill -9 -$pid; done"]).spawn(); log::debug!("Killing keqing_unlock.exe as game crashed!"); }
            }
            return;
        }
        let mut rpc_client = None;
        if install.show_discord_rpc { rpc_client = discord_rpc::init(&app, install.clone(), gm.clone()); }
        let mut keepawake = None;
        if install.disable_system_idle { keepawake = prevent_system_idle(true); }
        let started = std::time::Instant::now();
        loop {
            std::thread::sleep(std::time::Duration::from_millis(POLL_MS));
            let running = is_process_running(&exe_name);
            let elapsed = started.elapsed().as_secs();
            if !running || elapsed - last_db_write_elapsed >= 10 {
                let new_total = base_playtime + elapsed;
                update_install_total_playtime_by_id(&app, install_id.clone(), new_total.to_string());
                last_db_write_elapsed = elapsed;
            }
            if !running {
                if install.show_discord_rpc { if let Some(ref mut client) = rpc_client { discord_rpc::terminate(client); } }
                if install.disable_system_idle { drop(keepawake); }
                app.emit("game_closed", install_id.clone()).unwrap();
                return;
            }
        }
    });
}
