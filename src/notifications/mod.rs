pub mod ntfy;
pub mod gotify;
pub mod discord;
pub mod slack;
pub mod github;
pub mod docker;

use tokio::task;
use crate::models::{GithubReleaseInfo, DockerReleaseInfo};
use crate::config::Config;
use crate::database::{Connection, is_new_version, update_version};
use rusqlite::Result as SqliteResult;

pub async fn send_notifications(
    github_releases: Vec<GithubReleaseInfo>,
    docker_releases: Vec<DockerReleaseInfo>,
    config: &Config,
    db_conn: &Connection,
) -> SqliteResult<()> {
    let mut tasks = Vec::new();

    // Create tasks for GitHub notifications
    for release in &github_releases {
        if is_new_version(db_conn, &release.repo, &release.tag_name)? {
            if let Some(url) = &config.ntfy_url {
                let release = release.clone();
                let auth = config.auth.clone();
                let url = url.clone();
                tasks.push(task::spawn(async move {
                    github::send_to_ntfy(release, &auth, &url).await;
                }));
            }

            if let (Some(gotify_url), Some(gotify_token)) = (&config.gotify_url, &config.gotify_token) {
                let release = release.clone();
                let url = gotify_url.clone();
                let token = gotify_token.clone();
                tasks.push(task::spawn(async move {
                    github::send_to_gotify(release, &token, &url).await;
                }));
            }

            if let Some(discord_url) = &config.discord_webhook_url {
                let release = release.clone();
                let url = discord_url.clone();
                tasks.push(task::spawn(async move {
                    github::send_to_discord(release, &url).await;
                }));
            }

            if let Some(slack_url) = &config.slack_webhook_url {
                let release = release.clone();
                let url = slack_url.clone();
                tasks.push(task::spawn(async move {
                    github::send_to_slack(release, &url).await;
                }));
            }

            update_version(db_conn, &release.repo, &release.tag_name, Some(release.changelog.as_str()))?;
        }
    }

    for release in &docker_releases {
        if is_new_version(db_conn, &release.repo, &release.digest)? {
            if let Some(url) = &config.ntfy_url {
                let release = release.clone();
                let auth = config.auth.clone();
                let url = url.clone();
                tasks.push(task::spawn(async move {
                    docker::send_to_ntfy(release, &auth, &url).await;
                }));
            }

            if let (Some(gotify_url), Some(gotify_token)) = (&config.gotify_url, &config.gotify_token) {
                let release = release.clone();
                let url = gotify_url.clone();
                let token = gotify_token.clone();
                tasks.push(task::spawn(async move {
                    docker::send_to_gotify(release, &token, &url).await;
                }));
            }

            if let Some(discord_url) = &config.discord_webhook_url {
                let release = release.clone();
                let url = discord_url.clone();
                tasks.push(task::spawn(async move {
                    docker::send_to_discord(release, &url).await;
                }));
            }

            if let Some(slack_url) = &config.slack_webhook_url {
                let release = release.clone();
                let url = slack_url.clone();
                tasks.push(task::spawn(async move {
                    docker::send_to_slack(release, &url).await;
                }));
            }

            update_version(db_conn, &release.repo, &release.digest, None)?;
        }
    }

    // Wait for all tasks to complete
    for task in tasks {
        let _ = task.await;
    }

    Ok(())
}