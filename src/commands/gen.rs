use anyhow::Result;
use clap::Subcommand;

use crate::{api, art, credentials, display, utils::resource};

#[derive(Subcommand)]
pub enum GenCmd {
    /// Generate Prisma-ready DB URLs for a project
    #[clap(visible_alias = "token")]
    Tk {
        /// Project ID (omit to use active project context)
        project_id: Option<String>,
        /// Optional branch id or branch name
        #[arg(short, long)]
        branch: Option<String>,
    },
}

pub async fn run(cmd: GenCmd) -> Result<()> {
    match cmd {
        GenCmd::Tk { project_id, branch } => tk(project_id, branch).await,
    }
}

async fn tk(project_id: Option<String>, branch: Option<String>) -> Result<()> {
    let project_id = resource::resolve_project(project_id.as_deref()).await?;
    let sp = art::spinner("Resolving project DB URLs…");
    let path = credentials_path(&project_id, branch.as_deref());
    let creds: credentials::Credentials = api::get(&path).await?;
    sp.finish_and_clear();
    credentials::ensure_prisma_contract(&creds)?;

    print_credentials(&creds);
    display::ok("Copy the block above into Prisma .env; DIRECT_URL is for migrations.");
    Ok(())
}

pub(crate) fn credentials_path(project_id: &str, branch: Option<&str>) -> String {
    match branch {
        Some(branch) => format!("/projects/{project_id}/branches/{branch}/credentials"),
        None => format!("/projects/{project_id}/credentials"),
    }
}

pub(crate) fn print_credentials(creds: &credentials::Credentials) {
    let title = match creds.branch_name.as_deref() {
        Some(branch_name) => format!("Prisma URLs: {branch_name} -> {}", creds.database),
        None => format!("Prisma URLs: {}", creds.database),
    };
    display::header(&title);

    if let Some(branch_id) = &creds.branch_id {
        display::kv(&[
            (
                "Branch",
                creds
                    .branch_name
                    .clone()
                    .unwrap_or_else(|| "selected".to_string()),
            ),
            ("Branch ID", branch_id.clone()),
            ("Database", creds.database.clone()),
        ]);
        println!();
    }

    display::kv(&[
        ("DATABASE_URL", creds.database_url.clone()),
        ("DIRECT_URL", creds.direct_url.clone()),
    ]);
    println!(
        "\n{}",
        credentials::format_prisma_env_block(&creds.database_url, &creds.direct_url)
    );
}

#[cfg(test)]
mod tests {
    use super::credentials_path;

    #[test]
    fn builds_project_and_branch_credential_paths() {
        assert_eq!(
            credentials_path("project-1", None),
            "/projects/project-1/credentials"
        );
        assert_eq!(
            credentials_path("project-1", Some("feature-proof")),
            "/projects/project-1/branches/feature-proof/credentials"
        );
    }
}
