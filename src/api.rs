use log::{error, info};
use rusqlite::{Connection, Result as SqliteResult, params};
use serde_json::json;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{Filter, Reply, Rejection};
use warp::http::{StatusCode, header};
use serde::{Serialize, Deserialize};
use warp::cors::Cors;
use chrono::Utc;
use crate::database::{
    get_user_by_username, verify_password, create_user, create_session,
    get_session, delete_session, get_app_settings, update_app_settings
};
use crate::models::{UserLogin, UserRegistration, AuthResponse, ApiResponse, AppSettings};

#[derive(Debug, Serialize, Deserialize)]
struct RepoRequest {
    repo: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateInfo {
    date: String,
    repo: String,
    version: String,
    changelog: String,
}

pub async fn start_api() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Open the database
    let db_path = env::var("DB_PATH").unwrap_or_else(|_| "/github-ntfy".to_string());
    std::fs::create_dir_all(&db_path).ok();
    let repos_path = format!("{}/watched_repos.db", db_path);
    let versions_path = format!("{}/ghntfy_versions.db", db_path);

    match Connection::open(&repos_path) {
        Ok(conn) => {
            info!("Database connection established successfully");
            let db = Arc::new(Mutex::new(conn));

            let versions_conn = match Connection::open(&versions_path) {
                Ok(c) => c,
                Err(e) => {
                    error!("Unable to open versions database: {}", e);
                    return Err(Box::new(e));
                }
            };

            let versions_db = Arc::new(Mutex::new(versions_conn));

            // Route definitions
            let add_github = warp::path("app_github_repo")
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

            let get_updates = warp::path("latest_updates")
                .and(warp::get())
                .and(with_db(db.clone()))
                .and_then(get_latest_updates);

            let login_route = warp::path("auth")
                .and(warp::path("login"))
                .and(warp::post())
                .and(warp::body::json())
                .and(with_db(versions_db.clone()))
                .and_then(login);

            let register_route = warp::path("auth")
                .and(warp::path("register"))
                .and(warp::post())
                .and(warp::body::json())
                .and(with_db(versions_db.clone()))
                .and_then(register);

            let logout_route = warp::path("auth")
                .and(warp::path("logout"))
                .and(warp::post())
                .and(with_auth())
                .and(with_db(versions_db.clone()))
                .and_then(logout);

            let get_settings_route = warp::path("settings")
                .and(warp::get())
                .and(with_db(versions_db.clone()))
                .and(with_auth())
                .and_then(get_settings);

            let update_settings_route = warp::path("settings")
                .and(warp::put())
                .and(warp::body::json())
                .and(with_db(versions_db.clone()))
                .and(with_auth())
                .and_then(update_settings);

            let is_configured_route = warp::path("is_configured")
                .and(warp::get())
                .and(with_db(versions_db.clone()))
                .and_then(is_configured);

            // Configure CORS
            let cors = warp::cors()
                .allow_any_origin()
                .allow_headers(vec!["Content-Type", "Authorization"])
                .allow_methods(vec!["GET", "POST", "PUT", "DELETE"]);

            // Combine all routes with CORS
            let routes = add_github
                .or(add_docker)
                .or(get_github)
                .or(get_docker)
                .or(delete_github)
                .or(delete_docker)
                .or(get_updates)
                .or(login_route)
                .or(register_route)
                .or(logout_route)
                .or(get_settings_route)
                .or(update_settings_route)
                .or(is_configured_route)
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

fn with_auth() -> impl Filter<Extract = (String,), Error = warp::Rejection> + Clone {
    warp::header::<String>("Authorization")
        .map(|header: String| {
            if header.starts_with("Bearer ") {
                header[7..].to_string()
            } else {
                header
            }
        })
        .or_else(|_| async {
            Err(warp::reject::custom(AuthError::MissingToken))
        })
}

#[derive(Debug)]
enum AuthError {
    MissingToken,
}

impl warp::reject::Reject for AuthError {}

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
                warp::reply::json(&json!({"error": "An internal server error occurred."})),
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

async fn get_latest_updates(db: Arc<Mutex<Connection>>) -> Result<impl Reply, Rejection> {
    let updates = {
        let db_guard = db.lock().await;

        let db_path = env::var("DB_PATH").unwrap_or_else(|_| "/github-ntfy".to_string());
        let versions_path = format!("{}/ghntfy_versions.db", db_path);

        match Connection::open(&versions_path) {
            Ok(versions_db) => {
                match versions_db.prepare("SELECT repo, version, changelog, datetime('now') as date FROM versions ORDER BY rowid DESC LIMIT 5") {
                    Ok(mut stmt) => {
                        let rows = match stmt.query_map([], |row| {
                            Ok(UpdateInfo {
                                repo: row.get(0)?,
                                version: row.get(1)?,
                                changelog: row.get(2)?,
                                date: row.get(3)?, 
                            })
                        }) {
                            Ok(rows) => rows,
                            Err(e) => {
                                error!("Error executing query: {}", e);
                                return Ok(warp::reply::with_status(
                                    warp::reply::json(&json!({"error": format!("Database error: {}", e)})),
                                    StatusCode::INTERNAL_SERVER_ERROR
                                ));
                            }
                        };

                        let mut updates = Vec::new();
                        for row in rows {
                            if let Ok(update) = row {
                                updates.push(update);
                            }
                        }

                        if updates.is_empty() {
                            vec![
                                UpdateInfo {
                                    date: Utc::now().to_rfc3339(),
                                    repo: "BreizhHardware/ntfy_alerts".to_string(),
                                    version: "2.0.2".to_string(),
                                    changelog: "- Aucune mise à jour trouvée dans la base de données\n- Ceci est une donnée d'exemple".to_string(),
                                }
                            ]
                        } else {
                            updates
                        }
                    },
                    Err(e) => {
                        error!("Error preparing query: {}", e);
                        vec![
                            UpdateInfo {
                                date: Utc::now().to_rfc3339(),
                                repo: "Erreur".to_string(),
                                version: "N/A".to_string(),
                                changelog: format!("- Erreur lors de la préparation de la requête: {}", e),
                            }
                        ]
                    }
                }
            },
            Err(e) => {
                error!("Error opening versions database: {}", e);
                vec![
                    UpdateInfo {
                        date: Utc::now().to_rfc3339(),
                        repo: "Erreur".to_string(),
                        version: "N/A".to_string(),
                        changelog: format!("- Erreur lors de l'ouverture de la base de données: {}", e),
                    }
                ]
            }
        }
    };

    Ok(warp::reply::with_status(
        warp::reply::json(&updates),
        StatusCode::OK
    ))
}

async fn login(login: UserLogin, db: Arc<Mutex<Connection>>) -> Result<impl Reply, Rejection> {
    let conn = db.lock().await;

    match verify_password(&conn, &login.username, &login.password) {
        Ok(true) => {
            if let Ok(Some(user)) = get_user_by_username(&conn, &login.username) {
                if let Ok(token) = create_session(&conn, user.id) {
                    let auth_response = AuthResponse {
                        token,
                        user: user.clone(),
                    };

                    Ok(warp::reply::with_status(
                        warp::reply::json(&ApiResponse {
                            success: true,
                            message: "Login successful".to_string(),
                            data: Some(auth_response),
                        }),
                        StatusCode::OK,
                    ))
                } else {
                    Ok(warp::reply::with_status(
                        warp::reply::json(&ApiResponse::<()> {
                            success: false,
                            message: "Error creating session".to_string(),
                            data: None,
                        }),
                        StatusCode::INTERNAL_SERVER_ERROR,
                    ))
                }
            } else {
                Ok(warp::reply::with_status(
                    warp::reply::json(&ApiResponse::<()> {
                        success: false,
                        message: "User not found".to_string(),
                        data: None,
                    }),
                    StatusCode::NOT_FOUND,
                ))
            }
        },
        Ok(false) => {
            Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::<()> {
                    success: false,
                    message: "Incorrect username or password".to_string(),
                    data: None,
                }),
                StatusCode::UNAUTHORIZED,
            ))
        },
        Err(_) => {
            Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::<()> {
                    success: false,
                    message: "Internal server error".to_string(),
                    data: None,
                }),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

async fn register(registration: UserRegistration, db: Arc<Mutex<Connection>>) -> Result<impl Reply, Rejection> {
    let conn = db.lock().await;

    // Check if a user already exists with this username
    if let Ok(Some(_)) = get_user_by_username(&conn, &registration.username) {
        return Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()> {
                success: false,
                message: "A user with this name already exists".to_string(),
                data: None,
            }),
            StatusCode::CONFLICT,
        ));
    }

    // Create the new user
    match create_user(&conn, &registration.username, &registration.password, registration.is_admin) {
        Ok(user_id) => {
            if let Ok(Some(user)) = get_user_by_username(&conn, &registration.username) {
                if let Ok(token) = create_session(&conn, user_id) {
                    let auth_response = AuthResponse {
                        token,
                        user,
                    };

                    Ok(warp::reply::with_status(
                        warp::reply::json(&ApiResponse {
                            success: true,
                            message: "Registration successful".to_string(),
                            data: Some(auth_response),
                        }),
                        StatusCode::CREATED,
                    ))
                } else {
                    Ok(warp::reply::with_status(
                        warp::reply::json(&ApiResponse::<()> {
                            success: false,
                            message: "Error creating session".to_string(),
                            data: None,
                        }),
                        StatusCode::INTERNAL_SERVER_ERROR,
                    ))
                }
            } else {
                Ok(warp::reply::with_status(
                    warp::reply::json(&ApiResponse::<()> {
                        success: false,
                        message: "Error retrieving user".to_string(),
                        data: None,
                    }),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ))
            }
        },
        Err(_) => {
            Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::<()> {
                    success: false,
                    message: "Error creating user".to_string(),
                    data: None,
                }),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

async fn logout(token: String, db: Arc<Mutex<Connection>>) -> Result<impl Reply, Rejection> {
    let conn = db.lock().await;

    match delete_session(&conn, &token) {
        Ok(_) => {
            Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::<()> {
                    success: true,
                    message: "Logout successful".to_string(),
                    data: None,
                }),
                StatusCode::OK,
            ))
        },
        Err(_) => {
            Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::<()> {
                    success: false,
                    message: "Error during logout".to_string(),
                    data: None,
                }),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

