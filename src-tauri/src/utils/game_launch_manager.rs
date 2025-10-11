use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::process::{Child, Command, Stdio};
use tauri::{AppHandle, Error};
use crate::commands::settings::GlobalSettings;
use crate::utils::repo_manager::{GameManifest, LauncherInstall};
use crate::utils::{get_mi_path_from_game, is_flatpak, send_notification};
use crate::utils::{PathResolve};
use fischl::utils::wait_for_process;

#[cfg(target_os = "linux")]
use crate::utils::{runner_from_runner_version, patch_hkrpg};
#[cfg(target_os = "linux")]
use crate::utils::repo_manager::{get_compatibility};
#[cfg(target_os = "linux")]
use tauri::Manager;
#[cfg(target_os = "linux")]
use std::os::unix::process::CommandExt;

#[cfg(target_os = "linux")]
pub fn launch(app: &AppHandle, install: LauncherInstall, gm: GameManifest, gs: GlobalSettings) -> Result<bool, Error> {
    let rm = get_compatibility(&app, &runner_from_runner_version(install.runner_version.clone()).unwrap()).unwrap();
    let is_proton = rm.display_name.to_ascii_lowercase().contains("proton") && !rm.display_name.to_ascii_lowercase().contains("wine");

    let dir = Path::new(install.directory.as_str()).follow_symlink()?.to_str().unwrap().to_string();
    let prefix = Path::new(install.runner_prefix.as_str()).follow_symlink()?.to_str().unwrap().to_string();
    let runner = Path::new(install.runner_path.as_str()).follow_symlink()?.to_str().unwrap().to_string();
    let game = gm.paths.exe_filename.clone();
    let exe = gm.paths.exe_filename.clone().split('/').last().unwrap().to_string();
    let steamrt = app.path().app_data_dir()?.follow_symlink()?.join("compatibility/runners/steamrt/_v2-entry-point").follow_symlink()?.to_str().unwrap().to_string();
    //let reaper = app.path().app_data_dir()?.follow_symlink()?.join("reaper").follow_symlink()?.to_str().unwrap().to_string();

    let pre_launch = install.pre_launch_command.clone();
    let wine64 = if rm.paths.wine64.is_empty() { rm.paths.wine32 } else { rm.paths.wine64 };

    if !pre_launch.is_empty() {
        let command = format!("{pre_launch}").replace("%prefix%", prefix.clone().as_str()).replace("%runner_dir%", runner.clone().as_str()).replace("%runner%", &*(runner.clone() + "/" + wine64.as_str())).replace("%install_dir%", dir.clone().as_str()).replace("%game_exe%", &*(dir.clone() + "/" + exe.clone().as_str()));

        let mut cmd = Command::new("bash");
        cmd.arg("-c");
        cmd.arg(&command);

        /*cmd.env("SteamAppId", "55555555555555");
        cmd.env("SteamGameId", "55555555555555");*/
        cmd.env("WINEARCH","win64");
        cmd.env("WINEPREFIX", prefix.clone());
        cmd.env("STEAM_COMPAT_APP_ID", "0");
        cmd.env("STEAM_COMPAT_DATA_PATH", prefix.clone());
        cmd.env("STEAM_COMPAT_INSTALL_PATH", dir.clone());
        cmd.env("STEAM_COMPAT_CLIENT_INSTALL_PATH", "");
        cmd.env("STEAM_COMPAT_TOOL_PATHS", runner.clone());
        cmd.env("STEAM_COMPAT_SHADER_PATH", prefix.clone() + "/shadercache");
        cmd.env("PROTONFIXES_DISABLE", "1");
        cmd.env("WINEDLLOVERRIDES", "lsteamclient=d;KRSDKExternal.exe=d;dbghelp=n,b");

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.current_dir(dir.clone());
        cmd.process_group(0);

        match cmd.spawn() {
            Ok(mut child) => {
                match child.try_wait() {
                    Ok(Some(status)) => { if !status.success() { send_notification(&app, "Failed to run prelaunch command! Please try again or check install settings.", None); } }
                    Ok(None) => { write_log(app, Path::new(&dir).follow_symlink()?.to_path_buf(), child, "pre_launch.log".parse().unwrap()); }
                    Err(_) => { send_notification(&app, "Failed to run prelaunch command! Please try again or check the command correctness.", None); }
                }
            }
            Err(_) => { send_notification(&app, "Failed to run prelaunch command! Something serious is wrong.", None); }
        }
    }

    let rslt = if install.launch_command.is_empty() {
        let mut args = String::new();
        if !install.launch_args.is_empty() {
            args = install.clone().launch_args;
            if install.use_xxmi && gm.biz == "wuwa_global" { args += " -dx11" }
        } else {
            if install.use_xxmi && gm.biz == "wuwa_global" { args += "-dx11" }
        }
        let mut command = if rm.display_name.to_ascii_lowercase().contains("proton") && !rm.display_name.to_ascii_lowercase().contains("wine") {
            let steamrt_run = if is_flatpak() {
                //let appid = std::env::var("SteamGameId").ok().and_then(|s| s.parse::<i64>().ok()).unwrap_or(1) >> 32;
                format!("'{steamrt}' --verb=waitforexitandrun -- '{runner}/{wine64}' waitforexitandrun '{dir}/{game}' {args}")
            } else {
                format!("'{steamrt}' --verb=waitforexitandrun -- '{runner}/{wine64}' waitforexitandrun '{dir}/{game}' {args}")
            };
            if install.use_gamemode { format!("gamemoderun {steamrt_run}") } else { format!("{steamrt_run}") }
        } else {
            if install.use_gamemode { format!("gamemoderun '{runner}/{wine64}' '{dir}/{game}' {args}") } else { format!("'{runner}/{wine64}' '{dir}/{game}' {args}") }
        };

        if install.use_jadeite {
            let jadeite_path = gs.jadeite_path.clone();
            command = if rm.display_name.to_ascii_lowercase().contains("proton") && !rm.display_name.to_ascii_lowercase().contains("wine") {
                let steamrt_run = if is_flatpak() {
                    //let appid = std::env::var("SteamGameId").ok().and_then(|s| s.parse::<i64>().ok()).unwrap_or(1) >> 32;
                    format!("'{steamrt}' --verb=waitforexitandrun -- '{runner}/{wine64}' waitforexitandrun '{jadeite_path}/jadeite.exe' '{dir}/{game}' -- {args}")
                } else {
                    format!("'{steamrt}' --verb=waitforexitandrun -- '{runner}/{wine64}' waitforexitandrun '{jadeite_path}/jadeite.exe' '{dir}/{game}' -- {args}")
                };
                if install.use_gamemode { format!("gamemoderun {steamrt_run}") } else { format!("{steamrt_run}") }
            } else {
                if install.use_gamemode { format!("gamemoderun '{runner}/{wine64}' '{jadeite_path}/jadeite.exe' '{dir}/{game}' -- {args}") } else { format!("'{runner}/{wine64}' '{jadeite_path}/jadeite.exe' '{dir}/{game}' -- {args}") }
            };
        }

        let mut cmds = String::new();
        if install.use_xxmi {
            let xxmi_path = gs.xxmi_path.clone();
            let mi = get_mi_path_from_game(exe.clone());
            let mi_path = Path::new(&gs.xxmi_path.clone()).join(mi.unwrap()).to_str().unwrap().to_string();
            cmds += format!("{prefix}/pfx/drive_c/windows/system32/cmd.exe /c 'z:\\{xxmi_path}/3dmloader.exe' 'z:\\{mi_path}'").as_str();
        }

        let mut cmd = Command::new("bash");
        cmd.arg("-c");
        cmd.arg(&command);

       if !cmds.is_empty() { cmd.env("PROTON_REMOTE_DEBUG_CMD", cmds); }
        /*cmd.env("SteamAppId", "400000000000");
        cmd.env("SteamGameId", "400000000000");*/
        cmd.env("WINEARCH","win64");
        cmd.env("WINEPREFIX", prefix.clone());
        cmd.env("STEAM_COMPAT_APP_ID", "0");
        cmd.env("STEAM_COMPAT_DATA_PATH", prefix.clone());
        cmd.env("STEAM_COMPAT_INSTALL_PATH", dir.clone());
        cmd.env("STEAM_COMPAT_CLIENT_INSTALL_PATH", "");
        cmd.env("STEAM_COMPAT_TOOL_PATHS", runner.clone());
        cmd.env("STEAM_COMPAT_SHADER_PATH", prefix.clone() + "/shadercache");
        cmd.env("PROTONFIXES_DISABLE", "1");
        cmd.env("WINEDLLOVERRIDES", "lsteamclient=d;KRSDKExternal.exe=d");
        if install.use_mangohud {
            cmd.env("MANGOHUD","1");
            if install.mangohud_config_path != "" { cmd.env("MANGOHUD_CONFIGFILE", format!("{}", install.clone().mangohud_config_path).as_str()); }
        }

        // Make it more convenient for wuwa players because we can not load protonfixes
        if gm.biz == "wuwa_global" { cmd.env("SteamOS","1"); }
        // Set override for hkrpg fix
        if gm.biz == "hkrpg_global" { if !install.use_jadeite { cmd.env("WINEDLLOVERRIDES", "lsteamclient=d;KRSDKExternal.exe=d;dbghelp=n,b"); patch_hkrpg(app, dir.clone()); } }

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
                match (tmp.next(), tmp.next()) {
                    (Some(k), Some(v)) if !k.is_empty() => Some(Some((k, v.replace("\"", "")))),
                    _ => None,
                }
            }).collect::<Option<Vec<_>>>().and_then(|vec| Some(vec.into_iter().flatten().collect()));
            if let Some(env_vars) = parsed { for (k, v) in env_vars { cmd.env(k, v); } }
        }

        match cmd.spawn() {
            Ok(mut child) => {
                match child.try_wait() {
                    Ok(Some(status)) => { if !status.success() { send_notification(&app, "Failed to run launch command! Please try again or check install settings.", None); } }
                    Ok(None) => {
                        //load_xxmi(app, install.clone(), gm.biz.clone(), prefix.clone(), gs.xxmi_path.clone(), runner.clone(), wine64.clone(), exe.clone(), is_proton);
                        load_fps_unlock(app, install.clone(), gm.biz.clone(), prefix.clone(), gs.fps_unlock_path.clone(), dir.clone(), runner.clone(), wine64.clone(), exe.clone(), is_proton);
                        write_log(app, Path::new(&dir).follow_symlink()?.to_path_buf(), child, "game.log".parse().unwrap());
                    }
                    Err(_) => { send_notification(&app, "Failed to run launch command! Please try again or check the command correctness.", None); }
                }
            }
            Err(_) => { send_notification(&app, "Failed to run launch command! Something serious is wrong.", None); }
        }
        true
    } else {
        // We assume user knows what he/she is doing so we just execute command that is configured without any checks
        let c = install.launch_command.clone();
        let mut args = String::new();
        let mut command = format!("{c}").replace("%prefix%", prefix.clone().as_str()).replace("%runner_dir%", runner.clone().as_str()).replace("%runner%", &*(runner.clone() + "/" + wine64.as_str())).replace("%install_dir%", dir.clone().as_str()).replace("%game_exe%", &*(dir.clone() + "/" + exe.clone().as_str()));

        if !install.launch_args.is_empty() {
            args = install.clone().launch_args;
            if install.use_xxmi && gm.biz == "wuwa_global" { args += " -dx11" }
            command = format!("{c} {args}").replace("%prefix%", prefix.clone().as_str()).replace("%runner_dir%", runner.clone().as_str()).replace("%runner%", &*(runner.clone() + "/" + wine64.as_str())).replace("%install_dir%", dir.clone().as_str()).replace("%game_exe%", &*(dir.clone() + "/" + exe.clone().as_str()));
        } else {
            if install.use_xxmi && gm.biz == "wuwa_global" { args += "-dx11" }
        }

        let mut cmds = String::new();
        if install.use_xxmi {
            let xxmi_path = gs.xxmi_path.clone();
            let mi = get_mi_path_from_game(exe.clone());
            let mi_path = Path::new(&gs.xxmi_path.clone()).join(mi.unwrap()).to_str().unwrap().to_string();
            cmds += format!("{prefix}/pfx/drive_c/windows/system32/cmd.exe /c 'z:\\{xxmi_path}/3dmloader.exe' 'z:\\{mi_path}'").as_str();
        }

        let mut cmd = Command::new("bash");
        cmd.arg("-c");
        cmd.arg(&command);

        if !cmds.is_empty() { cmd.env("PROTON_REMOTE_DEBUG_CMD", cmds); }
        /*cmd.env("SteamAppId", "400000000000");
        cmd.env("SteamGameId", "400000000000");*/
        cmd.env("WINEARCH","win64");
        cmd.env("WINEPREFIX", prefix.clone());
        cmd.env("STEAM_COMPAT_APP_ID", "0");
        cmd.env("STEAM_COMPAT_DATA_PATH", prefix.clone());
        cmd.env("STEAM_COMPAT_INSTALL_PATH", dir.clone());
        cmd.env("STEAM_COMPAT_CLIENT_INSTALL_PATH", "");
        cmd.env("STEAM_COMPAT_TOOL_PATHS", runner.clone());
        cmd.env("STEAM_COMPAT_SHADER_PATH", prefix.clone() + "/shadercache");
        cmd.env("PROTONFIXES_DISABLE", "1");
        cmd.env("WINEDLLOVERRIDES", "lsteamclient=d;KRSDKExternal.exe=d");
        if install.use_mangohud {
            cmd.env("MANGOHUD","1");
            if install.mangohud_config_path != "" { cmd.env("MANGOHUD_CONFIGFILE", format!("{}", install.clone().mangohud_config_path).as_str()); }
        }

        // Make it more convenient for wuwa players because we can not load protonfixes
        if gm.biz == "wuwa_global" { cmd.env("SteamOS","1"); }
        // Set override for hkrpg fix
        if gm.biz == "hkrpg_global" { if !install.use_jadeite { cmd.env("WINEDLLOVERRIDES", "lsteamclient=d;KRSDKExternal.exe=d;dbghelp=n,b"); patch_hkrpg(app, dir.clone()); } }

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
                match (tmp.next(), tmp.next()) {
                    (Some(k), Some(v)) if !k.is_empty() => Some(Some((k, v.replace("\"", "")))),
                    _ => None,
                }
            }).collect::<Option<Vec<_>>>().and_then(|vec| Some(vec.into_iter().flatten().collect()));
            if let Some(env_vars) = parsed { for (k, v) in env_vars { cmd.env(k, v); } }
        }

        match cmd.spawn() {
            Ok(mut child) => {
                match child.try_wait() {
                    Ok(Some(status)) => { if !status.success() { send_notification(&app, "Failed to run launch command! Please try again or check install settings.", None); } }
                    Ok(None) => {
                        //load_xxmi(app, install.clone(), gm.biz.clone(), prefix.clone(), gs.xxmi_path.clone(), runner.clone(), wine64.clone(), exe.clone(), is_proton);
                        load_fps_unlock(app, install.clone(), gm.biz.clone(), prefix.clone(), gs.fps_unlock_path.clone(), dir.clone(), runner.clone(), wine64.clone(), exe.clone(), is_proton);
                        write_log(app, Path::new(&dir).follow_symlink()?.to_path_buf(), child, "game.log".parse().unwrap());
                    }
                    Err(_) => { send_notification(&app, "Failed to run launch command! Please try again or check the command correctness.", None); }
                }
            }
            Err(_) => { send_notification(&app, "Failed to run launch command! Something serious is wrong.", None); }
        }
        true
    };
    if rslt { Ok(true) } else { Ok(false) }
}

