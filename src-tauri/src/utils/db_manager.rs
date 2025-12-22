use std::collections::HashMap;
use std::fs;
use futures_core::future::BoxFuture;
use sqlx::{query, Error, Executor, Pool, Row, Sqlite, error::BoxDynError, sqlite::SqliteQueryResult, migrate::{Migration as SqlxMigration, MigrateDatabase, MigrationSource, MigrationType, Migrator}};
use sqlx::types::Json;
use tauri::{AppHandle, Manager};
use tokio::sync::{Mutex};
use crate::utils::repo_manager::{setup_compatibility_repository, setup_official_repository};
use crate::utils::{run_async_command, setup_or_fix_default_paths};
use crate::utils::models::{GlobalSettings, LauncherInstall, LauncherManifest, LauncherRepository, LauncherRunner, XXMISettings};

pub async fn init_db(app: &AppHandle) {
    let data_path = app.path().app_data_dir().unwrap();
    let conn_url = data_path.join("storage.db");
    let manifests_dir = data_path.join("manifests");

    if !conn_url.exists() {
        fs::create_dir_all(&data_path).unwrap();

        if !Sqlite::database_exists(conn_url.to_str().unwrap()).await.unwrap() {
            Sqlite::create_database(conn_url.to_str().unwrap()).await.unwrap();
            #[cfg(debug_assertions)]
            { println!("Database does not exist... Creating new one for you!"); }
        }
    }

    let migrationsl = vec![
        Migration {
            version: 1,
            description: "init_repository_table",
            sql: r#"CREATE TABLE IF NOT EXISTS "repository" ("id" string PRIMARY KEY,"github_id" string);"#,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "init_manifest_table",
            sql: r#"CREATE TABLE IF NOT EXISTS manifest ("id" string PRIMARY KEY, "repository_id" string, "display_name" string, "filename" string, "enabled" bool, CONSTRAINT fk_manifest_repo FOREIGN KEY(repository_id) REFERENCES repository(id));"#,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 6,
            description: "init_install_table",
            sql: r#"CREATE TABLE IF NOT EXISTS install ("id" TEXT PRIMARY KEY, "manifest_id" TEXT, "version" TEXT, "name" TEXT, "directory" TEXT, "runner_path" TEXT, "dxvk_path" TEXT, "runner_version" TEXT, "dxvk_version" TEXT, "game_icon" TEXT, "game_background" TEXT, "ignore_updates" bool, "skip_hash_check" bool, "use_jadeite" bool, "use_xxmi" bool, "use_fps_unlock" bool, "env_vars" TEXT, "pre_launch_command" TEXT, "launch_command" TEXT, "fps_value" TEXT, "runner_prefix_path" TEXT, "launch_args" TEXT, "audio_langs" TEXT, CONSTRAINT fk_install_manifest FOREIGN KEY(manifest_id) REFERENCES manifest(id));"#,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 7,
            description: "init_settings_table",
            sql: r#"CREATE TABLE IF NOT EXISTS settings ("default_game_path" TEXT default null, "third_party_repo_updates" bool default 0 not null, "xxmi_path" TEXT default null, fps_unlock_path TEXT default null, jadeite_path TEXT default null, default_runner_prefix_path TEXT default null, "launcher_action" TEXT default null, id integer not null CONSTRAINT settings_pk primary key autoincrement, "hide_manifests" bool not null);"#,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 5,
            description: "populate_settings_table",
            sql: r#"INSERT INTO settings (default_game_path, third_party_repo_updates, xxmi_path, fps_unlock_path, jadeite_path, default_runner_prefix_path, launcher_action, id, hide_manifests) values (null, false, null, null, null, null, "exit", 1, false);"#,
            kind: MigrationKind::Up,
        },
        // Beginning of update migrations
        Migration {
            version: 8,
            description: "alter_install_table_106",
            sql: r#"ALTER TABLE install ADD COLUMN use_gamemode bool DEFAULT false NOT NULL;"#,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 9,
            description: "alter_settings_table_108",
            sql: r#"ALTER TABLE settings ADD COLUMN default_runner_path TEXT default null;"#,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 10,
            description: "alter_settings_table_108_2",
            sql: r#"ALTER TABLE settings ADD COLUMN default_dxvk_path TEXT default null;"#,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 11,
            description: "alter_install_table_108_3",
            sql: r#"ALTER TABLE install ADD COLUMN use_mangohud bool DEFAULT false NOT NULL;"#,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 12,
            description: "alter_settings_table_109",
            sql: r#"UPDATE settings SET launcher_action = 'keep' WHERE id = 1;"#,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 13,
            description: "alter_install_table_112_1",
            sql: r#"ALTER TABLE install ADD COLUMN mangohud_config_path TEXT default null;"#,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 14,
            description: "alter_settings_table_112_2",
            sql: r#"ALTER TABLE settings ADD COLUMN default_mangohud_config_path TEXT default null;"#,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 15,
            description: "alter_install_table_118_1",
            sql: r#"ALTER TABLE install ADD COLUMN shortcut_is_steam bool DEFAULT false NOT NULL;"#,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 16,
            description: "alter_install_table_118_2",
            sql: r#"ALTER TABLE install ADD COLUMN shortcut_path TEXT default null;"#,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 17,
            description: "init_installed_runners_table_118",
            sql: r#"CREATE TABLE IF NOT EXISTS installed_runners ("runner_path" TEXT default null, "is_installed" bool default 0 not null, "version" TEXT default null, id integer not null CONSTRAINT ir_pk primary key autoincrement);"#,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 18,
            description: "alter_install_table_1113",
            sql: r#"ALTER TABLE install ADD COLUMN region_code TEXT DEFAULT 'glb_official' NOT NULL;"#,
            kind: MigrationKind::Up,
        },
        Migration {
            version: 19,
            description: "alter_install_table_1114",
            sql: r#"ALTER TABLE install ADD COLUMN xxmi_config TEXT DEFAULT '{"hunting_mode":0,"require_admin":true,"dll_init_delay":500,"close_delay":20,"show_warnings":0,"dump_shaders":false}' NOT NULL;"#,
            kind: MigrationKind::Up,
        },
    ];

    let mut migrations = add_migrations("db", migrationsl);

    let instances = DbInstances::default();
    let mut tmp = instances.0.lock().await;
    let pool: Pool<Sqlite> = Pool::connect(&conn_url.to_str().unwrap()).await.unwrap();
    tmp.insert(String::from("db"), pool.clone());

    if let Some(migrations) = migrations.as_mut().unwrap().remove("db") {
        let migrator = Migrator::new(migrations).await.unwrap();
        migrator.run(&pool).await.unwrap();
    }

    drop(tmp);
    app.manage(instances);

    // Init and setup default paths...
    setup_or_fix_default_paths(app, data_path, false);

    // Init this fuck AFTER you add shitty DB instances to state
    if !manifests_dir.exists() {
        fs::create_dir_all(&manifests_dir).unwrap();
        #[cfg(debug_assertions)]
        { println!("Manifests directory does not exist... Creating new one for you!"); }
        setup_official_repository(&app, &manifests_dir);
        setup_compatibility_repository(&app, &manifests_dir);
    } else {
        setup_official_repository(&app, &manifests_dir);
        setup_compatibility_repository(&app, &manifests_dir);
    }
}