async fn get_settings(db: Arc<Mutex<Connection>>, token: String) -> Result<impl Reply, Rejection> {
    let conn = db.lock().await;

    // Verify authentication
    if let Ok(Some(session)) = get_session(&conn, &token) {
        if session.expires_at < Utc::now() {
            return Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::<()> {
                    success: false,
                    message: "Session expired".to_string(),
                    data: None,
                }),
                StatusCode::UNAUTHORIZED,
            ));
        }

        // Retrieve settings
        match get_app_settings(&conn) {
            Ok(Some(settings)) => {
                Ok(warp::reply::with_status(
                    warp::reply::json(&ApiResponse {
                        success: true,
                        message: "Settings retrieved successfully".to_string(),
                        data: Some(settings),
                    }),
                    StatusCode::OK,
                ))
            },
            Ok(None) => {
                Ok(warp::reply::with_status(
                    warp::reply::json(&ApiResponse::<()> {
                        success: false,
                        message: "No settings found".to_string(),
                        data: None,
                    }),
                    StatusCode::NOT_FOUND,
                ))
            },
            Err(_) => {
                Ok(warp::reply::with_status(
                    warp::reply::json(&ApiResponse::<()> {
                        success: false,
                        message: "Error retrieving settings".to_string(),
                        data: None,
                    }),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ))
            }
        }
    } else {
        Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()> {
                success: false,
                message: "Unauthorized".to_string(),
                data: None,
            }),
            StatusCode::UNAUTHORIZED,
        ))
    }
}

