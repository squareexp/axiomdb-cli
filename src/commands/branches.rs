use anyhow::{bail, Result};
use clap::Subcommand;
use dialoguer::{Confirm, Input};
use serde::{Deserialize, Serialize};

use crate::{api, art, commands::gen, config, credentials, display};

#[derive(Subcommand)]
pub enum BranchesCmd {
    /// List all branches for a project
    #[clap(visible_alias = "ls", short_flag = 'l')]
    List {
        /// Project ID (omit to use active project context)
        project_id: Option<String>,
    },
    /// Create a new branch (max 10 active per project)
    #[clap(visible_alias = "cr")]
    Create {
        /// Project ID (omit to use active project context)
        project_id: Option<String>,
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Delete a branch
    #[clap(visible_alias = "rm")]
    Delete {
        project_id: String,
        branch_id: String,
        #[arg(short, long)]
        yes: bool,
    },
    /// Print Prisma-ready URLs for a branch
    #[clap(visible_alias = "url")]
    Urls {
        /// Branch ID (omit when using --name)
        branch_id: Option<String>,
        /// Branch name to resolve inside the selected project
        #[arg(short, long)]
        name: Option<String>,
        /// Project ID (omit to use active project context)
        #[arg(short, long)]
        project: Option<String>,
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
        BranchesCmd::List { project_id } => {
            list(config::resolve_project(project_id.as_deref())?).await
        }
        BranchesCmd::Create { project_id, name } => {
            create(config::resolve_project(project_id.as_deref())?, name).await
        }
        BranchesCmd::Delete {
            project_id,
            branch_id,
            yes,
        } => delete(project_id, branch_id, yes).await,
        BranchesCmd::Urls {
            branch_id,
            name,
            project,
        } => {
            urls(
                config::resolve_project(project.as_deref())?,
                branch_id,
                name,
            )
            .await
        }
    }
}

async fn list(project_id: String) -> Result<()> {
    let sp = art::spinner("Fetching branches…");
    let res: BranchListResponse = api::get(&format!("/projects/{project_id}/branches")).await?;
    sp.finish_and_clear();

    display::header(&format!("Branches for {project_id}"));
    if res.branches.is_empty() {
        println!("  No branches yet. Run: axm branches create");
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
                if v.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
                {
                    Ok(())
                } else {
                    Err("Lowercase letters, digits, hyphens, underscores only")
                }
            })
            .interact_text()?,
    };

    let sp = art::spinner(&format!("Creating branch \"{branch_name}\"…"));
    let branch: Branch = api::post(
        &format!("/projects/{project_id}/branches"),
        &CreateBranchRequest { branch_name },
    )
    .await?;
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

async fn urls(project_id: String, branch_id: Option<String>, name: Option<String>) -> Result<()> {
    let branch_ref = branch_ref(branch_id.as_deref(), name.as_deref())?;
    let sp = art::spinner("Resolving branch DB URLs…");
    let creds: credentials::Credentials = api::get(&format!(
        "/projects/{project_id}/branches/{branch_ref}/credentials"
    ))
    .await?;
    sp.finish_and_clear();
    credentials::ensure_prisma_contract(&creds)?;

    gen::print_credentials(&creds);
    display::ok("Runtime URL is for app traffic; DIRECT_URL is for Prisma migrate/db push.");
    Ok(())
}

pub(crate) fn branch_ref(branch_id: Option<&str>, name: Option<&str>) -> Result<String> {
    match (branch_id, name) {
        (Some(_), Some(_)) => bail!("Pass either a branch id or --name, not both."),
        (Some(id), None) if !id.trim().is_empty() => Ok(id.trim().to_string()),
        (None, Some(name)) if !name.trim().is_empty() => Ok(name.trim().to_lowercase()),
        _ => bail!(
            "Branch is required. Use: axm branches urls <branch-id> or axm branches urls --name <branch-name>"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::branch_ref;

    #[test]
    fn resolves_branch_id_or_name() {
        assert_eq!(
            branch_ref(Some("  branch-id  "), None).unwrap(),
            "branch-id"
        );
        assert_eq!(
            branch_ref(None, Some("Feature-Proof")).unwrap(),
            "feature-proof"
        );
    }

    #[test]
    fn rejects_missing_or_ambiguous_branch_refs() {
        assert!(branch_ref(None, None).is_err());
        assert!(branch_ref(Some("branch-id"), Some("feature-proof")).is_err());
    }
}
