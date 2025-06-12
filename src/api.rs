use log::{error, info};
use rusqlite::{Connection, Result as SqliteResult, params};
use serde_json::json;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{Filter, Reply, Rejection};
use warp::http::StatusCode;
use serde::{Serialize, Deserialize};
use warp::cors::Cors;

#[derive(Debug, Serialize, Deserialize)]
struct RepoRequest {
    repo: String,
}

pub async fn start_api() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Open the database
    let db_path = env::var("DB_PATH").unwrap_or_else(|_| "./data".to_string());
    std::fs::create_dir_all(&db_path).ok();
    let repos_path = format!("{}/watched_repos.db", db_path);

    match Connection::open(&repos_path) {
        Ok(conn) => {
            info!("Database connection established successfully");
            let db = Arc::new(Mutex::new(conn));

            // Route definitions
            let add_github = warp::path("app_repo")
                .and(warp::post())
                .and(warp::body::json())
                .and(with_db(db.clone()))
                .and_then(add_github_repo);

            let add_docker = warp::path("app_docker_repo")
                .and(warp::post())
                .and(warp::body::json())
                .and(with_db(db.clone()))
                .and_then(add_docker_repo);

            let get_github = warp::path("watched_repos")
                .and(warp::get())
                .and(with_db(db.clone()))
                .and_then(get_github_repos);

            let get_docker = warp::path("watched_docker_repos")
                .and(warp::get())
                .and(with_db(db.clone()))
                .and_then(get_docker_repos);

            let delete_github = warp::path("delete_repo")
                .and(warp::post())
                .and(warp::body::json())
                .and(with_db(db.clone()))
                .and_then(delete_github_repo);

            let delete_docker = warp::path("delete_docker_repo")
                .and(warp::post())
                .and(warp::body::json())
                .and(with_db(db.clone()))
                .and_then(delete_docker_repo);

            // Configure CORS
            let cors = warp::cors()
                .allow_any_origin()
                .allow_headers(vec!["Content-Type"])
                .allow_methods(vec!["GET", "POST"]);

            // Combine all routes with CORS
            let routes = add_github
                .or(add_docker)
                .or(get_github)
                .or(get_docker)
                .or(delete_github)
                .or(delete_docker)
                .with(cors);

            // Start the server
            info!("Starting API on 0.0.0.0:5000");
            warp::serve(routes).run(([0, 0, 0, 0], 5000)).await;
            Ok(())
        },
        Err(e) => {
            error!("Unable to open database: {}", e);
            Err(Box::new(e))
        }
    }
}

fn with_db(db: Arc<Mutex<Connection>>) -> impl Filter<Extract = (Arc<Mutex<Connection>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

async fn add_github_repo(body: RepoRequest, db: Arc<Mutex<Connection>>) -> Result<impl Reply, Rejection> {
    let repo = body.repo;

    if repo.is_empty() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&json!({"error": "The 'repo' field is required."})),
            StatusCode::BAD_REQUEST
        ));
    }

    let mut db_guard = db.lock().await;

    // Check if repository already exists
    match db_guard.query_row(
        "SELECT COUNT(*) FROM watched_repos WHERE repo = ?",
        params![repo],
        |row| row.get::<_, i64>(0)
    ) {
        Ok(count) if count > 0 => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({"error": format!("GitHub repository {} is already in the database.", repo)})),
                StatusCode::CONFLICT
            ));
        },
        Err(e) => {
            error!("Error while checking repository: {}", e);
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({"error": format!("Database error: {}", e)})),
                StatusCode::INTERNAL_SERVER_ERROR
            ));
        },
        _ => {}
    }

    // Add the repository
    match db_guard.execute("INSERT INTO watched_repos (repo) VALUES (?)", params![repo]) {
        Ok(_) => {
            Ok(warp::reply::with_status(
                warp::reply::json(&json!({"message": format!("GitHub repository {} has been added to watched repositories.", repo)})),
                StatusCode::OK
            ))
        },
        Err(e) => {
            error!("Error while adding repository: {}", e);
            Ok(warp::reply::with_status(
                warp::reply::json(&json!({"error": format!("Database error: {}", e)})),
                StatusCode::INTERNAL_SERVER_ERROR
            ))
        }
    }
}

