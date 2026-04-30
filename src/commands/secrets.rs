use anyhow::Result;
use clap::Subcommand;
use serde::{Deserialize, Serialize};

use crate::{api, art, display};

#[derive(Subcommand)]
pub enum SecretsCmd {
    /// Generate a cryptographic secret
    Generate {
        #[arg(short, long, default_value = "MY_SECRET")]
        label: String,
        #[arg(short, long, default_value = "base64url", help = "base64url | hex")]
        format: String,
        #[arg(short, long, default_value_t = 32)]
        bytes: u32,
    },
}

#[derive(Serialize)]
struct GenerateRequest {
    label: String,
    format: String,
    bytes: u32,
}

#[derive(Deserialize)]
struct GenerateResponse {
    label: String,
    value: String,
    bytes: u32,
}

pub async fn run(cmd: SecretsCmd) -> Result<()> {
    match cmd {
        SecretsCmd::Generate { label, format, bytes } => generate(label, format, bytes).await,
    }
}

async fn generate(label: String, format: String, bytes: u32) -> Result<()> {
    let sp = art::spinner("Generating…");
    let res: GenerateResponse =
        api::post("/secrets/generate", &GenerateRequest { label, format, bytes }).await?;
    sp.finish_and_clear();

    display::header("Generated secret");
    display::kv(&[
        ("Label", res.label),
        ("Bytes", res.bytes.to_string()),
        ("Value", res.value),
    ]);
    Ok(())
}
