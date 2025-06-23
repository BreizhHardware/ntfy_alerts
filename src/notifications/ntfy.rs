use log::{error, info};
use reqwest::header::{HeaderMap, HeaderValue};
use crate::models::{GithubReleaseInfo, DockerReleaseInfo};

pub async fn send_github_notification(release: &GithubReleaseInfo, auth: &str, ntfy_url: &str) {
    let client = reqwest::Client::new();
    let app_name = release.repo.split('/').last().unwrap_or(&release.repo);

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_str(&format!("Basic {}", auth))
        .unwrap_or_else(|_| HeaderValue::from_static("")));
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

pub async fn send_docker_notification(release: &DockerReleaseInfo, auth: &str, ntfy_url: &str) {
    let client = reqwest::Client::new();
    let app_name = release.repo.split('/').last().unwrap_or(&release.repo);

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", HeaderValue::from_str(&format!("Basic {}", auth))
        .unwrap_or_else(|_| HeaderValue::from_static("")));
    headers.insert("Title", HeaderValue::from_str(&format!("🆕 New version for {}", app_name))
        .unwrap_or_else(|_| HeaderValue::from_static("")));
    headers.insert("Priority", HeaderValue::from_static("urgent"));
    headers.insert("Markdown", HeaderValue::from_static("yes"));
    headers.insert("Actions", HeaderValue::from_str(&format!("View, Update {}, {}, clear=true", app_name, release.html_url))
        .unwrap_or_else(|_| HeaderValue::from_static("")));

    let message = format!(
        "🐳 *Docker Image Updated!*\n\n🔐 *New Digest*: `{}`\n\n📦 *App*: {}\n\n📢 *Published*: {}\n\n 🔗 *Release Url*: {}",
        release.digest,
        app_name,
        release.published_at.replace("T", " ").replace("Z", ""),
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