use log::info;
pub(crate) use rusqlite::{Connection, Result as SqliteResult, OpenFlags, Error as SqliteError};
use std::env;
use chrono::Utc;
use rand::Rng;
use bcrypt::{hash, verify, DEFAULT_COST};
use crate::models::{User, Session, AppSettings};

pub fn init_databases() -> SqliteResult<(Connection, Connection)> {
    let db_path = env::var("DB_PATH").unwrap_or_else(|_| "/github-ntfy".to_string());

    if let Err(e) = std::fs::create_dir_all(&db_path) {
        info!("Error while creating directory {}: {}", db_path, e);
    }

    let versions_path = format!("{}/ghntfy_versions.db", db_path);
    let repos_path = format!("{}/watched_repos.db", db_path);

    let conn = Connection::open_with_flags(&versions_path, OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_URI)?;

    info!("Database open at {}", versions_path);

    conn.execute(
        "CREATE TABLE IF NOT EXISTS versions (
            repo TEXT PRIMARY KEY,
            version TEXT,
            changelog TEXT
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS docker_versions (
            repo TEXT PRIMARY KEY,
            digest TEXT
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            is_admin INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            token TEXT PRIMARY KEY,
            user_id INTEGER NOT NULL,
            expires_at TEXT NOT NULL,
            FOREIGN KEY (user_id) REFERENCES users(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_settings (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            ntfy_url TEXT,
            github_token TEXT,
            docker_username TEXT,
            docker_password TEXT,
            gotify_url TEXT,
            gotify_token TEXT,
            discord_webhook_url TEXT,
            slack_webhook_url TEXT,
            check_interval INTEGER DEFAULT 3600,
            last_updated TEXT NOT NULL
        )",
        [],
    )?;

    let admin_exists = conn
        .query_row("SELECT COUNT(*) FROM users WHERE is_admin = 1", [], |row| {
            row.get::<_, i64>(0)
        })
        .unwrap_or(0);

    if admin_exists == 0 {
        if let (Ok(username), Ok(password)) = (env::var("USERNAME"), env::var("PASSWORD")) {
            if !username.is_empty() && !password.is_empty() {
                let hashed_password = hash(password, DEFAULT_COST).unwrap_or_else(|_| String::new());
                let now = Utc::now().to_rfc3339();
                if let Err(e) = conn.execute(
                    "INSERT INTO users (username, password_hash, is_admin, created_at) VALUES (?, ?, 1, ?)",
                    &[&username, &hashed_password, &now],
                ) {
                    info!("Erreur lors de la création de l'utilisateur admin: {}", e);
                } else {
                    info!("Utilisateur admin créé avec succès depuis les variables d'environnement");
                }
            }
        }
    }

    let settings_exist = conn
        .query_row("SELECT COUNT(*) FROM app_settings", [], |row| {
            row.get::<_, i64>(0)
        })
        .unwrap_or(0);

    if settings_exist == 0 {
        let ntfy_url = env::var("NTFY_URL").ok();
        let github_token = env::var("GHNTFY_TOKEN").ok();
        let docker_username = env::var("DOCKER_USERNAME").ok();
        let docker_password = env::var("DOCKER_PASSWORD").ok();
        let gotify_url = env::var("GOTIFY_URL").ok();
        let gotify_token = env::var("GOTIFY_TOKEN").ok();
        let discord_webhook_url = env::var("DISCORD_WEBHOOK_URL").ok();
        let slack_webhook_url = env::var("SLACK_WEBHOOK_URL").ok();
        let check_interval = env::var("GHNTFY_TIMEOUT")
            .ok()
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(3600);
        let now = Utc::now().to_rfc3339();

        if let Err(e) = conn.execute(
            "INSERT INTO app_settings (id, ntfy_url, github_token, docker_username, docker_password, gotify_url, gotify_token, discord_webhook_url, slack_webhook_url, check_interval, last_updated)
             VALUES (1, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                ntfy_url,
                github_token,
                docker_username,
                docker_password,
                gotify_url,
                gotify_token,
                discord_webhook_url,
                slack_webhook_url,
                check_interval,
                now
            ],
        ) {
            info!("Erreur lors de l'initialisation des paramètres: {}", e);
        } else {
            info!("Paramètres initialisés avec succès depuis les variables d'environnement");
        }
    }

    let conn2 = Connection::open_with_flags(&repos_path, OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_URI)?;

    info!("Database open at {}", repos_path);

    conn2.execute(
        "CREATE TABLE IF NOT EXISTS watched_repos (
            id INTEGER PRIMARY KEY,
            repo TEXT
        )",
        [],
    )?;

    conn2.execute(
        "CREATE TABLE IF NOT EXISTS docker_watched_repos (
            id INTEGER PRIMARY KEY,
            repo TEXT
        )",
        [],
    )?;

    Ok((conn, conn2))
}

