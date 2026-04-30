use anyhow::{bail, Context, Result};
use reqwest::{Client, Method, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

use crate::config;

fn client() -> Client {
    Client::builder()
        .user_agent("pulsardb-cli/0.1.0")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("HTTP client init failed")
}

pub async fn request<T: DeserializeOwned>(
    method: Method,
    path: &str,
    body: Option<&impl Serialize>,
    auth: bool,
) -> Result<T> {
    let cfg = config::load();
    let url = format!("{}/api/v1{}", cfg.base_url.trim_end_matches('/'), path);

    let mut req = client().request(method, &url).header("Content-Type", "application/json");

    if auth {
        let token = config::require_token()?;
        req = req.bearer_auth(token);
    }

    if let Some(b) = body {
        req = req.json(b);
    }

    let res = req.send().await.with_context(|| format!("Failed to reach {url}"))?;
    let status = res.status();
    let text = res.text().await.unwrap_or_default();

    if !status.is_success() {
        let parsed: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
        let msg = parsed
            .pointer("/error/message")
            .and_then(|v| v.as_str())
            .unwrap_or(&text);
        let code = parsed
            .pointer("/error/code")
            .and_then(|v| v.as_str())
            .unwrap_or("ERROR");
        bail!("[{code}] {msg}  (HTTP {status})");
    }

    serde_json::from_str(&text).with_context(|| format!("Failed to parse response from {url}"))
}

pub async fn get<T: DeserializeOwned>(path: &str) -> Result<T> {
    request(Method::GET, path, None::<&()>, true).await
}

pub async fn post<T: DeserializeOwned>(path: &str, body: &impl Serialize) -> Result<T> {
    request(Method::POST, path, Some(body), true).await
}

pub async fn post_no_auth<T: DeserializeOwned>(path: &str, body: &impl Serialize) -> Result<T> {
    request(Method::POST, path, Some(body), false).await
}

pub async fn delete_req(path: &str) -> Result<()> {
    let cfg = config::load();
    let url = format!("{}/api/v1{}", cfg.base_url.trim_end_matches('/'), path);
    let token = config::require_token()?;

    let res = client()
        .delete(&url)
        .bearer_auth(token)
        .send()
        .await
        .with_context(|| format!("DELETE {url} failed"))?;

    if !res.status().is_success() && res.status() != StatusCode::NO_CONTENT {
        let text = res.text().await.unwrap_or_default();
        bail!("Delete failed: {text}");
    }
    Ok(())
}

/// SSE streaming — yields raw data lines from the server.
pub async fn stream_sse(path: &str) -> Result<reqwest::Response> {
    let cfg = config::load();
    let url = format!("{}/api/v1{}", cfg.base_url.trim_end_matches('/'), path);
    let token = config::require_token()?;

    let res = client()
        .get(&url)
        .bearer_auth(token)
        .header("Accept", "text/event-stream")
        .send()
        .await
        .with_context(|| format!("SSE connect to {url} failed"))?;

    Ok(res)
}