// === SETTINGS ===

pub fn get_settings(app: &AppHandle) -> Option<GlobalSettings> {
    let mut rslt = vec![];

    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("SELECT * FROM settings WHERE id = 1");
        rslt = query.fetch_all(&db).await.unwrap();
    });

    if rslt.len() >= 1 {
        let rsltt = GlobalSettings {
            default_game_path: rslt.get(0).unwrap().get("default_game_path"),
            xxmi_path: rslt.get(0).unwrap().get("xxmi_path"),
            fps_unlock_path: rslt.get(0).unwrap().get("fps_unlock_path"),
            jadeite_path: rslt.get(0).unwrap().get("jadeite_path"),
            third_party_repo_updates: rslt.get(0).unwrap().get("third_party_repo_updates"),
            default_runner_prefix_path: rslt.get(0).unwrap().get("default_runner_prefix_path"),
            launcher_action: rslt.get(0).unwrap().get("launcher_action"),
            hide_manifests: rslt.get(0).unwrap().get("hide_manifests"),
            default_runner_path: rslt.get(0).unwrap().get("default_runner_path"),
            default_dxvk_path: rslt.get(0).unwrap().get("default_dxvk_path"),
            default_mangohud_config_path: rslt.get(0).unwrap().get("default_mangohud_config_path"),
        };

        Some(rsltt)
    } else {
        None
    }
}

