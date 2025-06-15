use log::info;
pub(crate) use rusqlite::{Connection, Result as SqliteResult, OpenFlags};
use std::env;
use std::path::Path;

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
    let mut stmt = conn.prepare("SELECT * FROM docker_watched_repos")?;
    let repos_iter = stmt.query_map([], |row| Ok(row.get::<_, String>(1)?))?;

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