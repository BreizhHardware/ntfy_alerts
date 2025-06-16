use log::error;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde_json::json;
use crate::models::{DockerTag, DockerReleaseInfo};

pub fn create_dockerhub_token(username: &str, password: &str) -> Option<String> {
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
            let status = response.status();
            if status.is_success() {
                if let Ok(json) = response.json::<serde_json::Value>() {
                    return json["token"].as_str().map(|s| s.to_string());
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

pub async fn get_latest_docker_releases(
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
                    if let Ok(tag) = response.json::<DockerTag>().await {
                        releases.push(DockerReleaseInfo {
                            repo: repo.clone(),
                            digest: tag.digest.clone(),
                            html_url: format!("https://hub.docker.com/r/{}", repo),
                            published_at: tag.last_updated,
                        });
                    }
                } else {
                    error!("Error fetching Docker tag for {}: {}", repo, response.status());
                }
            }
            Err(e) => {
                error!("Error fetching Docker tag for {}: {}", repo, e);
            }
        }
    }

    releases
}