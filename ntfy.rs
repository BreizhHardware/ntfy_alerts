use dotenv::dotenv;
use log::{error, info};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use rusqlite::{Connection, Result as SqliteResult};
use serde::{Deserialize};
use serde_json::json;
use std::env;
use std::fs::File;
use std::io::Read;
use std::thread;
use std::time::Duration;
use tokio::task;
mod api;

// Structures for GitHub data
#[derive(Debug, Deserialize, Clone)]
struct GithubRelease {
    name: String,
    tag_name: String,
    html_url: String,
    published_at: Option<String>,
    body: Option<String>,
}

#[derive(Debug, Clone)]
struct GithubReleaseInfo {
    repo: String,
    name: String,
    tag_name: String,
    html_url: String,
    changelog: String,
    published_at: String,
}

// Structures for Docker data
#[derive(Debug, Deserialize)]
struct DockerTag {
    digest: String,
    last_updated: String,
}

#[derive(Debug, Clone)]
struct DockerReleaseInfo {
    repo: String,
    digest: String,
    html_url: String,
    published_at: String,
}

// Configuration
struct Config {
    github_token: Option<String>,
    docker_username: Option<String>,
    docker_password: Option<String>,
    docker_token: Option<String>,
    ntfy_url: Option<String>,
    gotify_url: Option<String>,
    gotify_token: Option<String>,
    discord_webhook_url: Option<String>,
    slack_webhook_url: Option<String>,
    auth: String,
    timeout: f64,
}

impl Config {
    fn from_env() -> Self {
        dotenv().ok();

        let docker_username = env::var("DOCKER_USERNAME").ok();
        let docker_password = env::var("DOCKER_PASSWORD").ok();
        let docker_token = if let (Some(username), Some(password)) = (&docker_username, &docker_password) {
            create_dockerhub_token(username, password)
        } else {
            None
        };

        // Read authentication file
        let mut auth = String::new();
        if let Ok(mut file) = File::open("/auth.txt") {
            file.read_to_string(&mut auth).ok();
            auth = auth.trim().to_string();
        }

        Config {
            github_token: env::var("GHNTFY_TOKEN").ok(),
            docker_username,
            docker_password,
            docker_token,
            ntfy_url: env::var("NTFY_URL").ok(),
            gotify_url: env::var("GOTIFY_URL").ok(),
            gotify_token: env::var("GOTIFY_TOKEN").ok(),
            discord_webhook_url: env::var("DISCORD_WEBHOOK_URL").ok(),
            slack_webhook_url: env::var("SLACK_WEBHOOK_URL").ok(),
            auth,
            timeout: env::var("GHNTFY_TIMEOUT")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .unwrap_or(3600.0),
        }
    }

    fn github_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if let Some(token) = &self.github_token {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("token {}", token)).unwrap(),
            );
        }
        headers
    }

    fn docker_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if let Some(token) = &self.docker_token {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
            );
        }
        headers
    }
}

// Functions for DockerHub
fn create_dockerhub_token(username: &str, password: &str) -> Option<String> {
    let client = reqwest::blocking::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );

    let data = json!({
        "username": username,
        "password": password
    });

    match client
        .post("https://hub.docker.com/v2/users/login")
        .headers(headers)
        .json(&data)
        .send()
    {
        Ok(response) => {
            let status = response.status(); // Store status before consuming response
            if status.is_success() {
                if let Ok(json) = response.json::<serde_json::Value>() {
                    return json.get("token").and_then(|t| t.as_str()).map(String::from);
                }
            }
            error!("DockerHub authentication failed: {}", status);
            None
        }
        Err(e) => {
            error!("Error connecting to DockerHub: {}", e);
            None
        }
    }
}

// Database initialization
fn init_databases() -> SqliteResult<(Connection, Connection)> {
    let db_path = env::var("DB_PATH").unwrap_or_else(|_| "/github-ntfy".to_string());
    std::fs::create_dir_all(&db_path).ok();

    let versions_path = format!("{}/ghntfy_versions.db", db_path);
    let repos_path = format!("{}/watched_repos.db", db_path);

    let conn = Connection::open(&versions_path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS versions (
            repo TEXT PRIMARY KEY,
            version TEXT,
            changelog TEXT
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS docker_versions (
            repo TEXT PRIMARY KEY,
            digest TEXT
        )",
        [],
    )?;

    let conn2 = Connection::open(&repos_path)?;

    conn2.execute(
        "CREATE TABLE IF NOT EXISTS watched_repos (
            id INTEGER PRIMARY KEY,
            repo TEXT
        )",
        [],
    )?;

    conn2.execute(
        "CREATE TABLE IF NOT EXISTS docker_watched_repos (
            id INTEGER PRIMARY KEY,
            repo TEXT
        )",
        [],
    )?;

    Ok((conn, conn2))
}