pub fn update_settings_third_party_repo_update(app: &AppHandle, enabled: bool) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE settings SET 'third_party_repo_updates' = $1 WHERE id = 1").bind(enabled);
        query.execute(&db).await.unwrap();
    });
}

pub fn update_settings_default_game_location(app: &AppHandle, path: String) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE settings SET 'default_game_path' = $1 WHERE id = 1").bind(path);
        query.execute(&db).await.unwrap();
    });
}

pub fn update_settings_default_xxmi_location(app: &AppHandle, path: String) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE settings SET 'xxmi_path' = $1 WHERE id = 1").bind(path);
        query.execute(&db).await.unwrap();
    });
}

pub fn update_settings_default_fps_unlock_location(app: &AppHandle, path: String) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE settings SET 'fps_unlock_path' = $1 WHERE id = 1").bind(path);
        query.execute(&db).await.unwrap();
    });
}

pub fn update_settings_default_jadeite_location(app: &AppHandle, path: String) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE settings SET 'jadeite_path' = $1 WHERE id = 1").bind(path);
        query.execute(&db).await.unwrap();
    });
}

pub fn update_settings_default_prefix_location(app: &AppHandle, path: String) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE settings SET 'default_runner_prefix_path' = $1 WHERE id = 1").bind(path);
        query.execute(&db).await.unwrap();
    });
}

pub fn update_settings_default_runner_location(app: &AppHandle, path: String) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE settings SET 'default_runner_path' = $1 WHERE id = 1").bind(path);
        query.execute(&db).await.unwrap();
    });
}

pub fn update_settings_default_dxvk_location(app: &AppHandle, path: String) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE settings SET 'default_dxvk_path' = $1 WHERE id = 1").bind(path);
        query.execute(&db).await.unwrap();
    });
}

pub fn update_settings_default_mangohud_config_location(app: &AppHandle, path: String) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE settings SET 'default_mangohud_config_path' = $1 WHERE id = 1").bind(path);
        query.execute(&db).await.unwrap();
    });
}

pub fn update_settings_launch_action(app: &AppHandle, action: String) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE settings SET 'launcher_action' = $1 WHERE id = 1").bind(action);
        query.execute(&db).await.unwrap();
    });
}

pub fn update_settings_hide_manifests(app: &AppHandle, enabled: bool) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE settings SET 'hide_manifests' = $1 WHERE id = 1").bind(enabled);
        query.execute(&db).await.unwrap();
    });
}

// === REPOSITORIES ===

pub fn create_repository(app: &AppHandle, id: String, github_id: &str) -> Result<bool, Error> {
    let mut rslt = SqliteQueryResult::default();

    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

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
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

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
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

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

pub fn get_repository_info_by_github_id(app: &AppHandle, github_id: String) -> Option<LauncherRepository> {
    let mut rslt = vec![];

    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("SELECT * FROM repository WHERE github_id = $1").bind(github_id);
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
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

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
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        rslt = db.execute(format!("INSERT INTO manifest(id, repository_id, display_name, filename, enabled) VALUES ('{id}', '{repository_id}', '{display_name}', '{filename}', {enabled})").as_str()).await.unwrap();
    });

    if rslt.rows_affected() >= 1 {
        Ok(true)
    } else {
        Ok(false)
    }
}

#[allow(dead_code)]
pub fn delete_manifest_by_repository_id(app: &AppHandle, repository_id: String) -> Result<bool, Error> {
    let mut rslt = SqliteQueryResult::default();

    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();
        rslt = query("DELETE FROM manifest WHERE repository_id = ? LIMIT = 1;").bind(&repository_id).execute(&db).await.unwrap();
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
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

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
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

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
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

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
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

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

pub fn update_manifest_enabled_by_id(app: &AppHandle, id: String, enabled: bool) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE manifest SET 'enabled' = $1 WHERE id = $2").bind(enabled).bind(id);
        query.execute(&db).await.unwrap();
    });
}

// === INSTALLS ===

