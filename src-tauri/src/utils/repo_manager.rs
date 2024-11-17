use std::fs;
use std::io::BufReader;
use std::path::{PathBuf};
use git2::Repository;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use crate::utils::db_manager::{create_manifest, create_repository};
use crate::utils::generate_cuid;

pub async fn setup_official_repository(app: &AppHandle, path: &PathBuf) {
    let url = "https://github.com/TeamKeqing/launcher-manifests.git";
    let repo_path = path.join("official_manifests");
    let repo_manifest = repo_path.join("repository.json");

    let tmp = url.split("/").collect::<Vec<&str>>()[4];
    let user = url.split("/").collect::<Vec<&str>>()[3];
    let repo_name = tmp.split(".").collect::<Vec<&str>>()[0];

    if !path.exists() {
        return;
    } else if !repo_path.exists() {
        let repo = Repository::clone(url, &repo_path).unwrap();
        
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

            let mut mids = Vec::new();
            for m in rma.manifests {
                async {
                    let cuid = generate_cuid();
                    create_manifest(app, cuid.clone(), repo_id.clone(), m.as_str(), true).await.unwrap(); // enable all default manifests?? make behavior for no enabled manifests
                    mids.push(cuid);
                }.await
            }

            ()

        }
    } else {
        println!("Something is fucked so baddd babyyyyy");
    }
}

// === STRUCTS ===

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
    pub filename: String,
    pub enabled: bool
}