#[cfg(target_os = "linux")]
#[allow(dead_code)]
fn load_xxmi(app: &AppHandle, install: LauncherInstall, biz: String, prefix: String, xxmi_path: String, runner: String, wine64: String, game: String, is_proton: bool) {
    if install.use_xxmi {
        if biz == "hk4e_global" { std::thread::sleep(std::time::Duration::from_millis(3000)); } else { std::thread::sleep(std::time::Duration::from_millis(300)); }
        let xxmi_path = xxmi_path.clone();
        let mipath = get_mi_path_from_game(game.clone()).unwrap();
        let command = if is_proton { format!("'{runner}/{wine64}' run 'z:\\{xxmi_path}/3dmloader.exe' {mipath}") } else { format!("'{runner}/{wine64}' 'z:\\{xxmi_path}/3dmloader.exe' {mipath}") };

        let mut cmd = Command::new("bash");
        cmd.arg("-c");
        cmd.arg(&command);

        /*cmd.env("SteamAppId", "500000000000");
        cmd.env("SteamGameId", "500000000000");*/
        cmd.env("WINEARCH","win64");
        cmd.env("WINEPREFIX", prefix.clone());
        cmd.env("STEAM_COMPAT_APP_ID", "0");
        cmd.env("STEAM_COMPAT_DATA_PATH", prefix.clone());
        cmd.env("STEAM_COMPAT_INSTALL_PATH", xxmi_path.clone());
        cmd.env("STEAM_COMPAT_CLIENT_INSTALL_PATH", "");
        cmd.env("STEAM_COMPAT_TOOL_PATHS", runner.clone());
        cmd.env("PROTONFIXES_DISABLE", "1");
        cmd.env("WINEDLLOVERRIDES", "lsteamclient=d;KRSDKExternal.exe=d;dbghelp=n,b");

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.current_dir(xxmi_path.clone());
        cmd.process_group(0);

        let spawn = cmd.spawn();
        if spawn.is_ok() {
            let process = spawn.unwrap();
            write_log(app, Path::new(&xxmi_path).follow_symlink().unwrap().to_path_buf(), process, "xxmi.log".parse().unwrap());
        }
    }
}

