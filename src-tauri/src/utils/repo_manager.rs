use std::fs;
use std::io::BufReader;
use std::path::{PathBuf};
use std::sync::{RwLock};
use crate::utils::LinkedHashMap;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use crate::utils::db_manager::{create_manifest,create_repository,delete_manifest_by_id,get_manifest_info_by_filename,get_manifests_by_repository_id,get_repositories,get_repository_info_by_github_id,update_manifest_enabled_by_id};
use crate::utils::{generate_cuid, models::{RepositoryManifest, RunnerManifest, GameManifest}, show_dialog};
use crate::utils::git_helpers::{do_fetch, do_merge};

#[cfg(target_os = "linux")]
use crate::utils::{run_async_command, runner_from_runner_version};
#[cfg(target_os = "linux")]
use std::path::Path;
#[cfg(target_os = "linux")]
use crate::utils::db_manager::{create_installed_runner, update_install_runner_location_by_id, update_install_runner_version_by_id, get_installs, get_installed_runner_info_by_version, update_installed_runner_is_installed_by_version};

fn manifest_branch() -> &'static str { if cfg!(debug_assertions) { "next" } else { "main" } }

fn clone_repo(url: &str, path: &PathBuf) -> Result<git2::Repository, git2::Error> {
    if cfg!(debug_assertions) { git2::build::RepoBuilder::new().branch("next").clone(url, path) } else { git2::Repository::clone(url, path) }
}

pub fn setup_official_repository(app: &AppHandle, path: &PathBuf) {
    let url = "https://github.com/TwintailTeam/game-manifests.git";

    let tmp = url.split("/").collect::<Vec<&str>>()[4];
    let user = url.split("/").collect::<Vec<&str>>()[3];
    let repo_name = tmp.split(".").collect::<Vec<&str>>()[0];

    let repo_path = path.join(format!("{}/{}", user, repo_name).as_str());
    let repo_manifest = repo_path.join("repository.json");

    if !path.exists() {
        return;
    } else if !repo_path.exists() {
        clone_repo(url, &repo_path).unwrap();

        if repo_manifest.exists() {
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
                create_manifest(app, cuid.clone(), repo_id.clone(), mi.display_name.as_str(), m.as_str(), true).unwrap();
            }
            ()
        }
    } else {
        log::debug!("Official game repository is already cloned!");
        #[cfg(debug_assertions)]
        { println!("Official game repository is already cloned!"); }
        let r = update_repositories(&repo_path);
        match r {
            Ok(_) => {}
            Err(e) => { show_dialog(app, "warning", "TwintailLauncher", format!("Failed to fetch update(s) for game manifest repository! {}", e.to_string()).as_str(), None); }
        }
    }
}

pub fn clone_new_repository(app: &AppHandle, path: &PathBuf, url: String) -> Result<bool, git2::Error> {
    let tmp = url.split("/").collect::<Vec<&str>>()[4];
    let user = url.split("/").collect::<Vec<&str>>()[3];
    let repo_name = tmp.split(".").collect::<Vec<&str>>()[0];

    let repo_path = path.join(format!("{}/{}", user, repo_name).as_str());
    let repo_manifest = repo_path.join("repository.json");

    if !path.exists() {
        Ok(false)
    } else if !repo_path.exists() {
       let repo = clone_repo(url.as_str(), &repo_path);

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
                create_manifest(app, cuid.clone(), repo_id.clone(), mi.clone().display_name.as_str(), m.clone().as_str(), true).unwrap();
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
        let r = update_repositories(&repo_path);
        match r {
            Ok(_) => {}
            Err(e) => { show_dialog(app, "warning", "TwintailLauncher", format!("Failed to fetch update(s) for one or multiple 3rd party repositories! {}", e.to_string()).as_str(), None); }
        }
        Ok(false)
    }
}

pub fn update_repositories(path: &PathBuf) -> Result<bool, git2::Error> {
    let repo = git2::Repository::open(&path);

    if repo.is_ok() && path.exists() {
        let r = repo?;
        let mut remote = r.find_remote("origin")?;
        let branch = manifest_branch();
        let fetch_commit = do_fetch(&r, &[branch], &mut remote)?;
        do_merge(&r, branch, fetch_commit)?;
        log::debug!("Successfully updated repositories!");
        #[cfg(debug_assertions)]
        { println!("Successfully updated repositories!"); }
        Ok(true)
    } else {
        log::debug!("Failed to fetch repository updates!");
        #[cfg(debug_assertions)]
        { println!("Failed to fetch repository updates!"); }
        Ok(false)
    }
}

