use crate::types::{ProfileResponse, UsageResponse};
use reqwest::StatusCode;
use std::time::Duration;

const USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";
const PROFILE_URL: &str = "https://api.anthropic.com/api/oauth/profile";
const USER_AGENT: &str = "claude-code/1.0.0";
const BETA_HEADER: &str = "oauth-2025-04-20";

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("auth token expired or rejected")]
    TokenExpired,
    #[error("rate limited")]
    RateLimited { retry_after: Option<f64> },
    #[error("HTTP {0}")]
    Http(u16),
    #[error("request failed: {0}")]
    Request(String),
}

impl From<reqwest::Error> for ApiError {
    fn from(e: reqwest::Error) -> Self {
        ApiError::Request(e.to_string())
    }
}

pub struct ApiClient {
    client: reqwest::Client,
}

impl Default for ApiClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .timeout(Duration::from_secs(30))
                .build()
                .expect("reqwest client"),
        }
    }

    pub async fn fetch_usage(&self, token: &str) -> Result<UsageResponse, ApiError> {
        self.get(USAGE_URL, token).await
    }

    pub async fn fetch_profile(&self, token: &str) -> Result<ProfileResponse, ApiError> {
        self.get(PROFILE_URL, token).await
    }

    async fn get<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        token: &str,
    ) -> Result<T, ApiError> {
        let resp = self
            .client
            .get(url)
            .bearer_auth(token)
            .header("anthropic-beta", BETA_HEADER)
            .send()
            .await?;

        let status = resp.status();
        match status {
            StatusCode::OK => Ok(resp.json::<T>().await?),
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(ApiError::TokenExpired),
            StatusCode::TOO_MANY_REQUESTS => {
                let retry = resp
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<f64>().ok());
                Err(ApiError::RateLimited { retry_after: retry })
            }
            other => Err(ApiError::Http(other.as_u16())),
        }
    }
}
