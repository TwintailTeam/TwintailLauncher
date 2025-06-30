use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::process::{Child, Command, Stdio};
use tauri::{AppHandle, Error};
use crate::commands::settings::GlobalSettings;
use crate::utils::repo_manager::{GameManifest, LauncherInstall};
use crate::utils::get_mi_path_from_game;

#[cfg(target_os = "linux")]
use std::os::unix::process::CommandExt;
#[cfg(target_os = "linux")]
use crate::utils::runner_from_runner_version;
#[cfg(target_os = "linux")]
use crate::utils::repo_manager::{get_compatibility};
#[cfg(target_os = "linux")]
use fischl::utils::wait_for_process;

#[cfg(target_os = "linux")]
pub fn launch(app: &AppHandle, install: LauncherInstall, gm: GameManifest, gs: GlobalSettings) -> Result<bool, Error> {
    let rm = get_compatibility(&app, &runner_from_runner_version(install.runner_version.clone()).unwrap()).unwrap();

    let dir = install.directory.clone();
    let prefix = install.runner_prefix.clone();
    let runner = install.runner_path.clone();
    let game = gm.paths.exe_filename.clone();
    let exe = gm.paths.exe_filename.clone().split('/').last().unwrap().to_string();

    let pre_launch = install.pre_launch_command.clone();
    let wine64 = if rm.paths.wine64.is_empty() { rm.paths.wine32 } else { rm.paths.wine64 };

    if !pre_launch.is_empty() {
        let command = format!("'{pre_launch}'"); //format!("'{runner}/{wine64}' '{pre_launch}'");

        let mut cmd = Command::new("bash");
        cmd.arg("-c");
        cmd.arg(&command);

        cmd.env("WINEARCH","win64");
        cmd.env("WINEPREFIX", prefix.clone());
        cmd.env("STEAM_COMPAT_DATA_PATH", prefix.clone());
        cmd.env("STEAM_COMPAT_CLIENT_INSTALL_PATH", "");

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.current_dir(dir.clone());
        cmd.process_group(0);

        let spawned = cmd.spawn();
        if spawned.is_ok() {
            let process = spawned?;
            write_log(Path::new(&dir.clone()).to_path_buf(), process, "pre_launch.log".parse().unwrap());
        }
    }

    let rslt = if install.launch_command.is_empty() {
        let mut args = "";
        if !install.launch_args.is_empty() { args = &install.launch_args; }
        let mut command = if rm.display_name.to_ascii_lowercase().contains("proton") && !rm.display_name.to_ascii_lowercase().contains("wine") { format!("'{runner}/{wine64}' run '{dir}/{game}' {args}") } else { format!("'{runner}/{wine64}' '{dir}/{game}' {args}") };

        if install.use_jadeite {
            let jadeite_path = gs.jadeite_path.clone();
            command = if rm.display_name.to_ascii_lowercase().contains("proton") && !rm.display_name.to_ascii_lowercase().contains("wine") { format!("'{runner}/{wine64}' run '{jadeite_path}/jadeite.exe' '{dir}/{game}' -- {args}") } else { format!("'{runner}/{wine64}' '{jadeite_path}/jadeite.exe' '{dir}/{game}' -- {args}") };
        }

        let mut cmd = Command::new("bash");
        cmd.arg("-c");
        cmd.arg(&command);

        cmd.env("WINEARCH","win64");
        cmd.env("WINEPREFIX", prefix.clone());
        cmd.env("STEAM_COMPAT_DATA_PATH", prefix.clone());
        cmd.env("STEAM_COMPAT_CLIENT_INSTALL_PATH", "");

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.current_dir(dir.clone());
        cmd.process_group(0); // Start as detached process so killing launcher does not kill the game

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

        let spawned = cmd.spawn();
        if spawned.is_ok() {
            let process = spawned?;
            let is_proton = rm.display_name.to_ascii_lowercase().contains("proton") && !rm.display_name.to_ascii_lowercase().contains("wine");

            load_xxmi(install.clone(), prefix.clone(), gs.xxmi_path, runner.clone(), wine64.clone(), exe.clone(), is_proton);
            load_fps_unlock(install, prefix, gs.fps_unlock_path, runner, wine64, exe.clone(), is_proton);
            write_log(Path::new(&dir.clone()).to_path_buf(), process, "game.log".parse().unwrap());
            true
        } else {
            false
        }
    } else {
        // We assume user knows what he/she is doing so we just execute command that is configured without any checks
        let c = install.launch_command.clone();
        let args;
        let mut command = if rm.display_name.to_ascii_lowercase().contains("proton") && !rm.display_name.to_ascii_lowercase().contains("wine") { format!("'{runner}/{wine64}' run '{c}'") } else { format!("'{runner}/{wine64}' '{c}'") };

        if !install.launch_args.is_empty() {
            args = &install.launch_args;
            command = if rm.display_name.to_ascii_lowercase().contains("proton") && !rm.display_name.to_ascii_lowercase().contains("wine") { format!("'{runner}/{wine64}' run '{c}' {args}") } else { format!("'{runner}/{wine64}' '{c}' {args}") };
        }

        let mut cmd = Command::new("bash");
        cmd.arg("-c");
        cmd.arg(&command);

        cmd.env("WINEARCH","win64");
        cmd.env("WINEPREFIX", prefix.clone());
        cmd.env("STEAM_COMPAT_DATA_PATH", prefix.clone());
        cmd.env("STEAM_COMPAT_CLIENT_INSTALL_PATH", "");

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

            if let Some(env_vars) = parsed {
                for (k, v) in env_vars { cmd.env(k, v); }
            }
        }

        let spawned = cmd.spawn();
        if spawned.is_ok() {
            let process = spawned?;
            let is_proton = rm.display_name.to_ascii_lowercase().contains("proton") && !rm.display_name.to_ascii_lowercase().contains("wine");

            load_xxmi(install.clone(), prefix.clone(), gs.xxmi_path, runner.clone(), wine64.clone(), exe.clone(), is_proton);
            load_fps_unlock(install, prefix, gs.fps_unlock_path, runner, wine64, exe.clone(), is_proton);
            write_log(Path::new(&dir.clone()).to_path_buf(), process, "game.log".parse().unwrap());
            true
        } else {
            false
        }
    };

    Ok(rslt)
}