pub fn create_installation(app: &AppHandle, id: String, manifest_id: String, version: String, audio_langs: String, name: String, directory: String, runner_path: String, dxvk_path: String, runner_version: String, dxvk_version: String, game_icon: String, game_background: String, ignore_updates: bool, skip_hash_check: bool, use_jadeite: bool, use_xxmi: bool, use_fps_unlock: bool, env_vars: String, pre_launch_command: String, launch_command: String, fps_value: String, runner_prefix_path: String, launch_args: String, use_gamemode: bool, use_mangohud: bool, mangohud_config_path: String, region_code: String) -> Result<bool, Error> {
    let mut rslt = SqliteQueryResult::default();

    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("INSERT INTO install(id, manifest_id, version, name, directory, runner_path, dxvk_path, runner_version, dxvk_version, game_icon, game_background, ignore_updates, skip_hash_check, use_jadeite, use_xxmi, use_fps_unlock, env_vars, pre_launch_command, launch_command, fps_value, runner_prefix_path, launch_args, audio_langs, use_gamemode, use_mangohud, mangohud_config_path, region_code) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27)").bind(id).bind(manifest_id).bind(version).bind(name).bind(directory).bind(runner_path).bind(dxvk_path).bind(runner_version).bind(dxvk_version).bind(game_icon).bind(game_background).bind(ignore_updates).bind(skip_hash_check).bind(use_jadeite).bind(use_xxmi).bind(use_fps_unlock).bind(env_vars).bind(pre_launch_command).bind(launch_command).bind(fps_value).bind(runner_prefix_path).bind(launch_args).bind(audio_langs).bind(use_gamemode).bind(use_mangohud).bind(mangohud_config_path).bind(region_code);
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
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

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
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("SELECT * FROM install WHERE id = $1").bind(id);
        rslt = query.fetch_all(&db).await.unwrap();
    });

    if rslt.len() >= 1 {
        let rsltt = LauncherInstall {
            id: rslt.get(0).unwrap().get("id"),
            manifest_id: rslt.get(0).unwrap().get("manifest_id"),
            version: rslt.get(0).unwrap().get("version"),
            audio_langs: rslt.get(0).unwrap().get("audio_langs"),
            name: rslt.get(0).unwrap().get("name"),
            directory: rslt.get(0).unwrap().get("directory"),
            runner_path: rslt.get(0).unwrap().get("runner_path"),
            dxvk_path: rslt.get(0).unwrap().get("dxvk_path"),
            runner_version: rslt.get(0).unwrap().get("runner_version"),
            dxvk_version: rslt.get(0).unwrap().get("dxvk_version"),
            game_icon: rslt.get(0).unwrap().get("game_icon"),
            game_background: rslt.get(0).unwrap().get("game_background"),
            ignore_updates: rslt.get(0).unwrap().get("ignore_updates"),
            skip_hash_check: rslt.get(0).unwrap().get("skip_hash_check"),
            use_jadeite: rslt.get(0).unwrap().get("use_jadeite"),
            use_xxmi: rslt.get(0).unwrap().get("use_xxmi"),
            use_fps_unlock: rslt.get(0).unwrap().get("use_fps_unlock"),
            env_vars: rslt.get(0).unwrap().get("env_vars"),
            pre_launch_command: rslt.get(0).unwrap().get("pre_launch_command"),
            launch_command: rslt.get(0).unwrap().get("launch_command"),
            fps_value: rslt.get(0).unwrap().get("fps_value"),
            runner_prefix: rslt.get(0).unwrap().get("runner_prefix_path"),
            launch_args: rslt.get(0).unwrap().get("launch_args"),
            use_gamemode: rslt.get(0).unwrap().get("use_gamemode"),
            use_mangohud: rslt.get(0).unwrap().get("use_mangohud"),
            mangohud_config_path: rslt.get(0).unwrap().get("mangohud_config_path"),
            shortcut_is_steam: rslt.get(0).unwrap().get("shortcut_is_steam"),
            shortcut_path: rslt.get(0).unwrap().get("shortcut_path"),
            region_code: rslt.get(0).unwrap().get("region_code"),
            xxmi_config: rslt.get(0).unwrap().get("xxmi_config")
        };

        Some(rsltt)
    } else {
        None
    }
}

