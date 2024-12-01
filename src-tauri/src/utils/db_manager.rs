use std::collections::HashMap;
use std::fs;
use std::path::Path;
use futures_core::future::BoxFuture;
use sqlx::{Error, Pool, query, Sqlite, Executor, Row};
use sqlx::error::BoxDynError;
use sqlx::migrate::{MigrateDatabase, MigrationType, Migrator, Migration as SqlxMigration, MigrationSource};
use sqlx::sqlite::{SqliteConnectOptions, SqliteQueryResult};
use tauri::{AppHandle, Manager};
use tokio::sync::RwLock;
use crate::utils::repo_manager::{setup_official_repository, LauncherInstall, LauncherManifest, LauncherRepository};
use crate::utils::run_async_command;

pub fn init_db(app: &AppHandle) {
    let data_path = app.path().app_data_dir().unwrap();
    let conn_path = app.path().app_config_dir().unwrap();
    let conn_url = conn_path.join("storage.db");

    let manifests_dir = data_path.join("manifests");

    if !Path::new(&conn_url).exists() {
        
        fs::create_dir_all(&conn_path).unwrap();

        run_async_command(async {
            if !Sqlite::database_exists(conn_url.to_str().unwrap()).await.unwrap() {
                Sqlite::create_database(conn_url.to_str().unwrap()).await.unwrap();
                #[cfg(debug_assertions)]
                { println!("Database does not exist... Creating new one for you!"); }
            }
        });

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
            sql: r#"CREATE TABLE manifest ("id" string PRIMARY KEY,"repository_id" string,"display_name" string,"filename" string,"enabled" bool, CONSTRAINT fk_manifest_repo FOREIGN KEY(repository_id) REFERENCES repository(id));"#,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 3,
            description: "init_install_table",
            sql: r#"CREATE TABLE "install" ("id" string PRIMARY KEY,"manifest_id" string,"version" string,"name" string,"directory" string,"runner" string,"dxvk" string, CONSTRAINT fk_install_manifest FOREIGN KEY(manifest_id) REFERENCES manifest(id));"#,
            kind: MigrationKind::Up,
        }
    ];

    run_async_command(async {
        let instances = DbInstances::default();
        let mut lock = instances.0.write().await;

        let mut migrations = add_migrations("db", migrationsl);

        let pool: Pool<Sqlite> = Pool::connect(conn_url.to_str().unwrap()).await.unwrap();
        pool.set_connect_options(SqliteConnectOptions::new().foreign_keys(true));

        if let Some(migrations) = migrations.as_mut().unwrap().remove("db") {
            let migrator = Migrator::new(migrations).await.unwrap();
            migrator.run(&pool).await.unwrap();
        }

        lock.insert(String::from("db"), pool);
        drop(lock);
        app.manage(instances);
    });

    // Init this fuck AFTER you add shitty DB instances to state
    if !Path::new(&manifests_dir).exists() {
        fs::create_dir_all(&manifests_dir).unwrap();
        #[cfg(debug_assertions)]
        { println!("Manifests directory does not exist... Creating new one for you!"); }
        setup_official_repository(&app, &manifests_dir);
    } else {
        setup_official_repository(&app, &manifests_dir);
    }
}


// === SETTINGS ===

/*pub async fn get_settings(app: &AppHandle) -> Result<Vec<String>, Error> {
    let db = app.state::<DbInstances>().0.lock().unwrap().get("db").unwrap().clone();

    Ok(vec![])
}

pub async fn save_db_settings(app: &AppHandle) -> Result<bool, Error> {
    Ok(true)
}*/

// === REPOSITORIES ===