#[cfg(target_os = "linux")]
fn load_xxmi(install: LauncherInstall, prefix: String, xxmi_path: String, runner: String, wine64: String, game: String, is_proton: bool) {
    if install.use_xxmi {
        let xxmi_path = xxmi_path.clone();
        let mipath = get_mi_path_from_game(game.clone()).unwrap();
        let command = if is_proton { format!("'{runner}/{wine64}' run 'z:\\{xxmi_path}/3dmloader.exe' {mipath}") } else { format!("'{runner}/{wine64}' 'z:\\{xxmi_path}/3dmloader.exe' {mipath}") };

        let mut cmd = Command::new("bash");
        cmd.arg("-c");
        cmd.arg(&command);

        cmd.env("WINEARCH","win64");
        cmd.env("WINEPREFIX", prefix.clone());
        cmd.env("STEAM_COMPAT_DATA_PATH", prefix.clone());
        cmd.env("STEAM_COMPAT_CLIENT_INSTALL_PATH", "");

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.current_dir(xxmi_path.clone());
        cmd.process_group(0);

        let spawn = cmd.spawn();
        if spawn.is_ok() {
            let process = spawn.unwrap();
            write_log(Path::new(&xxmi_path.clone()).to_path_buf(), process, "xxmi.log".parse().unwrap());
        }
    }
}

#[cfg(target_os = "linux")]
fn load_fps_unlock(install: LauncherInstall, prefix: String, fpsunlock_path: String, runner: String, wine64: String, game: String, is_proton: bool) {
    if install.use_fps_unlock {
        wait_for_process(game.as_str(), 100,30, |found| {
            if found {
                let fpsunlock_path = fpsunlock_path.clone();
                let fpsv = install.fps_value.clone();
                let command = if is_proton { format!("'{runner}/{wine64}' run 'z:\\{fpsunlock_path}/fpsunlock.exe' {fpsv} 3000") } else { format!("'{runner}/{wine64}' 'z:\\{fpsunlock_path}/fpsunlock.exe' {fpsv} 3000") };

                let mut cmd = Command::new("bash");
                cmd.arg("-c");
                cmd.arg(&command);

                cmd.env("WINEARCH","win64");
                cmd.env("WINEPREFIX", prefix.clone());
                cmd.env("STEAM_COMPAT_DATA_PATH", prefix.clone());
                cmd.env("STEAM_COMPAT_CLIENT_INSTALL_PATH", "");

                cmd.stdout(Stdio::piped());
                cmd.stderr(Stdio::piped());
                cmd.current_dir(fpsunlock_path.clone());
                cmd.process_group(0);

                let spawn = cmd.spawn();
                if spawn.is_ok() {
                    let process = spawn.unwrap();
                    write_log(Path::new(&fpsunlock_path.clone()).to_path_buf(), process, "fps_unlocker.log".parse().unwrap());
                }
                true
            } else {
                false
            }
        });
    }
}

