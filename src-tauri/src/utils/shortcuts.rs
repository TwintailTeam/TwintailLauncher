use std::fs;
use std::io::{Write};
use std::path::{PathBuf};
use steam_shortcuts_util::{parse_shortcuts, shortcuts_to_bytes, Shortcut};

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
        if !config_dir.exists() { if let Err(_) = fs::create_dir_all(&config_dir) { break } }

        if shortcut_file.exists() {
            let reader = fs::read(&shortcut_file);
            match reader {
                Ok(data) => {
                    let mut shortcuts = parse_shortcuts(data.as_slice()).unwrap();
                    let shortcut_exists = shortcuts.clone().iter().filter(|shortcutt| shortcutt.app_name == appname).count() > 0;
                    if shortcut_exists { break }
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




