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
use tokio::task;

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

    // Load configuration from database, with fallback to environment variables
    let config = config::Config::from_database(&conn_versions);

    // Start the REST API
    start_api();

    let client = reqwest::Client::new();

    // Check if configuration is complete
    if config.auth.is_empty() || (config.ntfy_url.is_none() && config.gotify_url.is_none()
        && config.discord_webhook_url.is_none() && config.slack_webhook_url.is_none()) {
        info!("No notification service is configured.");
        info!("Please configure at least one notification service via the web interface or environment variables.");
        info!("The REST API is still available for configuration.");

        // Continue running to allow configuration through the API
        loop {
            thread::sleep(Duration::from_secs(60));
        }
    }

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