use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::process::{Child, Command, Stdio};
use tauri::{AppHandle, Error};
use crate::commands::settings::GlobalSettings;
use crate::utils::repo_manager::{GameManifest, LauncherInstall};

#[cfg(target_os = "linux")]
use std::os::unix::process::CommandExt;
use crate::utils::{wait_for_process};
#[cfg(target_os = "linux")]
use crate::utils::runner_from_runner_version;
#[cfg(target_os = "linux")]
use crate::utils::repo_manager::{get_compatibility};

#[cfg(target_os = "linux")]
pub fn launch(app: &AppHandle, install: LauncherInstall, gm: GameManifest, gs: GlobalSettings) -> Result<bool, Error> {
    let rm = get_compatibility(&app, &runner_from_runner_version(install.runner_version.clone()).unwrap()).unwrap();

    let dir = install.directory.clone();
    let prefix = install.runner_prefix.clone();
    let runner = install.runner_path.clone();
    let game = gm.paths.exe_filename.clone();
    let exe = gm.paths.exe_filename.clone().split('/').last().unwrap().to_string();

    let pre_launch = install.pre_launch_command.clone();
    let wine64 = if rm.paths.wine64.is_empty() {
        rm.paths.wine32
    } else {
        rm.paths.wine64
    };

    if !pre_launch.is_empty() {
        let command = format!("'{runner}/{wine64}' '{pre_launch}'");

        let mut cmd = Command::new("bash");
        cmd.arg("-c");
        cmd.arg(&command);

        cmd.env("WINEARCH","win64");
        cmd.env("WINEPREFIX", prefix.clone());

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.current_dir(dir.clone());
        cmd.process_group(0);

        let spawned = cmd.spawn();
        if spawned.is_ok() {
            let process = spawned?;
            write_log(Path::new(&dir.clone()).to_path_buf(), process, "game.log".parse().unwrap());
        }
    }

    let rslt = if install.launch_command.is_empty() {
        let mut args = "";
        if !install.launch_args.is_empty() {
            args = &install.launch_args;
        }

        let mut command = format!("'{runner}/{wine64}' '{dir}/{game}' {args}");

        if install.use_jadeite {
            let jadeite_path = gs.jadeite_path.clone();
            command = format!("'{runner}/{wine64}' '{jadeite_path}/jadeite.exe' '{dir}/{game}' -- {args}");
        }

        let mut cmd = Command::new("bash");
        cmd.arg("-c");
        cmd.arg(&command);

        cmd.env("WINEARCH","win64");
        cmd.env("WINEPREFIX", prefix.clone());

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.current_dir(dir.clone());
        cmd.process_group(0); // Start as detached process so killing launcher does not kill the game

        if !install.env_vars.is_empty() {
            let envs = install.env_vars.clone();
            let splitted = envs.split(";").collect::<Vec<&str>>();
            splitted.iter().for_each(|env| {
                if !env.is_empty() {
                    let tmp = env.split("=").collect::<Vec<&str>>();
                    let k = tmp[0];
                    let v = tmp[1];
                    cmd.env(k, v);
                }
            });
        }

        let spawned = cmd.spawn();
        if spawned.is_ok() {
            let process = spawned?;
            load_xxmi(install.clone(), prefix.clone(), gs.xxmi_path, runner.clone(), wine64.clone(), exe.clone());
            load_fps_unlock(install, prefix, gs.fps_unlock_path, runner, wine64, exe.clone());
            write_log(Path::new(&dir.clone()).to_path_buf(), process, "game.log".parse().unwrap());
            true
        } else {
            false
        }
    } else {
        // We assume user knows what he/she is doing so we just execute command that is configured without any checks
        let c = install.launch_command.clone();
        let args;
        let mut command = format!("'{runner}/{wine64}' '{c}'");

        if !install.launch_args.is_empty() {
            args = &install.launch_args;
            command = format!("'{runner}/{wine64}' '{c}' {args}");
        }

        let mut cmd = Command::new("bash");
        cmd.arg("-c");
        cmd.arg(&command);

        cmd.env("WINEARCH","win64");
        cmd.env("WINEPREFIX", prefix.clone());

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.current_dir(dir.clone());
        cmd.process_group(0);

        if !install.env_vars.is_empty() {
            let envs = install.env_vars.clone();
            let splitted = envs.split(";").collect::<Vec<&str>>();
            splitted.iter().for_each(|env| {
                if !env.is_empty() {
                    let tmp = env.split("=").collect::<Vec<&str>>();
                    let k = tmp[0];
                    let v = tmp[1];
                    cmd.env(k, v);
                }
            });
        }

        let spawned = cmd.spawn();
        if spawned.is_ok() {
            let process = spawned?;
            load_xxmi(install.clone(), prefix.clone(), gs.xxmi_path, runner.clone(), wine64.clone(), exe.clone());
            load_fps_unlock(install, prefix, gs.fps_unlock_path, runner, wine64, exe.clone());
            write_log(Path::new(&dir.clone()).to_path_buf(), process, "game.log".parse().unwrap());
            true
        } else {
            false
        }
    };

    Ok(rslt)
}

