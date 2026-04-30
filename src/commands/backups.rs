use anyhow::Result;
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{api, art, display};

#[derive(Subcommand)]
pub enum BackupsCmd {
    /// Show backup catalog
    #[clap(alias = "ls")]
    List { project_id: String },
    /// Queue a restore job
    Restore {
        project_id: String,
        #[arg(short = 't', long)]
        point_in_time: Option<String>,
        #[arg(short, long)]
        snapshot_id: Option<String>,
    },
}

#[derive(Deserialize)]
struct BackupListResponse {
    app_key: String,
    env: String,
    catalog: Value,
}

#[derive(Deserialize)]
struct RestoreResponse {
    job_id: String,
    status: String,
    message: String,
}

#[derive(Serialize)]
struct RestoreRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    point_in_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    snapshot_id: Option<String>,
}

pub async fn run(cmd: BackupsCmd) -> Result<()> {
    match cmd {
        BackupsCmd::List { project_id } => list(project_id).await,
        BackupsCmd::Restore { project_id, point_in_time, snapshot_id } => {
            restore(project_id, point_in_time, snapshot_id).await
        }
    }
}

async fn list(project_id: String) -> Result<()> {
    let sp = art::spinner("Loading backup catalog…");
    let res: BackupListResponse = api::get(&format!("/projects/{project_id}/backups")).await?;
    sp.finish_and_clear();

    display::header(&format!("Backups — {}-{}", res.app_key, res.env));
    println!("{}", serde_json::to_string_pretty(&res.catalog)?);
    Ok(())
}

async fn restore(project_id: String, point_in_time: Option<String>, snapshot_id: Option<String>) -> Result<()> {
    let sp = art::spinner("Queuing restore job…");
    let res: RestoreResponse = api::post(
        &format!("/projects/{project_id}/backups/restore"),
        &RestoreRequest { point_in_time, snapshot_id },
    )
    .await?;
    sp.finish_and_clear();

    display::ok("Restore job queued");
    display::kv(&[
        ("Job ID", res.job_id),
        ("Status", display::status_color(&res.status)),
        ("Message", res.message),
    ]);
    Ok(())
}