#[cfg(target_os = "linux")]
fn load_fps_unlock(app: &AppHandle, install: LauncherInstall, biz: String, prefix: String, fpsunlock_path: String, game_path: String, runner: String, wine64: String, game: String, is_proton: bool) {
    if install.use_fps_unlock {
        wait_for_process(game.as_str(), 100,30, |found| {
            if found {
                let fpsunlock_path = fpsunlock_path.clone();
                let fpsv = install.fps_value.clone();
                let command = if is_proton { format!("'{runner}/{wine64}' run 'z:\\{fpsunlock_path}/fpsunlock.exe' run {biz} {fpsv} 3000 '{game_path}'") } else { format!("'{runner}/{wine64}' 'z:\\{fpsunlock_path}/fpsunlock.exe' run {biz} {fpsv} 3000 '{game_path}'") };

                let mut cmd = Command::new("bash");
                cmd.arg("-c");
                cmd.arg(&command);

                /*cmd.env("SteamAppId", "600000000000");
                cmd.env("SteamGameId", "600000000000");*/
                cmd.env("WINEARCH","win64");
                cmd.env("WINEPREFIX", prefix.clone());
                cmd.env("STEAM_COMPAT_APP_ID", "0");
                cmd.env("STEAM_COMPAT_DATA_PATH", prefix.clone());
                cmd.env("STEAM_COMPAT_INSTALL_PATH", fpsunlock_path.clone());
                cmd.env("STEAM_COMPAT_CLIENT_INSTALL_PATH", "");
                cmd.env("STEAM_COMPAT_TOOL_PATHS", runner.clone());
                cmd.env("PROTONFIXES_DISABLE", "1");
                cmd.env("WINEDLLOVERRIDES", "lsteamclient=d;KRSDKExternal.exe=d;dbghelp=n,b");

                cmd.stdout(Stdio::piped());
                cmd.stderr(Stdio::piped());
                cmd.current_dir(fpsunlock_path.clone());
                cmd.process_group(0);

                let spawn = cmd.spawn();
                if spawn.is_ok() {
                    let process = spawn.unwrap();
                    write_log(app, Path::new(&fpsunlock_path.clone()).follow_symlink().unwrap().to_path_buf(), process, "fps_unlocker.log".parse().unwrap());
                }
                true
            } else {
                false
            }
        });
    }
}

