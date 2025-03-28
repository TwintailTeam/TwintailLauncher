use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use tauri::{AppHandle, Error, Manager};
use crate::commands::settings::GlobalSettings;
use crate::utils::repo_manager::{get_compatibility, GameManifest, LauncherInstall};
use crate::utils::runner_from_runner_version;

#[cfg(target_os = "linux")]
pub fn launch(app: &AppHandle, install: LauncherInstall, gm: GameManifest, gs: GlobalSettings) -> Result<bool, Error> {
    let rm = get_compatibility(&app, &runner_from_runner_version(install.runner_version.clone()).unwrap()).unwrap();

    let dir = install.directory.clone();
    let prefix = install.runner_prefix.clone();
    let runner = install.runner_path.clone();
    let game = gm.paths.exe_filename;

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

    if install.use_fps_unlock {
        println!("Unlocking fps");
    }

    let rslt = if install.launch_command.is_empty() {
        let mut command = format!("{runner}/{wine64} {dir}/{game}");

        if install.use_jadeite {
            let jadeite_path = gs.jadeite_path.clone();
            command = format!("{runner}/{wine64} '{jadeite_path}/jadeite.exe' 'z:\\{dir}/{game}' -- ");
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
            // delay injection of xxmi...
            std::thread::sleep(std::time::Duration::from_secs(2));
            load_xxmi(&app, install.clone(), prefix.clone(), gs.xxmi_path, dir.clone(), runner.clone(), wine64, game.clone());
            true
        } else {
            false
        }
    } else {
        // We assume user knows what he/she is doing so we just execute command that is configured without any checks (jadeite usage, etc...)
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
            // delay injection of xxmi...
            std::thread::sleep(std::time::Duration::from_secs(2));
            load_xxmi(&app, install.clone(), prefix.clone(), gs.xxmi_path, dir.clone(), runner.clone(), wine64, game.clone());
            true
        } else {
            false
        }
    };

    Ok(rslt)
}

fn load_xxmi(app: &AppHandle, install: LauncherInstall, prefix: String, xxmi_path: String, dir: String, runner: String, wine64: String, game: String) {
    if install.use_xxmi {
        println!("injecting xxmi loader...");
        let injector = app.path().app_data_dir().unwrap().join("extras").join("injector").as_os_str().to_str().unwrap().to_string();
        let xxmi_path = xxmi_path.clone();

        let command = format!("{runner}/{wine64} '{injector}/injector.exe' -n '{game}' 'z:\\{xxmi_path}/3dmloader.dll'");

        let mut cmd = Command::new("bash");
        cmd.arg("-c");
        cmd.arg(command);

        cmd.env("WINEARCH","win64");
        cmd.env("WINEPREFIX", prefix.clone());

        //cmd.stdout(Stdio::piped());
        //cmd.stderr(Stdio::piped());
        cmd.current_dir(dir.clone());
        cmd.process_group(0);
        cmd.spawn().unwrap();
    }
}

#[cfg(target_os = "windows")]
pub fn launch() {
    println!("launch");
}