async fn add_docker_repo(body: RepoRequest, db: Arc<Mutex<Connection>>) -> Result<impl Reply, Rejection> {
    let repo = body.repo;

    if repo.is_empty() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&json!({"error": "The 'repo' field is required."})),
            StatusCode::BAD_REQUEST
        ));
    }

    let mut db_guard = db.lock().await;

    // Check if repository already exists
    match db_guard.query_row(
        "SELECT COUNT(*) FROM docker_watched_repos WHERE repo = ?",
        params![repo],
        |row| row.get::<_, i64>(0)
    ) {
        Ok(count) if count > 0 => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({"error": format!("Docker repository {} is already in the database.", repo)})),
                StatusCode::CONFLICT
            ));
        },
        Err(e) => {
            error!("Error while checking repository: {}", e);
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({"error": format!("Database error: {}", e)})),
                StatusCode::INTERNAL_SERVER_ERROR
            ));
        },
        _ => {}
    }

    // Add the repository
    match db_guard.execute("INSERT INTO docker_watched_repos (repo) VALUES (?)", params![repo]) {
        Ok(_) => {
            Ok(warp::reply::with_status(
                warp::reply::json(&json!({"message": format!("Docker repository {} has been added to watched repositories.", repo)})),
                StatusCode::OK
            ))
        },
        Err(e) => {
            error!("Error while adding repository: {}", e);
            Ok(warp::reply::with_status(
                warp::reply::json(&json!({"error": format!("Database error: {}", e)})),
                StatusCode::INTERNAL_SERVER_ERROR
            ))
        }
    }
}

async fn get_github_repos(db: Arc<Mutex<Connection>>) -> Result<impl Reply, Rejection> {
    // Solution: collect all results inside the locked block
    let repos = {
        let db_guard = db.lock().await;

        let mut stmt = match db_guard.prepare("SELECT repo FROM watched_repos") {
            Ok(stmt) => stmt,
            Err(e) => {
                error!("Error while preparing query: {}", e);
                return Ok(warp::reply::with_status(
                    warp::reply::json(&json!({"error": format!("Database error: {}", e)})),
                    StatusCode::INTERNAL_SERVER_ERROR
                ));
            }
        };

        let rows = match stmt.query_map([], |row| row.get::<_, String>(0)) {
            Ok(rows) => rows,
            Err(e) => {
                error!("Error while executing query: {}", e);
                return Ok(warp::reply::with_status(
                    warp::reply::json(&json!({"error": format!("Database error: {}", e)})),
                    StatusCode::INTERNAL_SERVER_ERROR
                ));
            }
        };

        let mut repos = Vec::new();
        for row in rows {
            if let Ok(repo) = row {
                repos.push(repo);
            }
        }

        repos
    }; // Lock is released here

    Ok(warp::reply::with_status(
        warp::reply::json(&repos),
        StatusCode::OK
    ))
}

async fn get_docker_repos(db: Arc<Mutex<Connection>>) -> Result<impl Reply, Rejection> {
    // Solution: collect all results inside the locked block
    let repos = {
        let db_guard = db.lock().await;

        let mut stmt = match db_guard.prepare("SELECT repo FROM docker_watched_repos") {
            Ok(stmt) => stmt,
            Err(e) => {
                error!("Error while preparing query: {}", e);
                return Ok(warp::reply::with_status(
                    warp::reply::json(&json!({"error": format!("Database error: {}", e)})),
                    StatusCode::INTERNAL_SERVER_ERROR
                ));
            }
        };

        let rows = match stmt.query_map([], |row| row.get::<_, String>(0)) {
            Ok(rows) => rows,
            Err(e) => {
                error!("Error while executing query: {}", e);
                return Ok(warp::reply::with_status(
                    warp::reply::json(&json!({"error": format!("Database error: {}", e)})),
                    StatusCode::INTERNAL_SERVER_ERROR
                ));
            }
        };

        let mut repos = Vec::new();
        for row in rows {
            if let Ok(repo) = row {
                repos.push(repo);
            }
        }

        repos
    }; // Lock is released here

    Ok(warp::reply::with_status(
        warp::reply::json(&repos),
        StatusCode::OK
    ))
}

