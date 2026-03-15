//! Shared HTTP client (reqwest) — API key validation, model fetching

use reqwest::Client;
use thiserror::Error;

pub const GROQ_BASE_URL: &str = "https://api.groq.com/openai/v1";

#[derive(Debug, Error)]
pub enum HttpClientError {
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("API returned status {0}: {1}")]
    ApiError(u16, String),
    #[error("Invalid API key (empty)")]
    EmptyApiKey,
}

/// Build a shared reqwest Client with rustls TLS.
pub fn build_client() -> Result<Client, HttpClientError> {
    let client = Client::builder()
        .use_rustls_tls()
        .build()?;
    Ok(client)
}

/// Validate an API key by hitting GET /models and checking for HTTP 200.
pub async fn validate_api_key(
    client: &Client,
    api_key: &str,
    base_url: &str,
) -> bool {
    let trimmed = api_key.trim();
    if trimmed.is_empty() {
        return false;
    }

    let url = format!("{}/models", base_url);
    let result = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", trimmed))
        .send()
        .await;

    match result {
        Ok(resp) => resp.status().as_u16() == 200,
        Err(_) => false,
    }
}