pub fn create_repository(app: &AppHandle, id: String, github_id: &str) -> Result<bool, Error> {
    let mut rslt = SqliteQueryResult::default();

    run_async_command(async {
        let db = app.state::<DbInstances>().0.read().await.get("db").unwrap().clone();

        let query = query("INSERT INTO repository(id, github_id) VALUES ($1, $2)").bind(id).bind(github_id);
        rslt = query.execute(&db).await.unwrap();
    });

    if rslt.rows_affected() >= 1 {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn delete_repository_by_id(app: &AppHandle, id: String) -> Result<bool, Error> {
    let mut rslt = SqliteQueryResult::default();

    run_async_command(async {
        let db = app.state::<DbInstances>().0.read().await.get("db").unwrap().clone();

        let query = query("DELETE FROM repository WHERE id = $1").bind(id);
        rslt = query.execute(&db).await.unwrap();
    });

    if rslt.rows_affected() >= 1 {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn get_repository_info_by_id(app: &AppHandle, id: String) -> Option<LauncherRepository> {
    let mut rslt = vec![];

    run_async_command(async {
        let db = app.state::<DbInstances>().0.read().await.get("db").unwrap().clone();

        let query = query("SELECT * FROM repository WHERE id = $1").bind(id);
        rslt = query.fetch_all(&db).await.unwrap();
    });

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

pub fn get_repositories(app: &AppHandle) -> Option<Vec<LauncherRepository>> {
    let mut rslt = vec![];

    run_async_command(async {
        let db = app.state::<DbInstances>().0.read().await.get("db").unwrap().clone();

        let query = query("SELECT * FROM repository");
        rslt = query.fetch_all(&db).await.unwrap();
    });

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

pub fn create_manifest(app: &AppHandle, id: String, repository_id: String, display_name: &str, filename: &str, enabled: bool) -> Result<bool, Error> {
    let mut rslt = SqliteQueryResult::default();

    run_async_command(async {
        let db = app.state::<DbInstances>().0.read().await.get("db").unwrap().clone();

        rslt = db.execute(format!("INSERT INTO manifest(id, repository_id, display_name, filename, enabled) VALUES ('{id}', '{repository_id}', '{display_name}', '{filename}', {enabled})").as_str()).await.unwrap();
    });

    if rslt.rows_affected() >= 1 {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn delete_manifest_by_repository_id(app: &AppHandle, repository_id: String) -> Result<bool, Error> {
    let mut rslt = SqliteQueryResult::default();

    run_async_command(async {
        let db = app.state::<DbInstances>().0.read().await.get("db").unwrap().clone();

        let query = query("DELETE FROM manifest WHERE repository_id = $1").bind(repository_id);
        rslt = query.execute(&db).await.unwrap();
    });

    if rslt.rows_affected() >= 1 {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn delete_manifest_by_id(app: &AppHandle, id: String) -> Result<bool, Error> {
    let mut rslt = SqliteQueryResult::default();

    run_async_command(async {
        let db = app.state::<DbInstances>().0.read().await.get("db").unwrap().clone();

        let query = query("DELETE FROM manifest WHERE id = $1").bind(id);
        rslt = query.execute(&db).await.unwrap();
    });

    if rslt.rows_affected() >= 1 {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn get_manifest_info_by_id(app: &AppHandle, id: String) -> Option<LauncherManifest> {
    let mut rslt = vec![];

    run_async_command(async {
        let db = app.state::<DbInstances>().0.read().await.get("db").unwrap().clone();

        let query = query("SELECT * FROM manifest WHERE id = $1").bind(id);
        rslt = query.fetch_all(&db).await.unwrap();
    });

    if rslt.len() >= 1 {
        let rsltt = LauncherManifest {
            id: rslt.get(0).unwrap().get("id"),
            repository_id: rslt.get(0).unwrap().get("repository_id"),
            display_name: rslt.get(0).unwrap().get("display_name"),
            filename: rslt.get(0).unwrap().get("filename"),
            enabled: rslt.get(0).unwrap().get("enabled")
        };

        Some(rsltt)
    } else {
        None
    }
}

pub fn get_manifest_info_by_filename(app: &AppHandle, filename: String) -> Option<LauncherManifest> {
    let mut rslt = vec![];

    run_async_command(async {
        let db = app.state::<DbInstances>().0.read().await.get("db").unwrap().clone();

        let query = query("SELECT * FROM manifest WHERE filename = $1").bind(filename);
        rslt = query.fetch_all(&db).await.unwrap();
    });

    if rslt.len() >= 1 {
        let rsltt = LauncherManifest {
            id: rslt.get(0).unwrap().get("id"),
            repository_id: rslt.get(0).unwrap().get("repository_id"),
            display_name: rslt.get(0).unwrap().get("display_name"),
            filename: rslt.get(0).unwrap().get("filename"),
            enabled: rslt.get(0).unwrap().get("enabled")
        };

        Some(rsltt)
    } else {
        None
    }
}

pub fn get_manifests_by_repository_id(app: &AppHandle, repository_id: String) -> Option<Vec<LauncherManifest>> {
    let mut rslt = vec![];

    run_async_command(async {
        let db = app.state::<DbInstances>().0.read().await.get("db").unwrap().clone();

        let query = query("SELECT * FROM manifest WHERE repository_id = $1").bind(repository_id);
        rslt = query.fetch_all(&db).await.unwrap();
    });

    if rslt.len() >= 1 {
        let mut rsltt = Vec::<LauncherManifest>::new();
        for r in rslt {
            rsltt.push(LauncherManifest {
                id: r.get("id"),
                repository_id: r.get("repository_id"),
                display_name: r.get("display_name"),
                filename: r.get("filename"),
                enabled: r.get("enabled")
            })
        }

        Some(rsltt)
    } else {
        None
    }
}

// === INSTALLS ===

pub fn create_installation(app: &AppHandle, id: String, manifest_id: String, version: String, name: String, directory: String, runner: String, dxvk: String) -> Result<bool, Error> {
    let mut rslt = SqliteQueryResult::default();

    run_async_command(async {
        let db = app.state::<DbInstances>().0.read().await.get("db").unwrap().clone();

        let query = query("INSERT INTO install(id, manifest_id, version, name, directory, runner, dxvk) VALUES ($1, $2, $3, $4, $5, $6, $7)").bind(id).bind(manifest_id).bind(version).bind(name).bind(directory).bind(runner).bind(dxvk);
        rslt = query.execute(&db).await.unwrap();
    });

    if rslt.rows_affected() >= 1 {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn delete_installation_by_id(app: &AppHandle, id: String) -> Result<bool, Error> {
    let mut rslt = SqliteQueryResult::default();

    run_async_command(async {
        let db = app.state::<DbInstances>().0.read().await.get("db").unwrap().clone();

        let query = query("DELETE FROM install WHERE id = $1").bind(id);
        rslt = query.execute(&db).await.unwrap();
    });

    if rslt.rows_affected() >= 1 {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn get_install_info_by_id(app: &AppHandle, id: String) -> Option<LauncherInstall> {
    let mut rslt = vec![];

    run_async_command(async {
        let db = app.state::<DbInstances>().0.read().await.get("db").unwrap().clone();

        let query = query("SELECT * FROM install WHERE id = $1").bind(id);
        rslt = query.fetch_all(&db).await.unwrap();
    });

    if rslt.len() >= 1 {
        let rsltt = LauncherInstall {
            id: rslt.get(0).unwrap().get("id"),
            manifest_id: rslt.get(0).unwrap().get("manifest_id"),
            version: rslt.get(0).unwrap().get("version"),
            name: rslt.get(0).unwrap().get("name"),
            directory: rslt.get(0).unwrap().get("directory"),
            runner: rslt.get(0).unwrap().get("runner"),
            dxvk: rslt.get(0).unwrap().get("dxvk"),
        };

        Some(rsltt)
    } else {
        None
    }
}

pub fn get_installs_by_manifest_id(app: &AppHandle, manifest_id: String) -> Option<Vec<LauncherInstall>> {
    let mut rslt = vec![];

    run_async_command(async {
        let db = app.state::<DbInstances>().0.read().await.get("db").unwrap().clone();

        let query = query("SELECT * FROM install WHERE manifest_id = $1").bind(manifest_id);
        rslt = query.fetch_all(&db).await.unwrap();
    });

    if rslt.len() >= 1 {
        let mut rsltt = Vec::<LauncherInstall>::new();
        for r in rslt {
            rsltt.push(LauncherInstall {
                id: r.get("id"),
                manifest_id: r.get("manifest_id"),
                version: r.get("version"),
                name: r.get("name"),
                directory: r.get("directory"),
                runner: r.get("runner"),
                dxvk: r.get("dxvk"),
            })
        }

        Some(rsltt)
    } else {
        None
    }
}

pub fn get_installs(app: &AppHandle) -> Option<Vec<LauncherInstall>> {
    let mut rslt = vec![];

    run_async_command(async {
        let db = app.state::<DbInstances>().0.read().await.get("db").unwrap().clone();

        let query = query("SELECT * FROM install");
        rslt = query.fetch_all(&db).await.unwrap();
    });

    if rslt.len() >= 1 {
        let mut rsltt = Vec::<LauncherInstall>::new();
        for r in rslt {
            rsltt.push(LauncherInstall {
                id: r.get("id"),
                manifest_id: r.get("manifest_id"),
                version: r.get("version"),
                name: r.get("name"),
                directory: r.get("directory"),
                runner: r.get("runner"),
                dxvk: r.get("dxvk"),
            })
        }

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

#[derive(Default)]
pub struct DbInstances(pub RwLock<HashMap<String, Pool<Sqlite>>>);

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