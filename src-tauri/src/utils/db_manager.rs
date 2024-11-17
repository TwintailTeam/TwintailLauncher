use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use futures_core::future::BoxFuture;
use sqlx::{Error, Pool, query, Sqlite, Executor, Row};
use sqlx::error::BoxDynError;
use sqlx::migrate::{MigrateDatabase, MigrationType, Migrator, Migration as SqlxMigration, MigrationSource};
use sqlx::sqlite::SqliteConnectOptions;
use tauri::{AppHandle, Manager};
use crate::utils::repo_manager::{setup_official_repository, LauncherManifest, LauncherRepository};

pub async fn init_db(app: &AppHandle) {
    let data_path = app.path().app_data_dir().unwrap();
    let conn_path = app.path().app_config_dir().unwrap();
    let conn_url = conn_path.join("storage.db");

    let manifests_dir = data_path.join("manifests");

    if !Path::new(&conn_url).exists() {
        
        fs::create_dir_all(&conn_path).unwrap();

        if !Sqlite::database_exists(&conn_url.to_str().unwrap()).await.unwrap() {
            Sqlite::create_database(&conn_url.to_str().unwrap()).await.unwrap();
            #[cfg(debug_assertions)]
            {
                println!("Database does not exist... Creating new one for you!");
            }
        }

    }

    let migrationsl = vec![
        Migration {
            version: 1,
            description: "init_repository_table",
            sql: r#"CREATE TABLE "repository" ("id" string PRIMARY KEY,"github_id" string);"#,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "init_manifest_table",
            sql: r#"CREATE TABLE manifest ("id" string PRIMARY KEY,"repository_id" string,"filename" string,"enabled" bool, CONSTRAINT fk_manifest_repo FOREIGN KEY(repository_id) REFERENCES repository(id));"#,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 3,
            description: "init_install_table",
            sql: r#"CREATE TABLE "install" ("id" string PRIMARY KEY,"manifest_id" string,"version" string,"name" string,"directory" string,"runner" string,"dxvk" string, CONSTRAINT fk_install_manifest FOREIGN KEY(manifest_id) REFERENCES manifest(id));"#,
            kind: MigrationKind::Up,
        }
    ];

    let mut migrations = add_migrations("db", migrationsl);

    let instances = DbInstances::default();
    let mut tmp = instances.0.lock().unwrap();

    let pool: Pool<Sqlite> = Pool::connect(&conn_url.to_str().unwrap()).await.unwrap();
    pool.set_connect_options(SqliteConnectOptions::new().foreign_keys(true));

    tmp.insert(String::from("db"), pool.clone());

    if let Some(migrations) = migrations.as_mut().unwrap().remove("db") {
        let migrator = Migrator::new(migrations).await.unwrap();
        migrator.run(&pool).await.unwrap();
    }

    drop(tmp);

    app.manage(instances);

    // Init this fuck AFTER you add shitty DB instances to state
    if !Path::new(&manifests_dir).exists() {
        // TODO: fallback to backup path if $XDG_DATA_HOME is not set / any path resolution fails...
        fs::create_dir_all(&manifests_dir).unwrap();
        #[cfg(debug_assertions)]
        {
            println!("Manifests directory does not exist... Creating new one for you!");
        }
        setup_official_repository(&app, &manifests_dir).await;
    } else {
        setup_official_repository(&app, &manifests_dir).await;
    }
}

// === SETTINGS ===

pub async fn get_settings(app: &AppHandle) -> Result<Vec<String>, Error> {
    let db = app.state::<DbInstances>().0.lock().unwrap().get("db").unwrap().clone();

    Ok(vec![])
}

pub async fn save_db_settings(app: &AppHandle) -> Result<bool, Error> {
    Ok(true)
}

// === REPOSITORIES ===

