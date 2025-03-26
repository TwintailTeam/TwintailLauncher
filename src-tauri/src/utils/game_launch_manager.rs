use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use tauri::{AppHandle, Error};
use crate::utils::repo_manager::{get_compatibility, GameManifest, LauncherInstall};
use crate::utils::runner_from_runner_version;

#[cfg(target_os = "linux")]
pub fn launch(app: &AppHandle, install: LauncherInstall, gm: GameManifest) -> Result<bool, Error> {
    let rm = get_compatibility(&app, &runner_from_runner_version(install.runner_version).unwrap()).unwrap();

    let dir = install.directory;
    let prefix = install.runner_prefix;
    let runner = install.runner_path;
    let game = gm.paths.exe_filename;

    let pre_launch = install.pre_launch_command;
    let wine64 = rm.paths.wine64;

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
        cmd.process_group(0); // Start as detached process so killing launcher does not kill the application (Make parent of game process???)

        let spawned = cmd.spawn();
        if spawned.is_ok() {
            spawned?;
        }
    }

    if install.use_fps_unlock {
        println!("Unlocking fps");
    }

    if install.use_jadeite {
        println!("launching with jadeite");
    }

    let rslt = if install.launch_command.is_empty() {
        let command = format!("{runner}/{wine64} {dir}/{game}");

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
            true
        } else {
            false
        }
    } else {
        let mut cmd = Command::new("bash");
        cmd.arg("-c");
        cmd.arg(&install.launch_command);

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
            true
        } else {
            false
        }
    };

    if install.use_xxmi {
        println!("injecting xxmi loader...");
    }

    Ok(rslt)
}

#[cfg(target_os = "windows")]
pub fn launch() {
    println!("launch");
}