// Functions to retrieve watched repositories
fn get_watched_repos(conn: &Connection) -> SqliteResult<Vec<String>> {
    let mut stmt = conn.prepare("SELECT * FROM watched_repos")?;
    let repos_iter = stmt.query_map([], |row| Ok(row.get::<_, String>(1)?))?;

    let mut repos = Vec::new();
    for repo in repos_iter {
        repos.push(repo?);
    }
    Ok(repos)
}

fn get_docker_watched_repos(conn: &Connection) -> SqliteResult<Vec<String>> {
    let mut stmt = conn.prepare("SELECT * FROM docker_watched_repos")?;
    let repos_iter = stmt.query_map([], |row| Ok(row.get::<_, String>(1)?))?;

    let mut repos = Vec::new();
    for repo in repos_iter {
        repos.push(repo?);
    }
    Ok(repos)
}

// Retrieving latest versions
async fn get_latest_releases(
    repos: &[String],
    client: &reqwest::Client,
    headers: HeaderMap,
) -> Vec<GithubReleaseInfo> {
    let mut releases = Vec::new();

    for repo in repos {
        let url = format!("https://api.github.com/repos/{}/releases/latest", repo);
        match client.get(&url).headers(headers.clone()).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    if let Ok(release_info) = response.json::<GithubRelease>().await {
                        let changelog = get_changelog(repo, client, headers.clone()).await;
                        let published_at = release_info.published_at
                            .unwrap_or_else(|| "Release date not available".to_string());

                        releases.push(GithubReleaseInfo {
                            repo: repo.clone(),
                            name: release_info.name,
                            tag_name: release_info.tag_name,
                            html_url: release_info.html_url,
                            changelog,
                            published_at,
                        });
                    }
                } else {
                    error!("Failed to retrieve info for {}: {}", repo, response.status());
                }
            }
            Err(e) => {
                error!("Error during request for {}: {}", repo, e);
            }
        }
    }

    releases
}

async fn get_changelog(
    repo: &str,
    client: &reqwest::Client,
    headers: HeaderMap,
) -> String {
    let url = format!("https://api.github.com/repos/{}/releases", repo);

    match client.get(&url).headers(headers).send().await {
        Ok(response) => {
            if response.status().is_success() {
                if let Ok(releases) = response.json::<Vec<GithubRelease>>().await {
                    if !releases.is_empty() {
                        if let Some(body) = &releases[0].body {
                            return body.clone();
                        }
                    }
                }
            }
        }
        Err(e) => {
            error!("Error retrieving changelog for {}: {}", repo, e);
        }
    }

    "Changelog not available".to_string()
}

async fn get_latest_docker_releases(
    repos: &[String],
    client: &reqwest::Client,
    headers: HeaderMap,
) -> Vec<DockerReleaseInfo> {
    let mut releases = Vec::new();

    for repo in repos {
        let url = format!("https://hub.docker.com/v2/repositories/{}/tags/latest", repo);
        match client.get(&url).headers(headers.clone()).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    if let Ok(release_info) = response.json::<DockerTag>().await {
                        releases.push(DockerReleaseInfo {
                            repo: repo.clone(),
                            digest: release_info.digest,
                            html_url: format!("https://hub.docker.com/r/{}", repo),
                            published_at: release_info.last_updated,
                        });
                    }
                } else {
                    error!("Failed to retrieve Docker info for {}: {}", repo, response.status());
                }
            }
            Err(e) => {
                error!("Error during Docker request for {}: {}", repo, e);
            }
        }
    }

    releases
}

