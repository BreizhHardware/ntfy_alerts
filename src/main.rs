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

    let has_env_notification = env_config.ntfy_url.is_some() ||
                               env_config.gotify_url.is_some() ||
                               env_config.discord_webhook_url.is_some() ||
                               env_config.slack_webhook_url.is_some();

    if has_env_notification {
        let now = chrono::Utc::now().to_rfc3339();
        conn_versions.execute(
            "INSERT OR REPLACE INTO app_settings (id, ntfy_url, github_token, docker_username, docker_password,
             gotify_url, gotify_token, discord_webhook_url, slack_webhook_url, check_interval, last_updated)
             VALUES (1, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![
                env_config.ntfy_url,
                env_config.github_token,
                env_config.docker_username,
                env_config.docker_password,
                env_config.gotify_url,
                env_config.gotify_token,
                env_config.discord_webhook_url,
                env_config.slack_webhook_url,
                env_config.timeout as i64,
                now
            ],
        ).map_err(|e| error!("Failed to update app settings in the database: {}", e)).ok();
        info!("Configuration updated from environment variables");
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