pub fn get_installs_by_manifest_id(app: &AppHandle, manifest_id: String) -> Option<Vec<LauncherInstall>> {
    let mut rslt = vec![];

    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

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
                audio_langs: r.get("audio_langs"),
                name: r.get("name"),
                directory: r.get("directory"),
                runner_path: r.get("runner_path"),
                dxvk_path: r.get("dxvk_path"),
                runner_version: r.get("runner_version"),
                dxvk_version: r.get("dxvk_version"),
                game_icon: r.get("game_icon"),
                game_background: r.get("game_background"),
                ignore_updates: r.get("ignore_updates"),
                skip_hash_check: r.get("skip_hash_check"),
                use_jadeite: r.get("use_jadeite"),
                use_xxmi: r.get("use_xxmi"),
                use_fps_unlock: r.get("use_fps_unlock"),
                env_vars: r.get("env_vars"),
                pre_launch_command: r.get("pre_launch_command"),
                launch_command: r.get("launch_command"),
                fps_value: r.get("fps_value"),
                runner_prefix: r.get("runner_prefix_path"),
                launch_args: r.get("launch_args"),
                use_gamemode: r.get("use_gamemode"),
                use_mangohud: r.get("use_mangohud"),
                mangohud_config_path: r.get("mangohud_config_path"),
                shortcut_is_steam: r.get("shortcut_is_steam"),
                shortcut_path: r.get("shortcut_path"),
                region_code: r.get("region_code"),
                xxmi_config: r.get("xxmi_config")
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
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

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
                audio_langs: r.get("audio_langs"),
                name: r.get("name"),
                directory: r.get("directory"),
                runner_path: r.get("runner_path"),
                dxvk_path: r.get("dxvk_path"),
                runner_version: r.get("runner_version"),
                dxvk_version: r.get("dxvk_version"),
                game_icon: r.get("game_icon"),
                game_background: r.get("game_background"),
                ignore_updates: r.get("ignore_updates"),
                skip_hash_check: r.get("skip_hash_check"),
                use_jadeite: r.get("use_jadeite"),
                use_xxmi: r.get("use_xxmi"),
                use_fps_unlock: r.get("use_fps_unlock"),
                env_vars: r.get("env_vars"),
                pre_launch_command: r.get("pre_launch_command"),
                launch_command: r.get("launch_command"),
                fps_value: r.get("fps_value"),
                runner_prefix: r.get("runner_prefix_path"),
                launch_args: r.get("launch_args"),
                use_gamemode: r.get("use_gamemode"),
                use_mangohud: r.get("use_mangohud"),
                mangohud_config_path: r.get("mangohud_config_path"),
                shortcut_is_steam: r.get("shortcut_is_steam"),
                shortcut_path: r.get("shortcut_path"),
                region_code: r.get("region_code"),
                xxmi_config: r.get("xxmi_config")
            })
        }

        Some(rsltt)
    } else {
        None
    }
}

pub fn update_install_game_location_by_id(app: &AppHandle, id: String, location: String) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE install SET 'directory' = $1 WHERE id = $2").bind(location).bind(id);
        query.execute(&db).await.unwrap();
    });
}

pub fn update_install_runner_location_by_id(app: &AppHandle, id: String, location: String) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE install SET 'runner_path' = $1 WHERE id = $2").bind(location).bind(id);
        query.execute(&db).await.unwrap();
    });
}

pub fn update_install_dxvk_location_by_id(app: &AppHandle, id: String, location: String) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE install SET 'dxvk_path' = $1 WHERE id = $2").bind(location).bind(id);
        query.execute(&db).await.unwrap();
    });
}

pub fn update_install_ignore_updates_by_id(app: &AppHandle, id: String, enabled: bool) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE install SET 'ignore_updates' = $1 WHERE id = $2").bind(enabled).bind(id);
        query.execute(&db).await.unwrap();
    });
}

pub fn update_install_skip_hash_check_by_id(app: &AppHandle, id: String, enabled: bool) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE install SET 'skip_hash_check' = $1 WHERE id = $2").bind(enabled).bind(id);
        query.execute(&db).await.unwrap();
    });
}

pub fn update_install_use_jadeite_by_id(app: &AppHandle, id: String, enabled: bool) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE install SET 'use_jadeite' = $1 WHERE id = $2").bind(enabled).bind(id);
        query.execute(&db).await.unwrap();
    });
}

pub fn update_install_use_xxmi_by_id(app: &AppHandle, id: String, enabled: bool) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE install SET 'use_xxmi' = $1 WHERE id = $2").bind(enabled).bind(id);
        query.execute(&db).await.unwrap();
    });
}

pub fn update_install_use_fps_unlock_by_id(app: &AppHandle, id: String, enabled: bool) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE install SET 'use_fps_unlock' = $1 WHERE id = $2").bind(enabled).bind(id);
        query.execute(&db).await.unwrap();
    });
}

pub fn update_install_fps_value_by_id(app: &AppHandle, id: String, fps: String) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE install SET 'fps_value' = $1 WHERE id = $2").bind(fps).bind(id);
        query.execute(&db).await.unwrap();
    });
}

