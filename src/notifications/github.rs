use tokio::task;
use crate::models::GithubReleaseInfo;
use crate::config::Config;
use crate::notifications::{ntfy, gotify, discord, slack};

pub async fn send_to_ntfy(release: GithubReleaseInfo, auth: &str, ntfy_url: &str) {
    ntfy::send_github_notification(&release, auth, ntfy_url).await;
}

pub async fn send_to_gotify(release: GithubReleaseInfo, token: &str, gotify_url: &str) {
    gotify::send_github_notification(&release, token, gotify_url).await;
}

pub async fn send_to_discord(release: GithubReleaseInfo, webhook_url: &str) {
    discord::send_github_notification(&release, webhook_url).await;
}

pub async fn send_to_slack(release: GithubReleaseInfo, webhook_url: &str) {
    slack::send_github_notification(&release, webhook_url).await;
}

pub async fn send_notifications(releases: &[GithubReleaseInfo], config: &Config) {
    let mut tasks = Vec::new();

    for release in releases {
        // Send to Ntfy
        if let Some(url) = &config.ntfy_url {
            let release_clone = release.clone();
            let auth = config.auth.clone();
            let url_clone = url.clone();
            tasks.push(task::spawn(async move {
                send_to_ntfy(release_clone, &auth, &url_clone).await;
            }));
        }

        // Send to Gotify
        if let (Some(gotify_url), Some(gotify_token)) = (&config.gotify_url, &config.gotify_token) {
            let release_clone = release.clone();
            let token = gotify_token.clone();
            let url = gotify_url.clone();
            tasks.push(task::spawn(async move {
                send_to_gotify(release_clone, &token, &url).await;
            }));
        }

        // Send to Discord
        if let Some(discord_url) = &config.discord_webhook_url {
            let release_clone = release.clone();
            let url = discord_url.clone();
            tasks.push(task::spawn(async move {
                send_to_discord(release_clone, &url).await;
            }));
        }

        // Send to Slack
        if let Some(slack_url) = &config.slack_webhook_url {
            let release_clone = release.clone();
            let url = slack_url.clone();
            tasks.push(task::spawn(async move {
                send_to_slack(release_clone, &url).await;
            }));
        }
    }

    // Wait for all tasks to complete
    for task in tasks {
        let _ = task.await;
    }
}