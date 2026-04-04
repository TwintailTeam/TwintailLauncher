use std::fs;
use std::io::{Write};
use std::path::{PathBuf};
use steam_shortcuts_util::{parse_shortcuts, shortcuts_to_bytes, Shortcut};
#[cfg(target_os = "linux")]
use tauri::{AppHandle, Manager};

#[cfg(target_os = "linux")]
pub fn resolve_normal_steam_userdata(home_dir: PathBuf) -> PathBuf {
    let steam_symlink = home_dir.join(".steam/steam");
    let steam_base = if steam_symlink.exists() { fs::canonicalize(&steam_symlink).unwrap_or_else(|_| home_dir.join(".local/share/Steam")) } else { home_dir.join(".local/share/Steam") };
    steam_base.join("userdata")
}

#[allow(dead_code)]
fn check_steam_user_data_dir(steam_userdata_dir: PathBuf) -> Vec<String> {
    let ignore_folders = ["0", "ac", "anonymous"];
    if !steam_userdata_dir.exists() { return vec![]; }
    let mut folders: Vec<String> = Vec::new();

    if let Ok(entries) = fs::read_dir(steam_userdata_dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_dir() {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    if !ignore_folders.contains(&name.as_str()) { folders.push(name); }
                }
            }
        }
    }
    folders
}

#[allow(dead_code)]
pub fn add_steam_shortcut(file: PathBuf, appname: &str, shortcut: Shortcut) -> bool {
    let dir = check_steam_user_data_dir(file.clone());
    let mut result = false;
    for u in dir {
        let config_dir = file.join(u).join("config");
        let shortcut_file = config_dir.join("shortcuts.vdf");
        if !config_dir.exists() { if let Err(_) = fs::create_dir_all(&config_dir) { continue } }

        if shortcut_file.exists() {
            let reader = fs::read(&shortcut_file);
            match reader {
                Ok(data) => {
                    let mut shortcuts = parse_shortcuts(data.as_slice()).unwrap();
                    let shortcut_exists = shortcuts.clone().iter().filter(|shortcutt| shortcutt.app_name == appname).count() > 0;
                    if shortcut_exists { continue }
                    shortcuts.push(shortcut.clone());
                    let parsed = shortcuts_to_bytes(&shortcuts);
                    let ff = fs::write(&shortcut_file, &parsed);
                    match ff {
                        Ok(_) => result = true,
                        Err(_) => continue,
                    }
                }
                Err(_) => continue,
            }
        } else {
            let mut data = Vec::new();
            data.push(shortcut.clone());
            let d = shortcuts_to_bytes(&data);
            let ff = fs::write(&shortcut_file, &d);
            match ff {
                Ok(_) => result = true,
                Err(_) => continue,
            }
        }
    }
    result
}

#[allow(dead_code)]
pub fn remove_steam_shortcut(file: PathBuf, appname: &str) -> bool {
    let dir = check_steam_user_data_dir(file.clone());
    let mut result = false;
    for u in dir {
        let config_dir = file.join(u).join("config");
        let shortcut_file = config_dir.join("shortcuts.vdf");

        if !config_dir.exists() || !shortcut_file.exists() { continue }

        let reader = fs::read(&shortcut_file);
        match reader {
            Ok(data) => {
                let mut shortcuts = parse_shortcuts(data.as_slice()).unwrap();
                let shortcut_exists = shortcuts.clone().iter().filter(|shortcutt| shortcutt.app_name == appname).count() > 0;
                if !shortcut_exists { continue }
                let index= shortcuts.iter().position(|s| s.app_name == appname).unwrap();

                shortcuts.remove(index);
                let parsed = shortcuts_to_bytes(&shortcuts);
                let ff = fs::write(&shortcut_file, &parsed);
                match ff {
                    Ok(_) => result = true,
                    Err(_) => continue
                }
            }
            Err(_) => continue
        }
    }
    result
}

#[allow(dead_code)]
pub fn add_desktop_shortcut(file: PathBuf, content: String) -> bool {
    if file.exists() { return false; }
    let f = fs::File::create(file.clone());
    match f {
        Ok(mut ff) => {
            ff.write_all(content.as_bytes()).unwrap();
            true
        }
        Err(_) => { false }
    }
}

pub fn remove_desktop_shortcut(file: PathBuf) -> bool {
    if !file.exists() { return false; }
    let rem = fs::remove_file(file);
    match rem {
        Ok(_) => { true }
        Err(_) => { false }
    }
}

#[cfg(target_os = "linux")]
pub fn sync_desktop_shortcut(app: &AppHandle, install_id: String, new_name: String) {
    let install = crate::utils::db_manager::get_install_info_by_id(app, install_id).unwrap();
    if install.shortcut_path.is_empty() { return; }
    let base = app.path().home_dir().unwrap().join(".local/share/applications");
    update_desktop_shortcut(std::path::Path::new(&install.shortcut_path).to_path_buf(), new_name, base);
}

#[cfg(target_os = "linux")]
fn update_desktop_shortcut(old_desktop_path: PathBuf, new_name: String, base_dir: PathBuf) -> bool {
    if !old_desktop_path.exists() { return false; }
    let content = match fs::read_to_string(&old_desktop_path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let updated_content = {
        let lines: Vec<&str> = content.lines().collect();
        let updated_lines: Vec<String> = lines.iter().map(|line| { if line.starts_with("Name=") { format!("Name={}", new_name) } else { line.to_string() } }).collect();
        updated_lines.join("\n")
    };
    let new_desktop_path = base_dir.join(format!("{}.desktop", new_name));
    if old_desktop_path == new_desktop_path {
        return match fs::write(&old_desktop_path, updated_content) {
            Ok(_) => true,
            Err(_) => false,
        };
    }
    let write_result = match fs::File::create(&new_desktop_path) {
        Ok(mut f) => f.write_all(updated_content.as_bytes()).is_ok(),
        Err(_) => false,
    };
    if !write_result { return false; }
    fs::remove_file(&old_desktop_path).is_ok() && write_result
}
