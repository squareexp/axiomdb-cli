use anyhow::Result;
use clap::Subcommand;
use serde::Deserialize;

use crate::{api, art, display, utils::resource};

#[derive(Subcommand)]
pub enum TablesCmd {
    /// List all tables in the project database
    #[clap(visible_alias = "ls")]
    List { project_ref: Option<String> },
}

#[derive(Deserialize)]
struct TablesResponse {
    tables: Vec<String>,
    database: Option<String>,
}

pub async fn run(cmd: TablesCmd) -> Result<()> {
    match cmd {
        TablesCmd::List { project_ref } => list(project_ref).await,
    }
}

async fn list(project_ref: Option<String>) -> Result<()> {
    let project_id = resource::resolve_project(project_ref.as_deref()).await?;
    let sp = art::spinner("Querying schema…");
    let res: TablesResponse = api::get(&format!("/projects/{project_id}/tables")).await?;
    sp.finish_and_clear();

    let db = res.database.as_deref().unwrap_or(&project_id);
    display::header(&format!("Tables in {db}"));

    if res.tables.is_empty() {
        println!("  No tables found.");
        return Ok(());
    }

    display::table(
        &["Table name"],
        res.tables.iter().map(|t| vec![t.clone()]).collect(),
    );
    Ok(())
}