pub async fn create_repository(app: &AppHandle, id: String, github_id: &str) -> Result<bool, Error> {
    let db = app.state::<DbInstances>().0.lock().unwrap().get("db").unwrap().clone();

    let query = query("INSERT INTO repository(id, github_id) VALUES ($1, $2)").bind(id).bind(github_id);
    let rslt = query.execute(&db).await?;

    if rslt.rows_affected() >= 1 {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub async fn delete_repository(app: &AppHandle, id: String) -> Result<bool, Error> {
    let db = app.state::<DbInstances>().0.lock().unwrap().get("db").unwrap().clone();

    let query = query("DELETE FROM repository WHERE id = $1").bind(id);
    let rslt = query.execute(&db).await?;

    if rslt.rows_affected() >= 1 {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub async fn get_repository_by_id(app: &AppHandle, id: String) -> Option<LauncherRepository> {
    let db = app.state::<DbInstances>().0.lock().unwrap().get("db").unwrap().clone();

    let query = query("SELECT * FROM repository WHERE id = $1").bind(id);
    let rslt = query.fetch_all(&db).await.unwrap();

    if rslt.len() >= 1 {
        let rsltt = LauncherRepository {
            id: rslt.get(0).unwrap().get("id"),
            github_id: rslt.get(0).unwrap().get("github_id"),
        };

        Some(rsltt)
    } else {
        None
    }
}

pub async fn get_repositories(app: &AppHandle) -> Option<Vec<LauncherRepository>> {
    let db = app.state::<DbInstances>().0.lock().unwrap().get("db").unwrap().clone();

    let query = query("SELECT * FROM repository");
    let rslt = query.fetch_all(&db).await.unwrap();

    if rslt.len() >= 1 {
        let mut rsltt = Vec::<LauncherRepository>::new();
        for r in rslt {
            rsltt.push(LauncherRepository {
                id: r.get("id"),
                github_id: r.get("github_id"),
            })
        }

        Some(rsltt)
    } else {
        None
    }
}

// === MANIFESTS ===

pub async fn create_manifest(app: &AppHandle, id: String, repository_id: String, filename: &str, enabled: bool) -> Result<bool, Error> {
    let db = app.state::<DbInstances>().0.lock().unwrap().get("db").unwrap().clone();

    let rslt = db.execute(format!("INSERT INTO manifest(id, repository_id, filename, enabled) VALUES ('{id}', '{repository_id}', '{filename}', {enabled})").as_str()).await?;

    if rslt.rows_affected() >= 1 {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub async fn delete_manifest_by_repository_id(app: &AppHandle, repository_id: String) -> Result<bool, Error> {
    let db = app.state::<DbInstances>().0.lock().unwrap().get("db").unwrap().clone();

    let query = query("DELETE FROM manifest WHERE repository_id = $1").bind(repository_id);
    let rslt = query.execute(&db).await?;

    if rslt.rows_affected() >= 1 {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub async fn delete_manifest_by_id(app: &AppHandle, id: String) -> Result<bool, Error> {
    let db = app.state::<DbInstances>().0.lock().unwrap().get("db").unwrap().clone();

    let query = query("DELETE FROM manifest WHERE _id = $1").bind(id);
    let rslt = query.execute(&db).await?;

    if rslt.rows_affected() >= 1 {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub async fn get_manifest_info_by_id(app: &AppHandle, id: String) -> Option<LauncherManifest> {
    let db = app.state::<DbInstances>().0.lock().unwrap().get("db").unwrap().clone();

    let query = query("SELECT * FROM manifest WHERE id = $1").bind(id);
    let rslt = query.fetch_all(&db).await.unwrap();

    if rslt.len() >= 1 {
        let rsltt = LauncherManifest {
            id: rslt.get(0).unwrap().get("id"),
            repository_id: rslt.get(0).unwrap().get("repository_id"),
            filename: rslt.get(0).unwrap().get("filename"),
            enabled: rslt.get(0).unwrap().get("enabled"),
        };

        Some(rsltt)
    } else {
        None
    }
}

pub async fn get_manifest_info_by_filename(app: &AppHandle, filename: String) -> Option<LauncherManifest> {
    let db = app.state::<DbInstances>().0.lock().unwrap().get("db").unwrap().clone();

    let query = query("SELECT * FROM manifest WHERE filename = $1").bind(filename);
    let rslt = query.fetch_all(&db).await.unwrap();

    if rslt.len() >= 1 {
        let rsltt = LauncherManifest {
            id: rslt.get(0).unwrap().get("id"),
            repository_id: rslt.get(0).unwrap().get("repository_id"),
            filename: rslt.get(0).unwrap().get("filename"),
            enabled: rslt.get(0).unwrap().get("enabled"),
        };

        Some(rsltt)
    } else {
        None
    }
}

// === DB RELATED ===

fn add_migrations(db_url: &str, migrations: Vec<Migration>) -> Option<HashMap<String, MigrationList>> {
    let mut migrs: Option<HashMap<String, MigrationList>> = Some(HashMap::new());

    migrs.get_or_insert(Default::default()).insert(db_url.to_string(), MigrationList(migrations));
    migrs
}

#[derive(Default, Debug)]
pub struct DbInstances(Mutex<HashMap<String, Pool<Sqlite>>>);

#[derive(Debug)]
pub enum MigrationKind {
    Up,
    Down,
}

impl From<MigrationKind> for MigrationType {
    fn from(kind: MigrationKind) -> Self {
        match kind {
            MigrationKind::Up => Self::ReversibleUp,
            MigrationKind::Down => Self::ReversibleDown,
        }
    }
}

/// A migration definition.
#[derive(Debug)]
pub struct Migration {
    pub version: i64,
    pub description: &'static str,
    pub sql: &'static str,
    pub kind: MigrationKind,
}

#[derive(Debug)]
struct MigrationList(Vec<Migration>);

//struct Migrations(Mutex<HashMap<String, MigrationList>>);

impl MigrationSource<'static> for MigrationList {
    fn resolve(self) -> BoxFuture<'static, Result<Vec<SqlxMigration>, BoxDynError>> {
        Box::pin(async move {
            let mut migrations = Vec::new();
            for migration in self.0 {
                if matches!(migration.kind, MigrationKind::Up) {
                    migrations.push(SqlxMigration::new(migration.version, migration.description.into(), migration.kind.into(), migration.sql.into()));
                }
            }
            Ok(migrations)
        })
    }
}