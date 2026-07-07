use log::{error, info};
use serde_json::json;
use reqwest::header::HeaderMap;
use crate::models::{GithubReleaseInfo, DockerReleaseInfo};

pub async fn send_github_notification(release: &GithubReleaseInfo, webhook_url: &str) {
    let client = reqwest::Client::new();
    let app_name = release.repo.split('/').last().unwrap_or(&release.repo);

    let mut message = format!(
        "📌 *New version*: {}\n\n📦*For*: {}\n\n📅 *Published on*: {}\n\n📝 *Changelog*:\n\n```{}```",
        release.tag_name,
        app_name,
        release.published_at.replace("T", " ").replace("Z", ""),
        release.changelog
    );

    if message.len() > 2000 {
        message = format!(
            "📌 *New version*: {}\n\n📦*For*: {}\n\n📅 *Published on*: {}\n\n🔗 *Release Link*: {}",
            release.tag_name,
            app_name,
            release.published_at.replace("T", " ").replace("Z", ""),
            release.html_url
        );
    }

    let data = json!({
        "content": message,
        "username": "GitHub Ntfy"
    });

    let headers = HeaderMap::new();

    match client.post(webhook_url)
        .headers(headers)
        .json(&data)
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            info!("Message sent to Discord for {}", app_name);
        },
        Ok(response) => {
            error!("Failed to send message to Discord. Status code: {}", response.status());
        },
        Err(e) => {
            error!("Error sending to Discord: {}", e);
        }
    }
}

pub async fn send_docker_notification(release: &DockerReleaseInfo, webhook_url: &str) {
    let client = reqwest::Client::new();
    let app_name = release.repo.split('/').last().unwrap_or(&release.repo);

    let message = format!(
        "🐳 *Docker Image Updated!*\n\n🔐 *New Digest*: `{}`\n\n📦 *App*: {}\n\n📢 *Published*: {}\n\n🔗 *Link*: {}",
        release.digest,
        app_name,
        release.published_at.replace("T", " ").replace("Z", ""),
        release.html_url
    );

    let data = json!({
        "content": message,
        "username": "GitHub Ntfy"
    });

    match client.post(webhook_url)
        .json(&data)
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            info!("Message sent to Discord for {}", app_name);
        },
        Ok(response) => {
            error!("Failed to send message to Discord. Status code: {}", response.status());
        },
        Err(e) => {
            error!("Error sending to Discord: {}", e);
        }
    }
}

pub async fn send_notification(webhook_url: &str, title: &str, message: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();

    let data = json!({
        "content": format!("**{}**\n{}", title, message),
        "username": "GitHub Ntfy Test"
    });

    let response = client.post(webhook_url)
        .header("Content-Type", "application/json")
        .json(&data)
        .send()
        .await?;

    if response.status().is_success() {
        info!("Test notification sent to Discord successfully");
        Ok(())
    } else {
        let error_msg = format!("Failed to send test notification to Discord. Status code: {}", response.status());
        error!("{}", error_msg);
        Err(error_msg.into())
    }
}