async fn update_settings(settings: AppSettings, db: Arc<Mutex<Connection>>, token: String) -> Result<impl Reply, Rejection> {
    let conn = db.lock().await;

    // Verify authentication
    if let Ok(Some(session)) = get_session(&conn, &token) {
        if session.expires_at < Utc::now() {
            return Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::<()> {
                    success: false,
                    message: "Session expired".to_string(),
                    data: None,
                }),
                StatusCode::UNAUTHORIZED,
            ));
        }

        // Update settings
        match update_app_settings(&conn, &settings) {
            Ok(_) => {
                Ok(warp::reply::with_status(
                    warp::reply::json(&ApiResponse::<()> {
                        success: true,
                        message: "Settings updated successfully".to_string(),
                        data: None,
                    }),
                    StatusCode::OK,
                ))
            },
            Err(_) => {
                Ok(warp::reply::with_status(
                    warp::reply::json(&ApiResponse::<()> {
                        success: false,
                        message: "Error updating settings".to_string(),
                        data: None,
                    }),
                    StatusCode::INTERNAL_SERVER_ERROR,
                ))
            }
        }
    } else {
        Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()> {
                success: false,
                message: "Unauthorized".to_string(),
                data: None,
            }),
            StatusCode::UNAUTHORIZED,
        ))
    }
}

// Function to check if the application is configured
async fn is_configured(db: Arc<Mutex<Connection>>) -> Result<impl Reply, Rejection> {
    let conn = db.lock().await;

    // Check if at least one admin user exists
    let admin_exists = match conn.query_row(
        "SELECT COUNT(*) FROM users WHERE is_admin = 1",
        [],
        |row| row.get::<_, i64>(0)
    ) {
        Ok(count) => count > 0,
        Err(_) => false,
    };

    // Check if settings are configured
    let settings_exist = match get_app_settings(&conn) {
        Ok(Some(settings)) => {
            // Check if at least one notification service is configured
            settings.ntfy_url.is_some() ||
            settings.discord_webhook_url.is_some() ||
            settings.slack_webhook_url.is_some() ||
            settings.gotify_url.is_some()
        },
        _ => false,
    };

    Ok(warp::reply::with_status(
        warp::reply::json(&ApiResponse {
            success: true,
            message: "Configuration status retrieved".to_string(),
            data: Some(json!({
                "configured": admin_exists && settings_exist,
                "admin_exists": admin_exists,
                "settings_exist": settings_exist
            })),
        }),
        StatusCode::OK,
    ))
}