pub fn update_install_use_gamemode_by_id(app: &AppHandle, id: String, enabled: bool) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE install SET 'use_gamemode' = $1 WHERE id = $2").bind(enabled).bind(id);
        query.execute(&db).await.unwrap();
    });
}

pub fn update_install_use_mangohud_by_id(app: &AppHandle, id: String, enabled: bool) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE install SET 'use_mangohud' = $1 WHERE id = $2").bind(enabled).bind(id);
        query.execute(&db).await.unwrap();
    });
}

pub fn update_install_env_vars_by_id(app: &AppHandle, id: String, env_vars: String) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE install SET 'env_vars' = $1 WHERE id = $2").bind(env_vars).bind(id);
        query.execute(&db).await.unwrap();
    });
}

pub fn update_install_pre_launch_cmd_by_id(app: &AppHandle, id: String, cmd: String) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE install SET 'pre_launch_command' = $1 WHERE id = $2").bind(cmd).bind(id);
        query.execute(&db).await.unwrap();
    });
}

pub fn update_install_launch_cmd_by_id(app: &AppHandle, id: String, cmd: String) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE install SET 'launch_command' = $1 WHERE id = $2").bind(cmd).bind(id);
        query.execute(&db).await.unwrap();
    });
}

pub fn update_install_prefix_location_by_id(app: &AppHandle, id: String, location: String) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE install SET 'runner_prefix_path' = $1 WHERE id = $2").bind(location).bind(id);
        query.execute(&db).await.unwrap();
    });
}

pub fn update_install_launch_args_by_id(app: &AppHandle, id: String, args: String) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE install SET 'launch_args' = $1 WHERE id = $2").bind(args).bind(id);
        query.execute(&db).await.unwrap();
    });
}

#[allow(dead_code)]
pub fn update_install_runner_version_by_id(app: &AppHandle, id: String, version: String) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE install SET 'runner_version' = $1 WHERE id = $2").bind(version).bind(id);
        query.execute(&db).await.unwrap();
    });
}

#[allow(dead_code)]
pub fn update_install_dxvk_version_by_id(app: &AppHandle, id: String, version: String) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE install SET 'dxvk_version' = $1 WHERE id = $2").bind(version).bind(id);
        query.execute(&db).await.unwrap();
    });
}

pub fn update_install_after_update_by_id(app: &AppHandle, id: String, name: String, icon: String, background: String, version: String) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE install SET 'name' = $1 WHERE id = $2").bind(name).bind(id.clone());
        let query2 = sqlx::query("UPDATE install SET 'game_icon' = $1 WHERE id = $2").bind(icon).bind(id.clone());
        let query3 = sqlx::query("UPDATE install SET 'game_background' = $1 WHERE id = $2").bind(background).bind(id.clone());
        let query4 = sqlx::query("UPDATE install SET 'version' = $1 WHERE id = $2").bind(version).bind(id.clone());

        query.execute(&db).await.unwrap();
        query2.execute(&db).await.unwrap();
        query3.execute(&db).await.unwrap();
        query4.execute(&db).await.unwrap();
    });
}

pub fn update_install_mangohud_config_location_by_id(app: &AppHandle, id: String, location: String) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE install SET 'mangohud_config_path' = $1 WHERE id = $2").bind(location).bind(id);
        query.execute(&db).await.unwrap();
    });
}

pub fn update_install_shortcut_location_by_id(app: &AppHandle, id: String, location: String) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE install SET 'shortcut_path' = $1 WHERE id = $2").bind(location).bind(id);
        query.execute(&db).await.unwrap();
    });
}

#[allow(dead_code)]
pub fn update_install_shortcut_is_steam_by_id(app: &AppHandle, id: String, is_steam: bool) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE install SET 'shortcut_is_steam' = $1 WHERE id = $2").bind(is_steam).bind(id);
        query.execute(&db).await.unwrap();
    });
}

pub fn update_install_xxmi_config_by_id(app: &AppHandle, id: String, xxmi_config: Json<XXMISettings>) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE install SET 'xxmi_config' = $1 WHERE id = $2").bind(xxmi_config).bind(id);
        query.execute(&db).await.unwrap();
    });
}

// === INSTALLED RUNNERS ===