// Functions to retrieve watched repositories
pub fn get_watched_repos(conn: &Connection) -> SqliteResult<Vec<String>> {
    let mut stmt = conn.prepare("SELECT repo FROM watched_repos")?;
    let repos_iter = stmt.query_map([], |row| Ok(row.get::<_, String>(0)?))?;

    let mut repos = Vec::new();
    for repo in repos_iter {
        repos.push(repo?);
    }
    Ok(repos)
}

pub fn get_docker_watched_repos(conn: &Connection) -> SqliteResult<Vec<String>> {
    let mut stmt = conn.prepare("SELECT repo FROM docker_watched_repos")?;
    let repos_iter = stmt.query_map([], |row| Ok(row.get::<_, String>(0)?))?;

    let mut repos = Vec::new();
    for repo in repos_iter {
        repos.push(repo?);
    }
    Ok(repos)
}

pub fn is_new_version(conn: &Connection, repo: &str, version: &str) -> SqliteResult<bool> {
    let mut stmt = conn.prepare("SELECT version FROM versions WHERE repo = ?")?;
    let result = stmt.query_map([repo], |row| row.get::<_, String>(0))?;

    for stored_version in result {
        if let Ok(v) = stored_version {
            return Ok(v != version);
        }
    }

    Ok(true)
}

pub fn update_version(conn: &Connection, repo: &str, version: &str, changelog: Option<&str>) -> SqliteResult<()> {
    conn.execute(
        "REPLACE INTO versions (repo, version, changelog) VALUES (?, ?, ?)",
        [repo, version, changelog.unwrap_or("")],
    )?;

    Ok(())
}

pub fn create_user(conn: &Connection, username: &str, password: &str, is_admin: bool) -> SqliteResult<i64> {
    let hashed_password = hash(password, DEFAULT_COST).map_err(|e| {
        SqliteError::SqliteFailure(
            rusqlite::ffi::Error::new(1), // Code d'erreur personnalisé
            Some(e.to_string())
        )
    })?;

    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO users (username, password_hash, is_admin, created_at) VALUES (?, ?, ?, ?)",
        &[username, &hashed_password, &(if is_admin { 1 } else { 0 }).to_string(), &now],
    )?;

    Ok(conn.last_insert_rowid())
}

pub fn get_user_by_username(conn: &Connection, username: &str) -> SqliteResult<Option<User>> {
    let mut stmt = conn.prepare("SELECT id, username, password_hash, is_admin, created_at FROM users WHERE username = ?")?;
    let mut rows = stmt.query(&[username])?;

    if let Some(row) = rows.next()? {
        let id = row.get(0)?;
        let username = row.get(1)?;
        let password_hash = row.get(2)?;
        let is_admin: i64 = row.get(3)?;
        let created_at_str: String = row.get(4)?;
        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| {
                SqliteError::SqliteFailure(
                    rusqlite::ffi::Error::new(1),
                    Some(e.to_string())
                )
            })?;

        Ok(Some(User {
            id,
            username,
            password_hash,
            is_admin: is_admin == 1,
            created_at,
        }))
    } else {
        Ok(None)
    }
}

pub fn verify_password(conn: &Connection, username: &str, password: &str) -> SqliteResult<bool> {
    if let Some(user) = get_user_by_username(conn, username)? {
        Ok(verify(password, &user.password_hash).unwrap_or(false))
    } else {
        Ok(false)
    }
}

