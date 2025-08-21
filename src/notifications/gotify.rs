use log::{error, info};
use serde_json::json;
use crate::models::{GithubReleaseInfo, DockerReleaseInfo};

pub async fn send_github_notification(release: &GithubReleaseInfo, token: &str, gotify_url: &str) {
    let client = reqwest::Client::new();
    let app_name = release.repo.split('/').last().unwrap_or(&release.repo);

    let url = format!("{}/message?token={}", gotify_url, token);

    let message = format!(
        "📌 *New version*: {}\n\n📦*For*: {}\n\n📅 *Published on*: {}\n\n📝 *Changelog*:\n\n```{}```\n\n🔗 *Release Url*:{}",
        release.tag_name,
        app_name,
        release.published_at.replace("T", " ").replace("Z", ""),
        release.changelog,
        release.html_url
    );

    let content = json!({
        "title": format!("New version for {}", app_name),
        "message": message,
        "priority": "2"
    });

    match client.post(&url)
        .json(&content)
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            info!("Message sent to Gotify for {}", app_name);
        },
        Ok(response) => {
            error!("Failed to send message to Gotify. Status code: {}", response.status());
        },
        Err(e) => {
            error!("Error sending to Gotify: {}", e);
        }
    }
}

pub async fn send_docker_notification(release: &DockerReleaseInfo, token: &str, gotify_url: &str) {
    let client = reqwest::Client::new();
    let app_name = release.repo.split('/').last().unwrap_or(&release.repo);

    let url = format!("{}/message?token={}", gotify_url, token);

    let message = format!(
        "🐳 *Docker Image Updated!*\n\n🔐 *New Digest*: `{}`\n\n📦 *App*: {}\n\n📢 *Published*: {}\n\n🔗 *Release Url*:{}",
        release.digest,
        app_name,
        release.published_at.replace("T", " ").replace("Z", ""),
        release.html_url
    );

    let content = json!({
        "title": format!("New version for {}", app_name),
        "message": message,
        "priority": "2"
    });

    match client.post(&url)
        .json(&content)
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            info!("Message sent to Gotify for {}", app_name);
        },
        Ok(response) => {
            error!("Failed to send message to Gotify. Status code: {}", response.status());
        },
        Err(e) => {
            error!("Error sending to Gotify: {}", e);
        }
    }
}

pub async fn send_notification(gotify_url: &str, title: &str, message: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    
    // Note: For test notifications, we'll need to get the token from settings
    // This function will be called from the API where we have access to settings
    let url = format!("{}/message", gotify_url);

    let content = json!({
        "title": title,
        "message": message,
        "priority": 5
    });

    let response = client.post(&url)
        .json(&content)
        .send()
        .await?;

    if response.status().is_success() {
        info!("Test notification sent to Gotify successfully");
        Ok(())
    } else {
        let error_msg = format!("Failed to send test notification to Gotify. Status code: {}", response.status());
        error!("{}", error_msg);
        Err(error_msg.into())
    }
}