// Complete notification sending function
async fn send_notifications(
    github_releases: Vec<GithubReleaseInfo>,
    docker_releases: Vec<DockerReleaseInfo>,
    config: &Config,
) {
    let mut tasks = Vec::new();

    // Create tasks for GitHub notifications
    for release in &github_releases {
        if let Some(url) = &config.ntfy_url {
            let release = release.clone();
            let auth = config.auth.clone();
            let url = url.clone();
            tasks.push(task::spawn(async move {
                github_send_to_ntfy(release, &auth, &url).await;
            }));
        }

        if let (Some(gotify_url), Some(gotify_token)) = (&config.gotify_url, &config.gotify_token) {
            let release = release.clone();
            let url = gotify_url.clone();
            let token = gotify_token.clone();
            tasks.push(task::spawn(async move {
                github_send_to_gotify(release, &token, &url).await;
            }));
        }

        if let Some(discord_url) = &config.discord_webhook_url {
            let release = release.clone();
            let url = discord_url.clone();
            tasks.push(task::spawn(async move {
                github_send_to_discord(release, &url).await;
            }));
        }

        if let Some(slack_url) = &config.slack_webhook_url {
            let release = release.clone();
            let url = slack_url.clone();
            tasks.push(task::spawn(async move {
                github_send_to_slack(release, &url).await;
            }));
        }
    }

    // Create tasks for Docker notifications
    for release in &docker_releases {
        if let Some(url) = &config.ntfy_url {
            let release = release.clone();
            let auth = config.auth.clone();
            let url = url.clone();
            tasks.push(task::spawn(async move {
                docker_send_to_ntfy(release, &auth, &url).await;
            }));
        }

        if let (Some(gotify_url), Some(gotify_token)) = (&config.gotify_url, &config.gotify_token) {
            let release = release.clone();
            let url = gotify_url.clone();
            let token = gotify_token.clone();
            tasks.push(task::spawn(async move {
                docker_send_to_gotify(release, &token, &url).await;
            }));
        }

        if let Some(discord_url) = &config.discord_webhook_url {
            let release = release.clone();
            let url = discord_url.clone();
            tasks.push(task::spawn(async move {
                docker_send_to_discord(release, &url).await;
            }));
        }

        if let Some(slack_url) = &config.slack_webhook_url {
            let release = release.clone();
            let url = slack_url.clone();
            tasks.push(task::spawn(async move {
                docker_send_to_slack(release, &url).await;
            }));
        }
    }

    // Wait for all tasks to complete
    for task in tasks {
        let _ = task.await;
    }
}

async fn github_send_to_ntfy(release: GithubReleaseInfo, auth: &str, ntfy_url: &str) {
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

async fn github_send_to_gotify(release: GithubReleaseInfo, token: &str, gotify_url: &str) {
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

async fn github_send_to_discord(release: GithubReleaseInfo, webhook_url: &str) {
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

async fn github_send_to_slack(release: GithubReleaseInfo, webhook_url: &str) {
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
                        "text": "🔗 Release Url"
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

async fn docker_send_to_ntfy(release: DockerReleaseInfo, auth: &str, ntfy_url: &str) {
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

async fn docker_send_to_gotify(release: DockerReleaseInfo, token: &str, gotify_url: &str) {
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

async fn docker_send_to_discord(release: DockerReleaseInfo, webhook_url: &str) {
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

async fn docker_send_to_slack(release: DockerReleaseInfo, webhook_url: &str) {
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
                        "text": "🔗 Release Url"
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

    let config = Config::from_env();
    let (_conn_versions, conn_repos) = init_databases()?;

    start_api();

    let client = reqwest::Client::new();

    if config.auth.is_empty() || (config.ntfy_url.is_none() && config.gotify_url.is_none()
        && config.discord_webhook_url.is_none() && config.slack_webhook_url.is_none()) {
        error!("Incorrect configuration!");
        error!("auth: can be generated with the command: echo -n 'username:password' | base64");
        error!("NTFY_URL: URL of the ntfy server");
        error!("GOTIFY_URL: URL of the gotify server");
        error!("GOTIFY_TOKEN: Gotify token");
        error!("DISCORD_WEBHOOK_URL: Discord webhook URL");
        error!("SLACK_WEBHOOK_URL: Slack webhook URL");
        error!("GHNTFY_TIMEOUT: interval between checks");
        return Ok(());
    }

    info!("Starting version monitoring...");

    loop {
        let github_repos = get_watched_repos(&conn_repos)?;
        let docker_repos = get_docker_watched_repos(&conn_repos)?;

        let github_releases = get_latest_releases(&github_repos, &client, config.github_headers()).await;
        let docker_releases = get_latest_docker_releases(&docker_repos, &client, config.docker_headers()).await;

        send_notifications(github_releases, docker_releases, &config).await;

        tokio::time::sleep(Duration::from_secs_f64(config.timeout)).await;
    }
}