#[cfg(target_os = "linux")]
pub fn setup_compatibility_repository(app: &AppHandle, path: &PathBuf) {
    let url = "https://github.com/TwintailTeam/runner-manifests.git";

    let tmp = url.split("/").collect::<Vec<&str>>()[4];
    let user = url.split("/").collect::<Vec<&str>>()[3];
    let repo_name = tmp.split(".").collect::<Vec<&str>>()[0];

    let repo_path = path.join(format!("{}/{}", user, repo_name).as_str());
    let repo_manifest = repo_path.join("repository.json");

    if !path.exists() {
        return;
    } else if !repo_path.exists() {
        clone_repo(url, &repo_path).unwrap();

        if repo_manifest.exists() {
            let rm = fs::File::open(&repo_manifest).unwrap();
            let reader = BufReader::new(rm);
            let rma: RepositoryManifest = serde_json::from_reader(reader).unwrap();

            let repo_id = generate_cuid();
            create_repository(app, repo_id.clone(), format!("{user}/{repo_name}").as_str()).unwrap();

            for m in rma.manifests {
                let mf = fs::File::open(&repo_path.join(&m.as_str()));
                match mf {
                    Ok(mm) => {
                        let reader = BufReader::new(mm);
                        let mi: RunnerManifest = serde_json::from_reader(reader).unwrap();
                        let cuid = generate_cuid();
                        create_manifest(app, cuid.clone(), repo_id.clone(), mi.display_name.as_str(), m.as_str(), true).unwrap();
                    }
                    Err(_) => {}
                }
            }
            ()
        }
    } else {
        log::debug!("Official compatibility repository is already cloned!");
        #[cfg(debug_assertions)]
        { println!("Official compatibility repository is already cloned!"); }
        let r = update_repositories(&repo_path);
        match r {
            Ok(_) => {}
            Err(e) => { show_dialog(app, "warning", "TwintailLauncher", format!("Failed to fetch update(s) for compatibility repository! {}", e.to_string()).as_str(), None); }
        }
    }
}

