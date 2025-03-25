use std::process::{Command, Stdio};
use tauri::{Error};
use crate::utils::repo_manager::{GameManifest, LauncherInstall};

#[cfg(target_os = "linux")]
pub fn launch(install: LauncherInstall, gm: GameManifest) -> Result<bool, Error> {
    let dir = install.directory;
    let prefix = install.runner_prefix;
    let runner = install.runner_path;
    let game = gm.paths.exe_filename;

    let pre_launch = install.pre_launch_command;

    if !pre_launch.is_empty() {
        // TODO: Make path to wine binary read from compatibility manifest...
        let command = format!("{runner}/bin/wine64 {pre_launch}");

        let mut cmd = Command::new("bash");
        cmd.arg("-c");
        cmd.arg(&command);

        cmd.env("WINEARCH","win64");
        cmd.env("WINEPREFIX", prefix.clone());

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let spawned = cmd.spawn();
        if spawned.is_ok() {
            spawned?;
        }
    }

    let rslt = if install.launch_command.is_empty() {
        let command = format!("{runner}/bin/wine64 {dir}/{game}");

        let mut cmd = Command::new("bash");
        cmd.arg("-c");
        cmd.arg(&command);

        cmd.env("WINEARCH","win64");
        cmd.env("WINEPREFIX", prefix.clone());

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

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

    Ok(rslt)
}

#[cfg(target_os = "windows")]
pub fn launch() {
    println!("launch");
}