#[cfg(target_os = "windows")]
pub fn launch(app: &AppHandle, install: LauncherInstall, gm: GameManifest, gs: GlobalSettings) -> Result<bool, Error> {
    let dir = install.directory.clone();
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
            Ok(mut child) => {
                match child.try_wait() {
                    Ok(Some(status)) => { if !status.success() { send_notification(&app, "Failed to run prelaunch command! Please try again or check install settings.", None); } }
                    Ok(None) => { write_log(app, Path::new(&dir).follow_symlink()?.to_path_buf(), child, "pre_launch.log".parse().unwrap()); }
                    Err(_) => { send_notification(&app, "Failed to run prelaunch command! Please try again or check the command correctness.", None); }
                }
            }
            Err(_) => { send_notification(&app, "Failed to run prelaunch command! Something serious is wrong.", None); }
        }
    }

    // Run xxmi first
    load_xxmi(app, install.clone(), gs.xxmi_path, exe.clone());
    load_fps_unlock(app, install.clone(), gm.biz.clone(), dir.clone(), gs.fps_unlock_path, game.clone());

    let rslt = if install.launch_command.is_empty() {
        let args;
        let dir = dir.trim_matches('\\');
        let game = game.trim_matches('\\');
        let tmp = game.replace("/", "\\");

        let full_path = Path::new(dir).join(&tmp);
        let full_path_str = full_path.to_str().unwrap().replace("/", "\\");
        let mut command = format!("Start-Process -FilePath '{full_path_str}' -WorkingDirectory '{dir}' -Verb RunAs");

        if !install.launch_args.is_empty() {
            args = &install.launch_args;
            command = format!("Start-Process -FilePath '{full_path_str}' -ArgumentList '{args}' -WorkingDirectory '{dir}' -Verb RunAs");
        }

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
                match (tmp.next(), tmp.next()) {
                    (Some(k), Some(v)) if !k.is_empty() => Some(Some((k, v.replace("\"", "")))),
                    _ => None,
                }
            }).collect::<Option<Vec<_>>>().and_then(|vec| Some(vec.into_iter().flatten().collect()));

            if let Some(env_vars) = parsed {
                for (k, v) in env_vars { cmd.env(k, v); }
            }
        }

        match cmd.spawn() {
            Ok(mut child) => {
                match child.try_wait() {
                    Ok(Some(status)) => { if !status.success() { send_notification(&app, "Failed to run launch command! Please try again or check install settings.", None); } }
                    Ok(None) => {
                        write_log(app, Path::new(&dir).follow_symlink()?.to_path_buf(), child, "game.log".parse().unwrap());
                    }
                    Err(_) => { send_notification(&app, "Failed to run launch command! Please try again or check the command correctness.", None); }
                }
            }
            Err(_) => { send_notification(&app, "Failed to run launch command! Something serious is wrong.", None); }
        }
        true
    } else {
        // We assume user knows what he/she is doing so we just execute command that is configured without any checks
        let c = install.launch_command.clone();
        let args;
        let mut command = format!("Start-Process -FilePath '{c}' -WorkingDirectory '{dir}' -Verb RunAs");

        if !install.launch_args.is_empty() {
            args = &install.launch_args;
            command = format!("Start-Process -FilePath '{c}' -ArgumentList '{args}' -WorkingDirectory '{dir}' -Verb RunAs");
        }

        let mut cmd = Command::new("powershell");
        cmd.arg("-Command");
        cmd.arg(&command);

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.current_dir(dir.clone());

        if !install.env_vars.is_empty() {
            let envs = install.env_vars.clone();
            let splitted = envs.split(";").collect::<Vec<&str>>();
            let parsed: Option<Vec<(&str, String)>> = splitted.iter().map(|env| {
                if env.is_empty() { return Some(None); }
                let mut tmp = env.splitn(2, "=");
                match (tmp.next(), tmp.next()) {
                    (Some(k), Some(v)) if !k.is_empty() => Some(Some((k, v.replace("\"", "")))),
                    _ => None,
                }
            }).collect::<Option<Vec<_>>>().and_then(|vec| Some(vec.into_iter().flatten().collect()));

            if let Some(env_vars) = parsed {
                for (k, v) in env_vars { cmd.env(k, v); }
            }
        }

        match cmd.spawn() {
            Ok(mut child) => {
                match child.try_wait() {
                    Ok(Some(status)) => { if !status.success() { send_notification(&app, "Failed to run launch command! Please try again or check install settings.", None); } }
                    Ok(None) => {
                        write_log(app, Path::new(&dir).follow_symlink()?.to_path_buf(), child, "game.log".parse().unwrap());
                    }
                    Err(_) => { send_notification(&app, "Failed to run launch command! Please try again or check the command correctness.", None); }
                }
            }
            Err(_) => { send_notification(&app, "Failed to run launch command! Something serious is wrong.", None); }
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
        let loader_path = Path::new(xxmi_path).join("3dmloader.exe");
        let loader_path_str = loader_path.to_str().unwrap().replace("/", "\\");
        let command = format!("Start-Process -FilePath '{}' -ArgumentList '{}' -WorkingDirectory '{}' -Verb RunAs", loader_path_str, mipath, xxmi_path);

        let mut cmd = Command::new("powershell");
        cmd.arg("-Command");
        cmd.arg(&command);

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.current_dir(xxmi_path);

        let spawned = cmd.spawn();
        if spawned.is_ok() {
            let process = spawned.unwrap();
            write_log(app, Path::new(&xxmi_path).to_path_buf(), process, "xxmi.log".parse().unwrap());
        }
    }
}