#[cfg(target_os = "linux")]
fn load_xxmi(install: LauncherInstall, prefix: String, xxmi_path: String, runner: String, wine64: String, game: String) {
    if install.use_xxmi {
        wait_for_process(game.as_str(), || {
            let xxmi_path = xxmi_path.clone();
            let command = format!("'{runner}/{wine64}' 'z:\\{xxmi_path}/3dmloader.exe'");

            let mut cmd = Command::new("bash");
            cmd.arg("-c");
            cmd.arg(&command);

            cmd.env("WINEARCH","win64");
            cmd.env("WINEPREFIX", prefix.clone());

            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());
            cmd.current_dir(xxmi_path.clone());
            cmd.process_group(0);

            let spawn = cmd.spawn();
            if spawn.is_ok() {
                let process = spawn.unwrap();
                write_log(Path::new(&xxmi_path.clone()).to_path_buf(), process, "xxmi.log".parse().unwrap());
            }
            true
        });
    }
}

#[cfg(target_os = "linux")]
fn load_fps_unlock(install: LauncherInstall, prefix: String, fpsunlock_path: String, runner: String, wine64: String, game: String) {
    if install.use_fps_unlock {
        wait_for_process(game.as_str(), || {
            let fpsunlock_path = fpsunlock_path.clone();
            let fpsv = install.fps_value.clone();
            let command = format!("'{runner}/{wine64}' 'z:\\{fpsunlock_path}/fpsunlock.exe' {fpsv} 3000");

            let mut cmd = Command::new("bash");
            cmd.arg("-c");
            cmd.arg(&command);

            cmd.env("WINEARCH","win64");
            cmd.env("WINEPREFIX", prefix.clone());

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
        });
    }
}

#[cfg(target_os = "linux")]
fn write_log(log_dir: PathBuf, child: Child, file: String) {
    let ld1 = Arc::new(Mutex::new(log_dir.clone()));
    let c1 = Arc::new(Mutex::new(child));
    std::thread::spawn(move || {
        let log_dir = ld1.lock().unwrap().clone();
        let mut child = c1.lock().unwrap();
        let log_file_size = 8 * 1024 * 1024; // 8 MiB

        // Credit to certain anime team for this too lol pointless to write from scratch...
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
                    if read == 0 {
                        break;
                    }

                    let Ok(mut game_output) = game_output.lock() else {
                        break;
                    };

                    for line in buf[..read].split(|c| c == &b'\n') {
                        game_output.write_all(b"    [stdout] ")?;
                        game_output.write_all(line)?;
                        game_output.write_all(b"\n")?;

                        written.fetch_add(line.len() + 14, Ordering::Relaxed);
                    }

                    if written.load(Ordering::Relaxed) > log_file_size {
                        break;
                    }
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
                    if read == 0 {
                        break;
                    }

                    let Ok(mut game_output) = game_output.lock() else {
                        break;
                    };

                    for line in buf[..read].split(|c| c == &b'\n') {
                        game_output.write_all(b"[!] [stderr] ")?;
                        game_output.write_all(line)?;
                        game_output.write_all(b"\n")?;

                        written.fetch_add(line.len() + 14, Ordering::Relaxed);
                    }

                    if written.load(Ordering::Relaxed) > log_file_size {
                        break;
                    }
                }

                Ok(())
            }));
        }

        child.wait().unwrap();
        if let Ok(mut file) = game_output.lock() {
            file.flush().unwrap();
        }

        drop(game_output);

        if let Some(join) = stdout_join {
            join.join().map_err(|err| format!("Failed to join stdout reader thread: {err:?}")).unwrap().unwrap();
        }

        if let Some(join) = stderr_join {
            join.join().map_err(|err| format!("Failed to join stderr reader thread: {err:?}")).unwrap().unwrap();
        }
    });
}

#[cfg(target_os = "windows")]
pub fn launch(_app: &AppHandle, _install: LauncherInstall, _gm: GameManifest, _gs: GlobalSettings) -> Result<bool, Error> {
    Ok(false)
}

#[cfg(target_os = "windows")]
fn load_xxmi(_install: LauncherInstall, _prefix: String, _xxmi_path: String, _runner: String, _wine64: String, _game: String) {

}

#[cfg(target_os = "windows")]
fn load_fps_unlock(_install: LauncherInstall, _prefix: String, _fpsunlock_path: String, _runner: String, _wine64: String) {

}

#[cfg(target_os = "windows")]
fn write_log(_log_dir: PathBuf, _child: Child, _file: String) {

}