#[cfg(target_os = "windows")]
pub fn setup_compatibility_repository(_app: &AppHandle, _path: &PathBuf) {}

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
                        log::debug!("Loading manifests from: {}", p.display());
                        #[cfg(debug_assertions)]
                        { println!("Loading manifests from: {}", p.display()); }
                        let repo_manifest = p.join("repository.json");

                        if repo_manifest.exists() {
                            let rm = fs::File::open(&repo_manifest).unwrap();
                            let reader = BufReader::new(rm);
                            let rma: RepositoryManifest = serde_json::from_reader(reader).unwrap();

                            let ml = app.state::<ManifestLoaders>().clone();

                            let mut tmp = ml.game.0.write().unwrap();
                            #[cfg(target_os = "linux")]
                            let mut tmp1 = ml.runner.0.write().unwrap();

                            for m in rma.manifests {
                                let mp = p.join(&m.as_str());
                                if mp.exists() {
                                    let file = fs::File::open(&mp).unwrap();
                                    let reader = BufReader::new(file);
                                    let manifest_data = serde_json::from_reader(reader).unwrap();

                                    match manifest_data {
                                        ManifestData::Game(mi) => {
                                            tmp.insert(m.clone(), mi.clone());
                                            update_manifest_table(&app, m.clone(), mi.display_name.clone().as_str(), p.clone());
                                            log::debug!("Loaded game manifest {}", m.as_str());
                                            #[cfg(debug_assertions)]
                                            { println!("Loaded game manifest {}", m.as_str()); }
                                        }
                                        #[cfg(target_os = "linux")]
                                        ManifestData::Runner(ri) => {
                                            tmp1.insert(m.clone(), ri.clone());
                                            update_manifest_table(&app, m.clone(), ri.display_name.clone().as_str(), p.clone());
                                            log::debug!("Loaded compatibility manifest {}", m.as_str());
                                            #[cfg(debug_assertions)]
                                            { println!("Loaded compatibility manifest {}", m.as_str()); }
                                        }
                                        #[cfg(target_os = "windows")]
                                        ManifestData::Runner(_) => {}
                                    }
                                } else {
                                    // Delete manifests that no longer exist
                                    let dbm = get_manifest_info_by_filename(&app, m.clone());
                                    if dbm.is_some() {
                                        let ml = dbm.unwrap();
                                        #[cfg(target_os = "linux")]
                                        {
                                            let dbr = crate::utils::db_manager::get_repository_info_by_id(&app, ml.repository_id.clone());
                                            if dbr.is_some() {
                                                let dbrr = dbr.unwrap();
                                                if dbrr.github_id.contains("runner-manifests") {
                                                    let installs = get_installs(&app);
                                                    if installs.is_some() {
                                                        let install = installs.unwrap();
                                                        // Fallback installs that use deprecated runner
                                                        for i in install {
                                                            let ir = runner_from_runner_version(app, i.runner_version.clone()).unwrap_or_default();
                                                            if ir == m {
                                                                let file = fs::File::open(p.join("proton_cachyos.json")).unwrap();
                                                                let reader = BufReader::new(file);
                                                                let manifest_data = serde_json::from_reader(reader).unwrap();
                                                                match manifest_data {
                                                                    ManifestData::Game(_mi) => {}
                                                                    #[cfg(target_os = "linux")]
                                                                    ManifestData::Runner(ri) => {
                                                                        let first = ri.versions.first().unwrap();
                                                                        let np = i.runner_path.replace(i.runner_version.as_str(), first.version.as_str());
                                                                        let pp = Path::new(&np).to_path_buf();
                                                                        let installedr = get_installed_runner_info_by_version(&app, first.version.clone());
                                                                        if installedr.is_none() { create_installed_runner(&app, first.version.clone(), true, np.clone()).unwrap(); } else { update_installed_runner_is_installed_by_version(&app, first.version.clone(), true); }
                                                                        let mut dl_url = first.url.clone();
                                                                        if let Some(ref urls) = first.urls { #[cfg(target_arch = "x86_64")] { dl_url = urls.x86_64.clone(); } #[cfg(target_arch = "aarch64")] { dl_url = if urls.aarch64.is_empty() { first.url.clone() } else { urls.aarch64.clone() }; } }
                                                                        if !pp.exists() {
                                                                            fs::create_dir_all(&pp).unwrap();
                                                                            run_async_command(async { fischl::compat::download_runner(dl_url.clone(), pp.to_str().unwrap().to_string(),true, move |_current, _total, _net, _disk| {}, move |_current, _total| {}).await });
                                                                        } else {
                                                                            run_async_command(async { fischl::compat::download_runner(dl_url, pp.to_str().unwrap().to_string(),true, move |_current, _total, _net, _disk| {}, move |_current, _total| {}).await });
                                                                        }
                                                                        update_install_runner_location_by_id(&app, i.id.clone(), np.clone());
                                                                        update_install_runner_version_by_id(&app, i.id, first.version.clone());
                                                                    }
                                                                    #[cfg(target_os = "windows")]
                                                                    ManifestData::Runner(_) => {}
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        delete_manifest_by_id(app, ml.id).unwrap();
                                    } // cleanup end
                                }
                            }
                            drop(tmp);
                            #[cfg(target_os = "linux")]
                            drop(tmp1);
                        } else {
                            log::debug!("Failed to load manifests from {}! Not a valid KeqingLauncher repository?", p.display());
                            #[cfg(debug_assertions)]
                            { println!("Failed to load manifests from {}! Not a valid KeqingLauncher repository?", p.display()); }
                        }
                    }
                }
            }
        }
        cleanup_unloaded_manifests(app);
    }

fn cleanup_unloaded_manifests(app: &AppHandle) {
    let game_loader = app.state::<ManifestLoaders>().game.0.read().unwrap().clone();
    #[cfg(target_os = "linux")]
    let runner_loader = app.state::<ManifestLoaders>().runner.0.read().unwrap().clone();

    if let Some(repos) = get_repositories(app) {
        for repo in repos {
            if let Some(manifests) = get_manifests_by_repository_id(app, repo.id) {
                for m in manifests {
                    let loaded = game_loader.contains_key(&m.filename);
                    #[cfg(target_os = "linux")]
                    let loaded = loaded || runner_loader.contains_key(&m.filename);
                    if !loaded && m.enabled { update_manifest_enabled_by_id(app, m.id, false); }
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
    } else if let Some(m) = dbm { if !m.enabled { update_manifest_enabled_by_id(app, m.id, true); } }
}

pub fn get_manifests(app: &AppHandle) -> LinkedHashMap<String, GameManifest> {
    app.state::<ManifestLoaders>().game.0.read().unwrap().clone()
}

pub fn get_manifest(app: &AppHandle, filename: String) -> Option<GameManifest> {
    let loader = app.state::<ManifestLoaders>().game.0.read().unwrap().clone();

    if loader.contains_key(&filename) {
        let content = loader.get(&filename).unwrap();
        Some(content.clone())
    } else {
        None
    }
}

#[cfg(target_os = "linux")]
pub fn get_compatibilities(app: &AppHandle) -> LinkedHashMap<String, RunnerManifest> {
    app.state::<ManifestLoaders>().runner.0.read().unwrap().clone()
}

#[cfg(target_os = "linux")]
pub fn get_compatibility(app: &AppHandle, filename: &String) -> Option<RunnerManifest> {
    let loader = app.state::<ManifestLoaders>().runner.0.read().unwrap().clone();

    if loader.contains_key(filename) {
        let content = loader.get(filename).unwrap();
        Some(content.clone())
    } else {
        None
    }
}

#[cfg(target_os = "linux")]
#[derive(Default)]
pub struct RunnerLoader(pub RwLock<LinkedHashMap<String, RunnerManifest>>);

#[derive(Default)]
pub struct ManifestLoader(pub RwLock<LinkedHashMap<String, GameManifest>>);

pub struct ManifestLoaders {
    pub game: ManifestLoader,
    #[cfg(target_os = "linux")]
    pub runner: RunnerLoader,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
enum ManifestData {
    Game(GameManifest),
    Runner(RunnerManifest)
}