#[cfg(target_os = "windows")]
fn load_fps_unlock(app: &AppHandle, install: LauncherInstall, biz: String, game_path: String, fpsunlock_path: String, game: String) {
    if install.use_fps_unlock {
        wait_for_process(game.as_str(), 100,30, |found| {
            if found {
                let fpsunlock_path = fpsunlock_path.trim_matches('\\');
                let loader_path = Path::new(fpsunlock_path).join("fpsunlock.exe");
                let loader_path_str = loader_path.to_str().unwrap().replace("/", "\\");
                let fpsv = install.fps_value.clone();
                let args = format!("run {} {} 3000 \"{}\"", biz, fpsv, game_path);
                let command = format!("Start-Process -FilePath '{}' -ArgumentList '{}' -WorkingDirectory '{}' -Verb RunAs", loader_path_str, args, fpsunlock_path);

                let mut cmd = Command::new("powershell");
                cmd.arg("-Command");
                cmd.arg(&command);

                cmd.stdout(Stdio::piped());
                cmd.stderr(Stdio::piped());
                cmd.current_dir(fpsunlock_path);

                let spawned = cmd.spawn();
                if spawned.is_ok() {
                    let process = spawned.unwrap();
                    write_log(app, Path::new(&fpsunlock_path).to_path_buf(), process, "fps_unlocker.log".parse().unwrap());
                }
                true
            } else {
                false
            }
        });
    }
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
            send_notification(&ac, &message, None);
        }

        if let Ok(mut file) = game_output.lock() { file.flush().unwrap(); }
        drop(game_output);
        if let Some(join) = stdout_join { join.join().map_err(|err| format!("Failed to join stdout reader thread: {err:?}")).unwrap().unwrap(); }
        if let Some(join) = stderr_join { join.join().map_err(|err| format!("Failed to join stderr reader thread: {err:?}")).unwrap().unwrap(); }
    });
}