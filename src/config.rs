use anyhow::{bail, Result};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: String,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_base_url")]
    pub base_url: String,
    pub tokens: Option<Tokens>,
    /// axiom will use this project id to perform all database operations unless
    /// explicitly overridden by --project <project_id>
    pub current_project: Option<String>,
}

fn default_base_url() -> String {
    "https://opsdc.squareexp.com".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            base_url: default_base_url(),
            tokens: None,
            current_project: None,
        }
    }
}

fn config_path() -> PathBuf {
    config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("axiom")
        .join("config.json")
}

pub fn load() -> Config {
    let path = config_path();
    if path.exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        Config::default()
    }
}

pub fn save(cfg: &Config) -> Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(cfg)?;
    fs::write(&path, json)?;
    Ok(())
}

pub fn set_base_url(url: &str) -> Result<()> {
    let mut cfg = load();
    cfg.base_url = url.trim_end_matches('/').to_string();
    save(&cfg)
}

pub fn set_tokens(tokens: Tokens) -> Result<()> {
    let mut cfg = load();
    cfg.tokens = Some(tokens);
    save(&cfg)
}

pub fn clear_tokens() -> Result<()> {
    let mut cfg = load();
    cfg.tokens = None;
    save(&cfg)
}

pub fn set_current_project(id: &str) -> Result<()> {
    let mut cfg = load();
    cfg.current_project = Some(id.to_string());
    save(&cfg)
}

pub fn clear_current_project() -> Result<()> {
    let mut cfg = load();
    cfg.current_project = None;
    save(&cfg)
}

/// Returns the current project id — from arg if given, else from saved context, else errors.
pub fn resolve_project(arg: Option<&str>) -> Result<String> {
    if let Some(id) = arg {
        return Ok(id.to_string());
    }
    let cfg = load();
    cfg.current_project.ok_or_else(|| {
        anyhow::anyhow!("No project selected. Pass a project ID or run:\n  axiom projects use <id>")
    })
}

pub fn require_token() -> Result<String> {
    let cfg = load();
    match cfg.tokens {
        Some(t) if !t.access_token.is_empty() => Ok(t.access_token),
        _ => bail!("Not logged in. Run: axiom login"),
    }
}