async fn delete_github_repo(body: RepoRequest, db: Arc<Mutex<Connection>>) -> Result<impl Reply, Rejection> {
    let repo = body.repo;

    if repo.is_empty() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&json!({"error": "The 'repo' field is required."})),
            StatusCode::BAD_REQUEST
        ));
    }

    let mut db_guard = db.lock().await;

    // Check if repository exists
    match db_guard.query_row(
        "SELECT COUNT(*) FROM watched_repos WHERE repo = ?",
        params![repo],
        |row| row.get::<_, i64>(0)
    ) {
        Ok(count) if count == 0 => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({"error": format!("GitHub repository {} is not in the database.", repo)})),
                StatusCode::NOT_FOUND
            ));
        },
        Err(e) => {
            error!("Error while checking repository: {}", e);
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({"error": format!("Database error: {}", e)})),
                StatusCode::INTERNAL_SERVER_ERROR
            ));
        },
        _ => {}
    }

    // Delete the repository
    match db_guard.execute("DELETE FROM watched_repos WHERE repo = ?", params![repo]) {
        Ok(_) => {
            Ok(warp::reply::with_status(
                warp::reply::json(&json!({"message": format!("GitHub repository {} has been removed from watched repositories.", repo)})),
                StatusCode::OK
            ))
        },
        Err(e) => {
            error!("Error while deleting repository: {}", e);
            Ok(warp::reply::with_status(
                warp::reply::json(&json!({"error": format!("Database error: {}", e)})),
                StatusCode::INTERNAL_SERVER_ERROR
            ))
        }
    }
}

async fn delete_docker_repo(body: RepoRequest, db: Arc<Mutex<Connection>>) -> Result<impl Reply, Rejection> {
    let repo = body.repo;

    if repo.is_empty() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&json!({"error": "The 'repo' field is required."})),
            StatusCode::BAD_REQUEST
        ));
    }

    let mut db_guard = db.lock().await;

    // Check if repository exists
    match db_guard.query_row(
        "SELECT COUNT(*) FROM docker_watched_repos WHERE repo = ?",
        params![repo],
        |row| row.get::<_, i64>(0)
    ) {
        Ok(count) if count == 0 => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({"error": format!("Docker repository {} is not in the database.", repo)})),
                StatusCode::NOT_FOUND
            ));
        },
        Err(e) => {
            error!("Error while checking repository: {}", e);
            return Ok(warp::reply::with_status(
                warp::reply::json(&json!({"error": format!("Database error: {}", e)})),
                StatusCode::INTERNAL_SERVER_ERROR
            ));
        },
        _ => {}
    }

    // Delete the repository
    match db_guard.execute("DELETE FROM docker_watched_repos WHERE repo = ?", params![repo]) {
        Ok(_) => {
            Ok(warp::reply::with_status(
                warp::reply::json(&json!({"message": format!("Docker repository {} has been removed from watched repositories.", repo)})),
                StatusCode::OK
            ))
        },
        Err(e) => {
            error!("Error while deleting repository: {}", e);
            Ok(warp::reply::with_status(
                warp::reply::json(&json!({"error": format!("Database error: {}", e)})),
                StatusCode::INTERNAL_SERVER_ERROR
            ))
        }
    }
}