use anyhow::{bail, Result};
use clap::Subcommand;
use serde::Deserialize;

use crate::{api, art, display};

#[derive(Subcommand)]
pub enum GenCmd {
    /// Generate Prisma-ready DB URLs for a project
    Tk {
        project_id: String,
        /// Optional branch id or branch name
        #[arg(short, long)]
        branch: Option<String>,
    },
}

#[derive(Deserialize)]
struct Credentials {
    #[allow(dead_code)]
    project_id: String,
    database: String,
    #[allow(dead_code)]
    runtime_key: String,
    #[allow(dead_code)]
    direct_key: String,
    database_url: String,
    direct_url: String,
}

pub async fn run(cmd: GenCmd) -> Result<()> {
    match cmd {
        GenCmd::Tk { project_id, branch } => tk(project_id, branch).await,
    }
}

async fn tk(project_id: String, branch: Option<String>) -> Result<()> {
    let sp = art::spinner("Resolving project DB URLs…");
    let path = match branch.as_deref() {
        Some(branch) => format!("/projects/{project_id}/branches/{branch}/credentials"),
        None => format!("/projects/{project_id}/credentials"),
    };
    let creds: Credentials = api::get(&path).await?;
    sp.finish_and_clear();
    ensure_prisma_contract(&creds)?;

    display::header(&format!("Prisma URLs: {}", creds.database));
    display::kv(&[
        ("DATABASE_URL", creds.database_url.clone()),
        ("DIRECT_URL", creds.direct_url.clone()),
    ]);
    println!(
        "\nDATABASE_URL=\"{}\"\nDIRECT_URL=\"{}\"",
        creds.database_url, creds.direct_url
    );
    display::ok("Copy the block above into Prisma .env; DIRECT_URL is for migrations.");
    Ok(())
}

fn ensure_prisma_contract(creds: &Credentials) -> Result<()> {
    ensure_url(&creds.database_url, "DATABASE_URL", 6432)?;
    ensure_url(&creds.direct_url, "DIRECT_URL", 5432)?;
    Ok(())
}

fn ensure_url(url: &str, label: &str, port: u16) -> Result<()> {
    let host_port = format!("@db.squareexp.com:{port}/");
    let has_sslmode = url.contains("?sslmode=require") || url.contains("&sslmode=require");
    if !url.starts_with("postgresql://") || !url.contains(&host_port) || !has_sslmode {
        bail!(
            "{label} from gateway is not Prisma-ready; expected db.squareexp.com:{port} with sslmode=require"
        );
    }
    Ok(())
}
