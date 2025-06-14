use serde::Deserialize;

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