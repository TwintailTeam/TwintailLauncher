use std::fs;
use std::io::BufReader;
use std::path::{PathBuf};
use std::sync::{RwLock};
use git2::{Error, Repository};
use linked_hash_map::LinkedHashMap;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use crate::utils::db_manager::{create_manifest, create_repository, get_manifest_info_by_filename, get_repository_info_by_github_id};
use crate::utils::{generate_cuid};
use crate::utils::git_helpers::{do_fetch, do_merge};

pub fn setup_official_repository(app: &AppHandle, path: &PathBuf) {
    let url = "https://github.com/AndigenaTeam/game-manifests.git";

    let tmp = url.split("/").collect::<Vec<&str>>()[4];
    let user = url.split("/").collect::<Vec<&str>>()[3];
    let repo_name = tmp.split(".").collect::<Vec<&str>>()[0];

    let repo_path = path.join(format!("{}/{}", user, repo_name).as_str());
    let repo_manifest = repo_path.join("repository.json");

    if !path.exists() {
        return;
    } else if !repo_path.exists() {
        Repository::clone(url, &repo_path).unwrap();
        
        if repo_manifest.exists() {
            let rm = fs::File::open(&repo_manifest).unwrap();
            let reader = BufReader::new(rm);
            let rma: RepositoryManifest = serde_json::from_reader(reader).unwrap();

            // remove this shit from actual manifest clone as normal people do not need it
            fs::remove_dir_all(&repo_path.join("scripts")).unwrap();

            let repo_id = generate_cuid();
            create_repository(app, repo_id.clone(), format!("{user}/{repo_name}").as_str()).unwrap();

            for m in rma.manifests {
                let mf = fs::File::open(&repo_path.join(&m.as_str())).unwrap();
                let reader = BufReader::new(mf);
                let mi: GameManifest = serde_json::from_reader(reader).unwrap();

                let cuid = generate_cuid();
                create_manifest(app, cuid.clone(), repo_id.clone(), mi.display_name.as_str(), m.as_str(), true).unwrap();
            }

            ()
        }
    } else {
        #[cfg(debug_assertions)]
        { println!("Official repository is already cloned!"); }
        update_repositories(&repo_path).unwrap();
    }
}

pub fn clone_new_repository(app: &AppHandle, path: &PathBuf, url: String) -> Result<bool, Error> {
    let tmp = url.split("/").collect::<Vec<&str>>()[4];
    let user = url.split("/").collect::<Vec<&str>>()[3];
    let repo_name = tmp.split(".").collect::<Vec<&str>>()[0];

    let repo_path = path.join(format!("{}/{}", user, repo_name).as_str());
    let repo_manifest = repo_path.join("repository.json");

    if !path.exists() {
        Ok(false)
    } else if !repo_path.exists() {
       let repo = Repository::clone(url.as_str(), &repo_path);

        if repo_manifest.exists() && repo.is_ok() {
            let rm = fs::File::open(&repo_manifest).unwrap();
            let reader = BufReader::new(rm);
            let rma: RepositoryManifest = serde_json::from_reader(reader).unwrap();

            let repo_id = generate_cuid();
            create_repository(app, repo_id.clone(), format!("{user}/{repo_name}").as_str()).unwrap();

            for m in rma.manifests {
                let mf = fs::File::open(&repo_path.join(&m.as_str())).unwrap();
                let reader = BufReader::new(mf);
                let mi: GameManifest = serde_json::from_reader(reader).unwrap();

                let cuid = generate_cuid();
                create_manifest(app, cuid.clone(), repo_id.clone(), mi.clone().display_name.as_str(), m.clone().as_str(), false).unwrap();
            }

            Ok(true)

        } else {
            #[cfg(debug_assertions)]
            { println!("Cannot clone repository! Not a valid repository?"); }

            Ok(false)
        }
    } else {
        #[cfg(debug_assertions)]
        { println!("Target repository already exists!"); }
        update_repositories(&repo_path)?;

        Ok(false)
    }
}

pub fn update_repositories(path: &PathBuf) -> Result<bool, Error> {
    let repo = Repository::open(&path);

    if repo.is_ok() && path.exists() {
        let r = repo?;
        let mut remote = r.find_remote("origin")?;
        let fetch_commit = do_fetch(&r, &["main"], &mut remote)?;
        do_merge(&r, "main", fetch_commit)?;

        #[cfg(debug_assertions)]
        { println!("Successfully updated repositories!"); }
        Ok(true)
    } else {
        #[cfg(debug_assertions)]
        { println!("Failed to fetch repository updates!"); }
        Ok(false)
    }
}

// === MANIFESTS ===

