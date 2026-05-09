use anyhow::{bail, Result};
use clap::Subcommand;
use dialoguer::{Confirm, Input, Select};
use serde::{Deserialize, Serialize};

use crate::{api, art, commands::gen, credentials, display, utils::resource};

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
        /// Source branch ID (defaults to main)
        #[arg(long)]
        source: Option<String>,
        /// Branch lifespan: 7d, 1m, 6m, 1y, forever
        #[arg(short = 'f', long)]
        lifespan: Option<String>,
    },
    /// Delete a branch
    #[clap(visible_alias = "rm")]
    Delete {
        /// Branch name/ID, or project name/ID when a second positional is provided
        first: String,
        /// Branch name/ID when the first positional is the project
        second: Option<String>,
        /// Project name/ID (omit to use active context)
        #[arg(short, long)]
        project: Option<String>,
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
    lifespan: Option<String>,
    expires_at: Option<String>,
    protected: Option<bool>,
    is_default: Option<bool>,
    created_at: String,
}

#[derive(Deserialize)]
struct BranchListResponse {
    branches: Vec<Branch>,
}

#[derive(Serialize)]
struct CreateBranchRequest {
    branch_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_branch_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lifespan: Option<String>,
}

pub async fn run(cmd: BranchesCmd) -> Result<()> {
    match cmd {
        BranchesCmd::List { project_id } => {
            list(resource::resolve_project(project_id.as_deref()).await?).await
        }
        BranchesCmd::Create {
            project_id,
            name,
            source,
            lifespan,
        } => {
            create(
                resource::resolve_project(project_id.as_deref()).await?,
                name,
                source,
                lifespan,
            )
            .await
        }
        BranchesCmd::Delete {
            first,
            second,
            project,
            yes,
        } => delete(first, second, project, yes).await,
        BranchesCmd::Urls {
            branch_id,
            name,
            project,
        } => {
            urls(
                resource::resolve_project(project.as_deref()).await?,
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
        &["Name", "Database", "Status", "Lifespan", "Expires"],
        res.branches
            .iter()
            .map(|b| {
                vec![
                    branch_label(b),
                    b.database_name.clone(),
                    display::status_color(&b.status),
                    b.lifespan.clone().unwrap_or_else(|| "forever".into()),
                    b.expires_at
                        .as_deref()
                        .map(|value| value[..10.min(value.len())].to_string())
                        .unwrap_or_else(|| "never".into()),
                ]
            })
            .collect(),
    );
    Ok(())
}

async fn create(
    project_id: String,
    name: Option<String>,
    source: Option<String>,
    lifespan: Option<String>,
) -> Result<()> {
    let interactive = name.is_none();
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

    let lifespan = match lifespan {
        Some(value) => Some(normalize_lifespan(&value)?),
        None if !interactive => Some("7d".to_string()),
        None => {
            let options = ["7d", "1m", "6m", "1y", "forever"];
            let i = Select::new()
                .with_prompt("Branch lifespan")
                .items(&["7 days", "1 month", "6 months", "1 year", "forever"])
                .default(0)
                .interact()?;
            Some(options[i].to_string())
        }
    };

    let sp = art::spinner(&format!("Creating branch \"{branch_name}\"…"));
    let branch: Branch = api::post(
        &format!("/projects/{project_id}/branches"),
        &CreateBranchRequest {
            branch_name,
            source_branch_id: source,
            lifespan,
        },
    )
    .await?;
    sp.finish_and_clear();

    display::ok(&format!(
        "Branch \"{}\" → {} ({})",
        branch.branch_name,
        branch.database_name,
        branch.lifespan.unwrap_or_else(|| "forever".into())
    ));
    Ok(())
}

async fn delete(
    first: String,
    second: Option<String>,
    project: Option<String>,
    yes: bool,
) -> Result<()> {
    let (project_ref, branch_ref) = match second {
        Some(branch_ref) => (Some(first), branch_ref),
        None => (project, first),
    };
    let project_id = resource::resolve_project(project_ref.as_deref()).await?;
    let branch = resource::resolve_branch(&project_id, &branch_ref).await?;

    if !yes {
        let ok = Confirm::new()
            .with_prompt(format!(
                "Delete branch {} ({})? This cannot be undone.",
                branch.branch_name, branch.database_name
            ))
            .default(false)
            .interact()?;
        if !ok {
            return Ok(());
        }
    }

    let sp = art::spinner("Deleting branch…");
    api::delete_req(&format!("/projects/{project_id}/branches/{}", branch.id)).await?;
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

fn branch_label(branch: &Branch) -> String {
    if branch.is_default.unwrap_or(false) {
        format!("{} (main)", branch.branch_name)
    } else if branch.protected.unwrap_or(false) {
        format!("{} (protected)", branch.branch_name)
    } else {
        branch.branch_name.clone()
    }
}

pub(crate) fn normalize_lifespan(value: &str) -> Result<String> {
    match value.trim().to_lowercase().as_str() {
        "7d" | "7days" | "7 days" => Ok("7d".into()),
        "1m" | "1month" | "1 month" => Ok("1m".into()),
        "6m" | "6months" | "6 months" => Ok("6m".into()),
        "1y" | "1year" | "1 year" => Ok("1y".into()),
        "forever" | "permanent" => Ok("forever".into()),
        _ => bail!("lifespan must be one of 7d, 1m, 6m, 1y, forever"),
    }
}

#[cfg(test)]
mod tests {
    use super::{branch_ref, normalize_lifespan};

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

    #[test]
    fn normalizes_lifespan_flags() {
        assert_eq!(normalize_lifespan("7 days").unwrap(), "7d");
        assert_eq!(normalize_lifespan("1month").unwrap(), "1m");
        assert_eq!(normalize_lifespan("permanent").unwrap(), "forever");
        assert!(normalize_lifespan("3d").is_err());
    }
}
