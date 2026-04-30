use anyhow::Result;
use clap::Subcommand;
use dialoguer::{Confirm, Input};
use serde::{Deserialize, Serialize};

use crate::{api, art, config, display};

#[derive(Subcommand)]
pub enum BranchesCmd {
    /// List all branches for a project
    #[clap(alias = "ls", short_flag = 'l')]
    List {
        /// Project ID (omit to use active project context)
        project_id: Option<String>
    },
    /// Create a new branch (max 10 active per project)
    Create {
        /// Project ID (omit to use active project context)
        project_id: Option<String>,
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Delete a branch
    #[clap(alias = "rm")]
    Delete {
        project_id: String,
        branch_id: String,
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct Branch {
    id: String,
    branch_name: String,
    database_name: String,
    status: String,
    created_at: String,
}

#[derive(Deserialize)]
struct BranchListResponse {
    branches: Vec<Branch>,
}

#[derive(Serialize)]
struct CreateBranchRequest {
    branch_name: String,
}

pub async fn run(cmd: BranchesCmd) -> Result<()> {
    match cmd {
        BranchesCmd::List   { project_id } => list(config::resolve_project(project_id.as_deref())?).await,
        BranchesCmd::Create { project_id, name } => create(config::resolve_project(project_id.as_deref())?, name).await,
        BranchesCmd::Delete { project_id, branch_id, yes } => {
            delete(project_id, branch_id, yes).await
        }
    }
}

async fn list(project_id: String) -> Result<()> {
    let sp = art::spinner("Fetching branches…");
    let res: BranchListResponse = api::get(&format!("/projects/{project_id}/branches")).await?;
    sp.finish_and_clear();

    display::header(&format!("Branches for {project_id}"));
    if res.branches.is_empty() {
        println!("  No branches yet. Run: pulsardb branches create {project_id}");
        return Ok(());
    }

    display::table(
        &["Name", "Database", "Status", "Created"],
        res.branches
            .iter()
            .map(|b| {
                vec![
                    b.branch_name.clone(),
                    b.database_name.clone(),
                    display::status_color(&b.status),
                    b.created_at[..10].to_string(),
                ]
            })
            .collect(),
    );
    Ok(())
}

async fn create(project_id: String, name: Option<String>) -> Result<()> {
    let branch_name: String = match name {
        Some(n) => n,
        None => Input::new()
            .with_prompt("Branch name")
            .validate_with(|v: &String| {
                if v.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_') {
                    Ok(())
                } else {
                    Err("Lowercase letters, digits, hyphens, underscores only")
                }
            })
            .interact_text()?,
    };

    let sp = art::spinner(&format!("Creating branch \"{branch_name}\"…"));
    let branch: Branch =
        api::post(&format!("/projects/{project_id}/branches"), &CreateBranchRequest { branch_name }).await?;
    sp.finish_and_clear();

    display::ok(&format!(
        "Branch \"{}\" → {}",
        branch.branch_name, branch.database_name
    ));
    Ok(())
}

async fn delete(project_id: String, branch_id: String, yes: bool) -> Result<()> {
    if !yes {
        let ok = Confirm::new()
            .with_prompt(format!("Delete branch {branch_id}? This cannot be undone."))
            .default(false)
            .interact()?;
        if !ok {
            return Ok(());
        }
    }

    let sp = art::spinner("Deleting branch…");
    api::delete_req(&format!("/projects/{project_id}/branches/{branch_id}")).await?;
    sp.finish_and_clear();
    display::ok("Branch deleted.");
    Ok(())
}
