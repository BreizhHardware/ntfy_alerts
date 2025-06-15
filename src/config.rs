use dotenv::dotenv;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use std::env;
use std::fs::File;
use std::io::Read;
use crate::docker::create_dockerhub_token;

// Configuration
pub struct Config {
    pub github_token: Option<String>,
    pub docker_username: Option<String>,
    pub docker_password: Option<String>,
    pub docker_token: Option<String>,
    pub ntfy_url: Option<String>,
    pub gotify_url: Option<String>,
    pub gotify_token: Option<String>,
    pub discord_webhook_url: Option<String>,
    pub slack_webhook_url: Option<String>,
    pub auth: String,
    pub timeout: f64,
}

impl Config {
    pub fn from_env() -> Self {
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

    pub fn github_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if let Some(token) = &self.github_token {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("token {}", token)).unwrap(),
            );
        }
        headers
    }

    pub fn docker_headers(&self) -> HeaderMap {
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