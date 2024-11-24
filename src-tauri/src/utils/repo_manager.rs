use std::collections::HashMap;
use std::fs;
use std::io::BufReader;
use std::path::{PathBuf};
use std::sync::Mutex;
use git2::{Error, Repository};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use crate::utils::db_manager::{create_manifest, create_repository};
use crate::utils::generate_cuid;

pub async fn setup_official_repository(app: &AppHandle, path: &PathBuf) {
    let url = "https://github.com/TeamKeqing/launcher-manifests.git";

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
            //fs::remove_dir_all(&repo_path.join(".idea")).unwrap();
            //fs::remove_dir_all(&repo_path.join(".vscode")).unwrap();

            let repo_id = generate_cuid();

            create_repository(app, repo_id.clone(), format!("{user}/{repo_name}").as_str()).await.unwrap();

            for m in rma.manifests {
                async {
                    let mf = fs::File::open(&repo_path.join(&m.as_str())).unwrap();
                    let reader = BufReader::new(mf);
                    let mi: GameManifest = serde_json::from_reader(reader).unwrap();

                    let cuid = generate_cuid();
                    create_manifest(app, cuid.clone(), repo_id.clone(), mi.display_name.as_str(), m.as_str(), true).await.unwrap(); // enable all default manifests?? make behavior for no enabled manifests
                }.await
            }

            ()

        }
    } else {
        #[cfg(debug_assertions)]
        { println!("Official repository is already cloned!"); }
    }
}

pub async fn clone_new_repository(app: &AppHandle, path: &PathBuf, url: String) -> Result<bool, Error> {

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

            create_repository(app, repo_id.clone(), format!("{user}/{repo_name}").as_str()).await.unwrap();

            //let mut curmanifets = app.state::<ManifestLoader>().0.lock().unwrap();

            for m in rma.manifests {
                async {
                    let mf = fs::File::open(&repo_path.join(&m.as_str())).unwrap();
                    let reader = BufReader::new(mf);
                    let mi: GameManifest = serde_json::from_reader(reader).unwrap();

                    let cuid = generate_cuid();
                    create_manifest(app, cuid.clone(), repo_id.clone(), mi.clone().display_name.as_str(), m.clone().as_str(), false).await.unwrap();

                    /*if !curmanifets.contains_key(&m) {
                        curmanifets.insert(m.clone(), mi.clone());
                    }*/
                }.await
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

                            for m in rma.manifests {
                                let mf = fs::File::open(&p.join(&m.as_str())).unwrap();
                                let reader = BufReader::new(mf);
                                let mi: GameManifest = serde_json::from_reader(reader).unwrap();

                                let ml = ManifestLoader::default();
                                let mut tmp = ml.0.lock().unwrap();

                                tmp.insert(m, mi.clone());

                                drop(tmp);

                                app.manage(ml);

                                #[cfg(debug_assertions)]
                                { println!("Loaded manifest {}", mi.clone().display_name.as_str()); }
                            }
                        } else {
                            #[cfg(debug_assertions)]
                            { println!("Failed to load manifests from {}! Not a valid KeqingLauncher repository?", p.display()); }
                        }
                    }
                }
            }
        }
    }

pub fn get_manifests(app: &AppHandle) -> HashMap<String, GameManifest> {
    app.state::<ManifestLoader>().0.lock().unwrap().clone()
}

pub fn get_manifest(app: &AppHandle, filename: String) -> Option<GameManifest> {
    let loader = app.state::<ManifestLoader>().0.lock().unwrap().clone();

    if loader.contains_key(&filename) {
        Some(loader.get(&filename).unwrap().clone())
    } else {
        None
    }
}

// === STRUCTS ===

#[derive(Debug, Default)]
pub struct ManifestLoader(pub Mutex<HashMap<String, GameManifest>>);

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
    pub dxvk: String
}

// === MANIFESTS ===

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameManifest {
    pub version: i32,
    pub display_name: String,
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
    pub game_logo: String,
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
    pub original_version: String
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