use log::{error, info};
use serde_json::json;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use std::iter::FromIterator;
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
            "📌 *New version*: {}\n\n📦*For*: {}\n\n📅 *Published on*: {}\n\n📝 *Changelog*:\n\n `truncated..` use 🔗 instead",
            release.tag_name,
            app_name,
            release.published_at.replace("T", " ").replace("Z", "")
        );
    }

    let data = json!({
        "blocks": [
            {
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": message
                },
                "accessory": {
                    "type": "button",
                    "text": {
                        "type": "plain_text",
                        "text": "View Release"
                    },
                    "url": release.html_url,
                    "action_id": "button-action"
                }
            },
            {
                "type": "divider"
            }
        ]
    });

    let headers = HeaderMap::from_iter([(
        CONTENT_TYPE,
        HeaderValue::from_static("application/json")
    )]);

    match client.post(webhook_url)
        .headers(headers)
        .json(&data)
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            info!("Message sent to Slack for {}", app_name);
        },
        Ok(response) => {
            error!("Failed to send message to Slack. Status code: {}", response.status());
        },
        Err(e) => {
            error!("Error sending to Slack: {}", e);
        }
    }
}

pub async fn send_docker_notification(release: &DockerReleaseInfo, webhook_url: &str) {
    let client = reqwest::Client::new();
    let app_name = release.repo.split('/').last().unwrap_or(&release.repo);

    let message = format!(
        "🐳 *Docker Image Updated!*\n\n🔐 *New Digest*: `{}`\n\n📦 *App*: {}\n\n📢*Published*: {}",
        release.digest,
        app_name,
        release.published_at.replace("T", " ").replace("Z", "")
    );

    let data = json!({
        "blocks": [
            {
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": message
                },
                "accessory": {
                    "type": "button",
                    "text": {
                        "type": "plain_text",
                        "text": "View Image"
                    },
                    "url": release.html_url,
                    "action_id": "button-action"
                }
            },
            {
                "type": "divider"
            }
        ]
    });

    let headers = HeaderMap::from_iter([(
        CONTENT_TYPE,
        HeaderValue::from_static("application/json")
    )]);

    match client.post(webhook_url)
        .headers(headers)
        .json(&data)
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            info!("Message sent to Slack for {}", app_name);
        },
        Ok(response) => {
            error!("Failed to send message to Slack. Status code: {}", response.status());
        },
        Err(e) => {
            error!("Error sending to Slack: {}", e);
        }
    }
}