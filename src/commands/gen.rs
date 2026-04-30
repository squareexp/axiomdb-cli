use anyhow::Result;
use clap::Subcommand;
use serde::Deserialize;

use crate::{api, art, display};

#[derive(Subcommand)]
pub enum GenCmd {
    /// Generate Prisma-ready DB URLs for a project
    Tk { project_id: String },
}

#[derive(Deserialize)]
struct Credentials {
    #[allow(dead_code)]
    project_id: String,
    database: String,
    runtime_key: String,
    direct_key: String,
    database_url: String,
    direct_url: String,
}

pub async fn run(cmd: GenCmd) -> Result<()> {
    match cmd {
        GenCmd::Tk { project_id } => tk(project_id).await,
    }
}

async fn tk(project_id: String) -> Result<()> {
    let sp = art::spinner("Resolving project DB URLs…");
    let creds: Credentials = api::get(&format!("/projects/{project_id}/credentials")).await?;
    sp.finish_and_clear();

    display::header(&format!("Prisma URLs: {}", creds.database));
    display::kv(&[
        (&creds.runtime_key, creds.database_url.clone()),
        (&creds.direct_key, creds.direct_url.clone()),
    ]);
    display::ok(&format!(
        "Use in .env:\n  DATABASE_URL=\"{}\"\n  DIRECT_URL=\"{}\"",
        creds.database_url, creds.direct_url
    ));
    Ok(())
}