#[cfg(target_os = "windows")]
pub fn launch(_app: &AppHandle, install: LauncherInstall, gm: GameManifest, gs: GlobalSettings) -> Result<bool, Error> {
    let dir = install.directory.clone();
    let game = gm.paths.exe_filename.clone();
    let exe = gm.paths.exe_filename.clone().split('/').last().unwrap().to_string();

    let pre_launch = install.pre_launch_command.clone();

    if !pre_launch.is_empty() {
        let command = format!("\"{}\"", pre_launch);

        let mut cmd = Command::new("cmd");
        cmd.arg("/C").arg("start").arg("/b").arg("");
        cmd.arg(&command);

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.current_dir(dir.clone());

        let spawned = cmd.spawn();
        if spawned.is_ok() {
            let process = spawned?;
            write_log(Path::new(&dir.clone()).to_path_buf(), process, "pre_launch.log".parse().unwrap());
        }
    }

    let rslt = if install.launch_command.is_empty() {
        let mut args = "";
        if !install.launch_args.is_empty() { args = &install.launch_args; }
        let dir = dir.trim_matches('\\');
        let game = game.trim_matches('\\');
        let tmp = game.replace("/", "\\");

        let full_path = Path::new(dir).join(&tmp);
        let full_path_str = full_path.to_str().unwrap().replace("/", "\\");

        let command = format!("\"{}\" {}", full_path_str, args);

        let mut cmd = Command::new("cmd");
        cmd.arg("/C").arg("start").arg("/b").arg("");
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

        let spawned = cmd.spawn();
        if spawned.is_ok() {
            let process = spawned?;
            load_xxmi(install.clone(), gs.xxmi_path, exe.clone());
            load_fps_unlock(install.clone(), String::new());
            write_log(Path::new(&dir).to_path_buf(), process, "game.log".parse().unwrap());
            true
        } else {
            false
        }
    } else {
        // We assume user knows what he/she is doing so we just execute command that is configured without any checks
        let c = install.launch_command.clone();
        let args;
        let mut command = format!("\"{c}\"");

        if !install.launch_args.is_empty() {
            args = &install.launch_args;
            command = format!("\"{c}\" {args}");
        }

        let mut cmd = Command::new("cmd");
        cmd.arg("/C").arg("start").arg("/b").arg("");
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

        let spawned = cmd.spawn();
        if spawned.is_ok() {
            let process = spawned?;
            load_xxmi(install.clone(), gs.xxmi_path, exe.clone());
            load_fps_unlock(install.clone(), String::new());
            write_log(Path::new(&dir).to_path_buf(), process, "game.log".parse().unwrap());
            true
        } else {
            false
        }
    };
    Ok(rslt)
}

#[cfg(target_os = "windows")]
fn load_xxmi(install: LauncherInstall, xxmi_path: String, game: String) {
    if install.use_xxmi {
        let xxmi_path = xxmi_path.trim_matches('\\');
        let mipath = get_mi_path_from_game(game.clone()).unwrap();
        let loader_path = Path::new(xxmi_path).join("3dmloader.exe");
        let loader_path_str = loader_path.to_str().unwrap().replace("/", "\\");
        let command = format!("{} {}", loader_path_str, mipath);

        let mut cmd = runas::Command::new("cmd");
        cmd.arg("/C").arg("start").arg("/b").arg("");
        cmd.arg(&command);
        cmd.gui(true).force_prompt(true);

        // NOTE: We need to elevate 3dmloader.exe because game on windows is actually elevated so logs are also useless
        let child = cmd.status();
        match child {
            Ok(_process) => {}
            Err(_e) => {}
        }
    }
}

#[cfg(target_os = "windows")]
fn load_fps_unlock(_install: LauncherInstall, _fpsunlock_path: String) {}

fn write_log(log_dir: PathBuf, child: Child, file: String) {
    let ld1 = Arc::new(Mutex::new(log_dir.clone()));
    let c1 = Arc::new(Mutex::new(child));
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

        child.wait().unwrap();
        if let Ok(mut file) = game_output.lock() { file.flush().unwrap(); }
        drop(game_output);
        if let Some(join) = stdout_join { join.join().map_err(|err| format!("Failed to join stdout reader thread: {err:?}")).unwrap().unwrap(); }
        if let Some(join) = stderr_join { join.join().map_err(|err| format!("Failed to join stderr reader thread: {err:?}")).unwrap().unwrap(); }
    });
}