pub fn create_session(conn: &Connection, user_id: i64) -> SqliteResult<String> {
    let token = generate_session_token();
    let expires_at = Utc::now() + chrono::Duration::days(7);
    let expires_at_str = expires_at.to_rfc3339();

    conn.execute(
        "INSERT INTO sessions (token, user_id, expires_at) VALUES (?, ?, ?)",
        &[&token, &user_id.to_string(), &expires_at_str],
    )?;

    Ok(token)
}

pub fn get_session(conn: &Connection, token: &str) -> SqliteResult<Option<Session>> {
    let mut stmt = conn.prepare("SELECT token, user_id, expires_at FROM sessions WHERE token = ?")?;
    let mut rows = stmt.query(&[token])?;

    if let Some(row) = rows.next()? {
        let token = row.get(0)?;
        let user_id = row.get(1)?;
        let expires_at_str: String = row.get(2)?;
        let expires_at = chrono::DateTime::parse_from_rfc3339(&expires_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| {
                SqliteError::SqliteFailure(
                    rusqlite::ffi::Error::new(1),
                    Some(e.to_string())
                )
            })?;

        Ok(Some(Session {
            token,
            user_id,
            expires_at,
        }))
    } else {
        Ok(None)
    }
}

pub fn delete_session(conn: &Connection, token: &str) -> SqliteResult<()> {
    conn.execute(
        "DELETE FROM sessions WHERE token = ?",
        &[token],
    )?;

    Ok(())
}

pub fn get_app_settings(conn: &Connection) -> SqliteResult<Option<AppSettings>> {
    let mut stmt = conn.prepare(
        "SELECT id, ntfy_url, github_token, docker_username, docker_password,
                gotify_url, gotify_token, discord_webhook_url, slack_webhook_url,
                check_interval, last_updated
         FROM app_settings
         WHERE id = 1"
    )?;

    let mut rows = stmt.query([])?;

    if let Some(row) = rows.next()? {
        let id = row.get(0)?;
        let ntfy_url = row.get(1)?;
        let github_token = row.get(2)?;
        let docker_username = row.get(3)?;
        let docker_password = row.get(4)?;
        let gotify_url = row.get(5)?;
        let gotify_token = row.get(6)?;
        let discord_webhook_url = row.get(7)?;
        let slack_webhook_url = row.get(8)?;
        let check_interval = row.get(9)?;
        let last_updated_str: String = row.get(10)?;
        let last_updated = chrono::DateTime::parse_from_rfc3339(&last_updated_str)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| {
                SqliteError::SqliteFailure(
                    rusqlite::ffi::Error::new(1),
                    Some(e.to_string())
                )
            })?;

        Ok(Some(AppSettings {
            id: Some(id),
            ntfy_url,
            github_token,
            docker_username,
            docker_password,
            gotify_url,
            gotify_token,
            discord_webhook_url,
            slack_webhook_url,
            check_interval,
            last_updated,
        }))
    } else {
        Ok(None)
    }
}

pub fn update_app_settings(conn: &Connection, settings: &AppSettings) -> SqliteResult<()> {
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE app_settings
         SET ntfy_url = ?, github_token = ?, docker_username = ?, docker_password = ?,
             gotify_url = ?, gotify_token = ?, discord_webhook_url = ?, slack_webhook_url = ?,
             check_interval = ?, last_updated = ?
         WHERE id = 1",
        rusqlite::params![
            settings.ntfy_url,
            settings.github_token,
            settings.docker_username,
            settings.docker_password,
            settings.gotify_url,
            settings.gotify_token,
            settings.discord_webhook_url,
            settings.slack_webhook_url,
            settings.check_interval,
            now
        ],
    )?;

    Ok(())
}

fn generate_session_token() -> String {
    let mut rng = rand::thread_rng();
    let token_bytes: Vec<u8> = (0..32).map(|_| rng.gen::<u8>()).collect();

    // Convertir en hexadécimal
    token_bytes.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<String>>()
        .join("")
}
