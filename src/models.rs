use serde::Deserialize;
use serde::Serialize;

// Structures for GitHub data
#[derive(Debug, Deserialize, Clone)]
pub struct GithubRelease {
    pub name: String,
    pub tag_name: String,
    pub html_url: String,
    pub published_at: Option<String>,
    pub body: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GithubReleaseInfo {
    pub repo: String,
    pub name: String,
    pub tag_name: String,
    pub html_url: String,
    pub changelog: String,
    pub published_at: String,
}

// Structures for Docker data
#[derive(Debug, Deserialize)]
pub struct DockerTag {
    pub digest: String,
    pub last_updated: String,
}

#[derive(Debug, Clone)]
pub struct DockerReleaseInfo {
    pub repo: String,
    pub digest: String,
    pub html_url: String,
    pub published_at: String,
}

pub struct NotifiedRelease {
    pub repo: String,
    pub tag_name: String,
    pub notified_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub is_admin: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserLogin {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserRegistration {
    pub username: String,
    pub password: String,
    pub is_admin: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Session {
    pub token: String,
    pub user_id: i64,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub id: Option<i64>,
    pub ntfy_url: Option<String>,
    pub github_token: Option<String>,
    pub docker_username: Option<String>,
    pub docker_password: Option<String>,
    pub gotify_url: Option<String>,
    pub gotify_token: Option<String>,
    pub discord_webhook_url: Option<String>,
    pub slack_webhook_url: Option<String>,
    pub check_interval: Option<i64>,
    pub auth: Option<String>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}