pub fn load_manifests(app: &AppHandle) {
        let data_path = app.path().app_data_dir().unwrap();
        let manifets_path = data_path.join("manifests");

        if !manifets_path.exists() {
            fs::create_dir_all(&manifets_path).unwrap();
        } else {
            for d in fs::read_dir(&manifets_path).unwrap() {
                let p = d.unwrap().path();

                if p.is_dir() {
                    for pp in fs::read_dir(p).unwrap() {
                        let p = pp.unwrap().path();
                        #[cfg(debug_assertions)]
                        { println!("Loading manifests from: {}", p.display()); }
                        let repo_manifest = p.join("repository.json");

                        if repo_manifest.exists() {
                            let rm = fs::File::open(&repo_manifest).unwrap();
                            let reader = BufReader::new(rm);
                            let rma: RepositoryManifest = serde_json::from_reader(reader).unwrap();

                            let ml = ManifestLoader::default();
                            let mut tmp = ml.0.write().unwrap();

                            for m in rma.manifests {
                                let mf = fs::File::open(&p.join(&m.as_str())).unwrap();
                                let reader = BufReader::new(mf);
                                let mi: GameManifest = serde_json::from_reader(reader).unwrap();

                                tmp.insert(m.clone(), mi.clone());
                                update_manifest_table(&app, m.clone(), mi.display_name.clone().as_str(), p.clone());

                                #[cfg(debug_assertions)]
                                { println!("Loaded manifest {}", mi.clone().display_name.as_str()); }
                            }

                            drop(tmp);
                            app.manage(ml);

                        } else {
                            #[cfg(debug_assertions)]
                            { println!("Failed to load manifests from {}! Not a valid KeqingLauncher repository?", p.display()); }
                        }
                    }
                }
            }
        }
    }

fn update_manifest_table(app: &AppHandle, filename: String, display_name: &str, path: PathBuf) {
    let dbm = get_manifest_info_by_filename(&app, filename.clone());
    if dbm.is_none() {
        let user = path.parent().unwrap().components().last().unwrap().as_os_str().to_str().unwrap();
        let repo_name = path.components().last().unwrap().as_os_str().to_str().unwrap();

        let dbr = get_repository_info_by_github_id(&app, format!("{user}/{repo_name}"));
        if dbr.is_some() {
            let dbrr = dbr.unwrap();
            let cuid = generate_cuid();
            create_manifest(&app, cuid, dbrr.id, display_name, filename.as_str(), true).unwrap();
        }
    }
}

pub fn get_manifests(app: &AppHandle) -> LinkedHashMap<String, GameManifest> {
    app.state::<ManifestLoader>().0.read().unwrap().clone()
}

pub fn get_manifest(app: &AppHandle, filename: &String) -> Option<GameManifest> {
    let loader = app.state::<ManifestLoader>().0.read().unwrap().clone();

    if loader.contains_key(filename) {
        let content = loader.get(filename).unwrap();
        Some(content.clone())
    } else {
        None
    }
}

// === STRUCTS ===

#[derive(Default)]
pub struct ManifestLoader(pub RwLock<LinkedHashMap<String, GameManifest>>);

#[derive(Serialize, Deserialize, Debug)]
struct RepositoryManifest {
    name: String,
    description: String,
    maintainers: Vec<String>,
    manifests: Vec<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LauncherRepository {
    pub id: String,
    pub github_id: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LauncherManifest {
    pub id: String,
    pub repository_id: String,
    pub display_name: String,
    pub filename: String,
    pub enabled: bool
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LauncherInstall {
    pub id: String,
    pub manifest_id: String,
    pub version: String,
    pub name: String,
    pub directory: String,
    pub runner: String,
    pub dxvk: String,
    pub game_icon: String,
    pub game_background: String,
    pub ignore_updates: bool,
    pub skip_hash_check: bool,
    pub use_jadeite: bool,
    pub use_xxmi: bool,
    pub use_fps_unlock: bool,
    pub env_vars: String,
    pub pre_launch_command: String,
    pub launch_command: String
}

// === MANIFESTS ===

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameManifest {
    pub version: i32,
    pub display_name: String,
    pub biz: String,
    pub latest_version: String,
    pub game_versions: Vec<GameVersion>,
    pub telemetry_hosts: Vec<String>,
    pub paths: GamePaths,
    pub assets: VersionAssets,
    pub extra: GameExtras
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameVersion {
    pub metadata: VersionMetadata,
    pub assets: VersionAssets,
    pub game: VersionGameFiles,
    pub audio: VersionAudioFiles
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GamePaths {
    pub exe_filename: String,
    pub installation_dir: String,
    pub screenshot_dir: String,
    pub screenshot_dir_relative_to: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VersionMetadata {
    pub versioned_name: String,
    pub version: String,
    pub game_hash: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VersionAssets {
    pub game_icon: String,
    pub game_background: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VersionGameFiles {
    pub full: Vec<FullGameFile>,
    pub diff: Vec<DiffGameFile>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FullGameFile {
    pub file_url: String,
    pub compressed_size: String,
    pub decompressed_size: String,
    pub file_hash: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DiffGameFile {
    pub file_url: String,
    pub compressed_size: String,
    pub decompressed_size: String,
    pub file_hash: String,
    pub diff_type: String,
    pub original_version: String,
    pub delete_files: Vec<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VersionAudioFiles {
    pub full: Vec<FullAudioFile>,
    pub diff: Vec<DiffAudioFile>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FullAudioFile {
    pub file_url: String,
    pub compressed_size: String,
    pub decompressed_size: String,
    pub file_hash: String,
    pub language: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DiffAudioFile {
    pub file_url: String,
    pub compressed_size: String,
    pub decompressed_size: String,
    pub file_hash: String,
    pub diff_type: String,
    pub original_version: String,
    pub language: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GamePreload {
    pub metadata: Option<VersionMetadata>,
    pub game: Option<VersionGameFiles>,
    pub audio: Option<VersionAudioFiles>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameExtras {
    pub preload: Option<GamePreload>,
}