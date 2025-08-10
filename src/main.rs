mod config;
mod models;
mod database;
mod github;
mod docker;
mod notifications;
mod api;

use log::{error, info};
use std::thread;
use std::time::Duration;

// Function to start the API in a separate thread
fn start_api() {
    std::thread::spawn(|| {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            match api::start_api().await {
                Ok(_) => info!("API closed correctly"),
                Err(e) => error!("API error: {}", e),
            }
        });
    });
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // Initialize databases
    let (conn_versions, conn_repos) = database::init_databases()?;

    // Load environment variables
    let env_config = config::Config::from_env();

    // Only update database with env vars if they are explicitly set
    // We check each field individually instead of overwriting everything
    let has_env_notification = env_config.ntfy_url.is_some() ||
                               env_config.gotify_url.is_some() ||
                               env_config.discord_webhook_url.is_some() ||
                               env_config.slack_webhook_url.is_some();

    if has_env_notification {
        let now = chrono::Utc::now().to_rfc3339();

        // First, ensure there's a record in the database
        conn_versions.execute(
            "INSERT OR IGNORE INTO app_settings (id, last_updated) VALUES (1, ?)",
            rusqlite::params![now],
        ).map_err(|e| error!("Failed to initialize app settings: {}", e)).ok();

        // Then update only the fields that are set in environment variables
        if let Some(ntfy_url) = &env_config.ntfy_url {
            conn_versions.execute(
                "UPDATE app_settings SET ntfy_url = ?, last_updated = ? WHERE id = 1",
                rusqlite::params![ntfy_url, now],
            ).ok();
        }

        if let Some(github_token) = &env_config.github_token {
            conn_versions.execute(
                "UPDATE app_settings SET github_token = ?, last_updated = ? WHERE id = 1",
                rusqlite::params![github_token, now],
            ).ok();
        }

        if let Some(docker_username) = &env_config.docker_username {
            conn_versions.execute(
                "UPDATE app_settings SET docker_username = ?, last_updated = ? WHERE id = 1",
                rusqlite::params![docker_username, now],
            ).ok();
        }

        if let Some(docker_password) = &env_config.docker_password {
            conn_versions.execute(
                "UPDATE app_settings SET docker_password = ?, last_updated = ? WHERE id = 1",
                rusqlite::params![docker_password, now],
            ).ok();
        }

        if let Some(gotify_url) = &env_config.gotify_url {
            conn_versions.execute(
                "UPDATE app_settings SET gotify_url = ?, last_updated = ? WHERE id = 1",
                rusqlite::params![gotify_url, now],
            ).ok();
        }

        if let Some(gotify_token) = &env_config.gotify_token {
            conn_versions.execute(
                "UPDATE app_settings SET gotify_token = ?, last_updated = ? WHERE id = 1",
                rusqlite::params![gotify_token, now],
            ).ok();
        }

        if let Some(discord_webhook_url) = &env_config.discord_webhook_url {
            conn_versions.execute(
                "UPDATE app_settings SET discord_webhook_url = ?, last_updated = ? WHERE id = 1",
                rusqlite::params![discord_webhook_url, now],
            ).ok();
        }

        if let Some(slack_webhook_url) = &env_config.slack_webhook_url {
            conn_versions.execute(
                "UPDATE app_settings SET slack_webhook_url = ?, last_updated = ? WHERE id = 1",
                rusqlite::params![slack_webhook_url, now],
            ).ok();
        }

        conn_versions.execute(
            "UPDATE app_settings SET check_interval = ?, last_updated = ? WHERE id = 1",
            rusqlite::params![env_config.timeout as i64, now],
        ).ok();

        info!("Configuration updated from environment variables (selective update)");
    }

    // Load configuration from database, with fallback to environment variables
    let config = config::Config::from_database(&conn_versions);

    // Check if configuration is complete
    let config_is_incomplete = config.auth.is_empty() || (config.ntfy_url.is_none() && config.gotify_url.is_none()
        && config.discord_webhook_url.is_none() && config.slack_webhook_url.is_none());

    let client = reqwest::Client::new();

    // Now handle incomplete configuration
    if config_is_incomplete {
        info!("No notification service is configured.");
        info!("Please configure at least one notification service via the web interface or environment variables.");
        info!("Starting the REST API for configuration.");

        // Start the REST API only if configuration is incomplete
        start_api();

        // Continue running to allow configuration through the API
        loop {
            thread::sleep(Duration::from_secs(60));
        }
    }

    // Start the REST API only if configuration is complete
    start_api();

    info!("Starting version monitoring...");

    loop {
        let github_repos = database::get_watched_repos(&conn_repos)?;
        let docker_repos = database::get_docker_watched_repos(&conn_repos)?;

        let github_releases = github::get_latest_releases(&github_repos, &client, config.github_headers()).await;
        let docker_releases = docker::get_latest_docker_releases(&docker_repos, &client, config.docker_headers()).await;

        notifications::send_notifications(github_releases, docker_releases, &config, &conn_versions).await;

        tokio::time::sleep(Duration::from_secs_f64(config.timeout)).await;
    }
}