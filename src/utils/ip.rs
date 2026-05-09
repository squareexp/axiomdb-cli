use anyhow::{bail, Result};
use serde::Deserialize;

#[derive(Deserialize)]
struct IpifyResponse {
    ip: String,
}

pub async fn public_ip() -> Result<String> {
    let response = reqwest::Client::builder()
        .user_agent(concat!("axiom-cli/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(8))
        .build()?
        .get("https://api.ipify.org?format=json")
        .send()
        .await?;
    if !response.status().is_success() {
        bail!("could not detect public IP");
    }
    let body: IpifyResponse = response.json().await?;
    if body.ip.trim().is_empty() {
        bail!("public IP response was empty");
    }
    Ok(body.ip)
}

pub async fn current_cidr_from_gateway_or_ipify() -> Result<String> {
    #[derive(Deserialize)]
    struct GatewayIp {
        suggested_cidr: Option<String>,
    }

    if let Ok(response) = crate::api::get::<GatewayIp>("/network/current-ip").await {
        if let Some(cidr) = response
            .suggested_cidr
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return Ok(cidr.to_string());
        }
    }

    Ok(format!("{}/32", public_ip().await?))
}
