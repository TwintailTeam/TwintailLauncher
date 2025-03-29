use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use tauri::{AppHandle, Error};
use crate::commands::settings::GlobalSettings;
use crate::utils::repo_manager::{get_compatibility, GameManifest, LauncherInstall};
use crate::utils::runner_from_runner_version;

#[cfg(target_os = "linux")]
pub fn launch(app: &AppHandle, install: LauncherInstall, gm: GameManifest, gs: GlobalSettings) -> Result<bool, Error> {
    let rm = get_compatibility(&app, &runner_from_runner_version(install.runner_version.clone()).unwrap()).unwrap();

    let dir = install.directory.clone();
    let prefix = install.runner_prefix.clone();
    let runner = install.runner_path.clone();
    let game = gm.paths.exe_filename.clone();
    let exe = gm.paths.exe_filename.clone().split('/').last().unwrap().to_string();

    let pre_launch = install.pre_launch_command.clone();
    let wine64 = rm.paths.wine64.clone();

    if !pre_launch.is_empty() {
        let command = format!("{runner}/{wine64} {pre_launch}");

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
            spawned?;
        }
    }

    let rslt = if install.launch_command.is_empty() {
        let mut args = "";
        if !install.launch_args.is_empty() {
            args = &install.launch_args;
        }

        let mut command = format!("{runner}/{wine64} {dir}/{game} {args}");

        if install.use_jadeite {
            let jadeite_path = gs.jadeite_path.clone();
            command = format!("{runner}/{wine64} z:\\{jadeite_path}/jadeite.exe z:\\{dir}/{game} -- {args}");
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
            spawned?;
            load_xxmi(install.clone(), prefix.clone(), gs.xxmi_path, runner.clone(), wine64.clone(), exe.clone());
            load_fps_unlock(install, prefix, gs.fps_unlock_path, runner, wine64);
            true
        } else {
            false
        }
    } else {
        // We assume user knows what he/she is doing so we just execute command that is configured without any checks
        let c = install.launch_command.clone();
        let args;
        let mut command = format!("{runner}/{wine64} {c}");

        if !install.launch_args.is_empty() {
            args = &install.launch_args;
            command = format!("{runner}/{wine64} {c} {args}");
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
            spawned?;
            load_xxmi(install.clone(), prefix.clone(), gs.xxmi_path, runner.clone(), wine64.clone(), exe.clone());
            load_fps_unlock(install, prefix, gs.fps_unlock_path, runner, wine64);
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
        std::thread::sleep(std::time::Duration::from_secs(3));
        println!("injecting xxmi loader...");
        let xxmi_path = xxmi_path.clone();
        let command = format!("{runner}/{wine64} z:\\{xxmi_path}/injector.exe -n {game} z:\\{xxmi_path}/3dmloader.dll");

        let mut cmd = Command::new("bash");
        cmd.arg("-c");
        cmd.arg(&command);

        cmd.env("WINEARCH","win64");
        cmd.env("WINEPREFIX", prefix.clone());

        //cmd.stdout(Stdio::piped());
        //cmd.stderr(Stdio::piped());
        cmd.current_dir(xxmi_path.clone());
        cmd.process_group(0);
        cmd.spawn().unwrap();
    }
}

#[cfg(target_os = "linux")]
fn load_fps_unlock(install: LauncherInstall, prefix: String, fpsunlock_path: String, runner: String, wine64: String) {
    // Delay for a second so game process is loaded, Test how delay behaves on other PCs
    std::thread::sleep(std::time::Duration::from_secs(1));
    if install.use_fps_unlock {
        let fpsunlock_path = fpsunlock_path.clone();
        let fpsv = install.fps_value;
        let command = format!("{runner}/{wine64} z:\\{fpsunlock_path}/fpsunlock.exe {fpsv} 3000");

        let mut cmd = Command::new("bash");
        cmd.arg("-c");
        cmd.arg(&command);

        cmd.env("WINEARCH","win64");
        cmd.env("WINEPREFIX", prefix.clone());

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.current_dir(fpsunlock_path.clone());
        cmd.process_group(0);
        cmd.spawn().unwrap();
    }
}

#[cfg(target_os = "windows")]
pub fn launch() {

}

#[cfg(target_os = "windows")]
fn load_xxmi(install: LauncherInstall, prefix: String, xxmi_path: String, runner: String, wine64: String, game: String) {

}

#[cfg(target_os = "windows")]
fn load_fps_unlock(install: LauncherInstall, prefix: String, fpsunlock_path: String, runner: String, wine64: String) {

}