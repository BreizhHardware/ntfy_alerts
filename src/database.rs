use log::info;
use rusqlite::{Connection, Result as SqliteResult};
use std::env;

pub fn init_databases() -> SqliteResult<(Connection, Connection)> {
    let db_path = env::var("DB_PATH").unwrap_or_else(|_| "/github-ntfy".to_string());
    std::fs::create_dir_all(&db_path).ok();

    let versions_path = format!("{}/ghntfy_versions.db", db_path);
    let repos_path = format!("{}/watched_repos.db", db_path);

    let conn = Connection::open(&versions_path)?;

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

    let conn2 = Connection::open(&repos_path)?;

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
    let mut stmt = conn.prepare("SELECT * FROM watched_repos")?;
    let repos_iter = stmt.query_map([], |row| Ok(row.get::<_, String>(1)?))?;

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