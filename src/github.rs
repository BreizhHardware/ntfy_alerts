use log::error;
use reqwest::header::HeaderMap;
use crate::models::{GithubRelease, GithubReleaseInfo};

pub async fn get_latest_releases(
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
                    if let Ok(release) = response.json::<GithubRelease>().await {
                        let changelog = get_changelog(repo, client, headers.clone()).await;

                        releases.push(GithubReleaseInfo {
                            repo: repo.clone(),
                            name: release.name,
                            tag_name: release.tag_name,
                            html_url: release.html_url,
                            changelog,
                            published_at: release.published_at.unwrap_or_else(|| "Unknown date".to_string()),
                        });
                    }
                } else {
                    error!("Error fetching GitHub release for {}: {}", repo, response.status());
                }
            }
            Err(e) => {
                error!("Error fetching GitHub release for {}: {}", repo, e);
            }
        }
    }

    releases
}

pub async fn get_changelog(
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