use anyhow::Result;
use clap::Subcommand;
use serde::Deserialize;

use crate::{api, art, display};

#[derive(Subcommand)]
pub enum TablesCmd {
    /// List all tables in the project database
    #[clap(alias = "ls")]
    List { project_id: String },
}

#[derive(Deserialize)]
struct TablesResponse {
    tables: Vec<String>,
    database: Option<String>,
}

pub async fn run(cmd: TablesCmd) -> Result<()> {
    match cmd {
        TablesCmd::List { project_id } => list(project_id).await,
    }
}

async fn list(project_id: String) -> Result<()> {
    let sp = art::spinner("Querying schema…");
    let res: TablesResponse = api::get(&format!("/projects/{project_id}/tables")).await?;
    sp.finish_and_clear();

    let db = res.database.as_deref().unwrap_or(&project_id);
    display::header(&format!("Tables in {db}"));

    if res.tables.is_empty() {
        println!("  No tables found.");
        return Ok(());
    }

    display::table(&["Table name"], res.tables.iter().map(|t| vec![t.clone()]).collect());
    Ok(())
}