pub fn get_installed_runners(app: &AppHandle) -> Option<Vec<LauncherRunner>> {
    let mut rslt = vec![];

    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("SELECT * FROM installed_runners");
        rslt = query.fetch_all(&db).await.unwrap();
    });

    if rslt.len() >= 1 {
        let mut rsltt = Vec::<LauncherRunner>::new();
        for r in rslt {
            rsltt.push(LauncherRunner {
                id: r.get("id"),
                runner_path: r.get("runner_path"),
                is_installed: r.get("is_installed"),
                version: r.get("version"),
                value: r.get("version"),
                name: r.get("version"),
            })
        }

        Some(rsltt)
    } else {
        None
    }
}

#[allow(dead_code)]
pub fn create_installed_runner(app: &AppHandle, version: String, is_installed: bool, runner_path: String) -> Result<bool, Error> {
    let mut rslt = SqliteQueryResult::default();

    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("INSERT INTO installed_runners(runner_path, is_installed, version, id) VALUES ($1, $2, $3, null)").bind(runner_path).bind(is_installed).bind(version);
        rslt = query.execute(&db).await.unwrap();
    });

    if rslt.rows_affected() >= 1 {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn get_installed_runner_info_by_id(app: &AppHandle, id: String) -> Option<LauncherRunner> {
    let mut rslt = vec![];

    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("SELECT * FROM installed_runners WHERE id = $1").bind(id);
        rslt = query.fetch_all(&db).await.unwrap();
    });

    if rslt.len() >= 1 {
        let rsltt = LauncherRunner {
            id: rslt.get(0).unwrap().get("id"),
            runner_path: rslt.get(0).unwrap().get("runner_path"),
            is_installed: rslt.get(0).unwrap().get("is_installed"),
            version: rslt.get(0).unwrap().get("version"),
            name:  rslt.get(0).unwrap().get("version"),
            value:  rslt.get(0).unwrap().get("version"),
        };

        Some(rsltt)
    } else {
        None
    }
}

pub fn get_installed_runner_info_by_version(app: &AppHandle, version: String) -> Option<LauncherRunner> {
    let mut rslt = vec![];

    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("SELECT * FROM installed_runners WHERE version = $1").bind(version);
        rslt = query.fetch_all(&db).await.unwrap();
    });

    if rslt.len() >= 1 {
        let rsltt = LauncherRunner {
            id: rslt.get(0).unwrap().get("id"),
            runner_path: rslt.get(0).unwrap().get("runner_path"),
            is_installed: rslt.get(0).unwrap().get("is_installed"),
            version: rslt.get(0).unwrap().get("version"),
            name:  rslt.get(0).unwrap().get("version"),
            value:  rslt.get(0).unwrap().get("version"),
        };

        Some(rsltt)
    } else {
        None
    }
}

pub fn update_installed_runner_is_installed_by_version(app: &AppHandle, version: String, is_installed: bool) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE installed_runners SET 'is_installed' = $1 WHERE version = $2").bind(is_installed).bind(version);
        query.execute(&db).await.unwrap();
    });
}

#[allow(dead_code)]
pub fn update_installed_runner_path_by_version(app: &AppHandle, version: String, runner_path: String) {
    run_async_command(async {
        let db = app.state::<DbInstances>().0.lock().await.get("db").unwrap().clone();

        let query = query("UPDATE installed_runners SET 'runner_path' = $1 WHERE version = $2").bind(runner_path).bind(version);
        query.execute(&db).await.unwrap();
    });
}

// === DB RELATED ===

fn add_migrations(db_url: &str, migrations: Vec<Migration>) -> Option<HashMap<String, MigrationList>> {
    let mut migrs: Option<HashMap<String, MigrationList>> = Some(HashMap::new());

    migrs.get_or_insert(Default::default()).insert(db_url.to_string(), MigrationList(migrations));
    migrs
}

#[derive(Default, Debug)]
pub struct DbInstances(pub Mutex<HashMap<String, Pool<Sqlite>>>);

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

impl MigrationSource<'static> for MigrationList {
    fn resolve(self) -> BoxFuture<'static, Result<Vec<SqlxMigration>, BoxDynError>> {
        Box::pin(async move {
            let mut migrations = Vec::new();
            for migration in self.0 {
                if matches!(migration.kind, MigrationKind::Up) {
                    migrations.push(SqlxMigration::new(
                        migration.version,
                        migration.description.into(),
                        migration.kind.into(),
                        migration.sql.into(),
                    ));
                }
            }
            Ok(migrations)
        })
    }
}