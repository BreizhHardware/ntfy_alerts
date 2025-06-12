use log::{error, info};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde_json::json;
use crate::models::GithubReleaseInfo;

pub async fn send_to_ntfy(release: GithubReleaseInfo, auth: &str, ntfy_url: &str) {
    let client = reqwest::Client::new();
    let app_name = release.repo.split('/').last().unwrap_or(&release.repo);

    let mut headers = HeaderMap::new();
    headers.insert("Title", HeaderValue::from_str(&format!("New version for {}", app_name))
        .unwrap_or_else(|_| HeaderValue::from_static("")));
    headers.insert("Priority", HeaderValue::from_static("urgent"));
    headers.insert("Markdown", HeaderValue::from_static("yes"));
    headers.insert("Actions", HeaderValue::from_str(&format!("view, Update {}, {}, clear=true", app_name, release.html_url))
        .unwrap_or_else(|_| HeaderValue::from_static("")));

    let message = format!(
        "📌 *New version*: {}\n\n📦*For*: {}\n\n📅 *Published on*: {}\n\n📝 *Changelog*:\n\n```{}```\n\n 🔗 *Release Url*: {}",
        release.tag_name,
        app_name,
        release.published_at.replace("T", " ").replace("Z", ""),
        release.changelog,
        release.html_url
    );

    match client.post(ntfy_url)
        .headers(headers)
        .body(message)
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            info!("Message sent to Ntfy for {}", app_name);
        },
        Ok(response) => {
            error!("Failed to send message to Ntfy. Status code: {}", response.status());
        },
        Err(e) => {
            error!("Error sending to Ntfy: {}", e);
        }
    }
}

pub async fn send_to_gotify(release: GithubReleaseInfo, token: &str, gotify_url: &str) {
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

pub async fn send_to_discord(release: GithubReleaseInfo, webhook_url: &str) {
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

pub async fn send_to_slack(release: GithubReleaseInfo, webhook_url: &str) {
    // Implémentation pour Slack similaire à celle pour Discord
    // ...
}