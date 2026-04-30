use anyhow::Result;
use clap::Subcommand;
use dialoguer::{Confirm, Input, Select};
use serde::{Deserialize, Serialize};

use crate::{api, art, config, display};

#[derive(Subcommand)]
pub enum ProjectsCmd {
    /// List all projects
    #[clap(alias = "ls", short_flag = 'l')]
    List,
    /// Create a new project (provisions a real DB on the VPS)
    Create {
        #[arg(short, long)] name: Option<String>,
        #[arg(short = 'k', long)] app_key: Option<String>,
        #[arg(short, long)] env: Option<String>,
    },
    /// Show details for a project
    Get {
        /// Project ID (omit to use current context)
        project_id: Option<String>,
    },
    /// Set the active project context (saves to ~/.config/axiom/config.json)
    Use { project_id: String },
    /// Show the currently selected project
    Current,
    /// Clear the active project context
    Unset,
}

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Deserialize, Debug)]
struct Project {
    id: String,
    name: String,
    app_key: String,
    env: String,
    status: String,
    created_at: String,
}

#[derive(Deserialize)]
struct ProjectListResponse {
    projects: Vec<Project>,
}

#[derive(Deserialize)]
struct ProjectDatabase {
    database_name: String,
    runtime_key: String,
    direct_key: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct ProjectOut {
    id: String,
    slug: String,
    name: String,
    app_key: String,
    env: String,
}

#[derive(Deserialize)]
struct DatabaseOut {
    database_name: String,
    runtime_key: String,
    direct_key: String,
}

#[derive(Deserialize)]
struct CreateProjectResponse {
    project: ProjectOut,
    database: DatabaseOut,
}

#[derive(Deserialize)]
struct ProjectDetailResponse {
    project: Project,
    databases: Vec<ProjectDatabase>,
}

#[derive(Serialize)]
struct CreateProjectRequest {
    name: String,
    app_key: String,
    env: String,
}

// ── Dispatch ──────────────────────────────────────────────────────────────────

pub async fn run(cmd: ProjectsCmd) -> Result<()> {
    match cmd {
        ProjectsCmd::List                     => list().await,
        ProjectsCmd::Create { name, app_key, env } => create(name, app_key, env).await,
        ProjectsCmd::Get    { project_id }    => get(project_id).await,
        ProjectsCmd::Use    { project_id }    => use_project(project_id),
        ProjectsCmd::Current                  => current_project(),
        ProjectsCmd::Unset                    => unset_project(),
    }
}

// ── list ─────────────────────────────────────────────────────────────────────

async fn list() -> Result<()> {
    let sp = art::spinner("Fetching projects…");
    let res: ProjectListResponse = api::get("/projects").await?;
    sp.finish_and_clear();

    let current = config::load().current_project;

    display::header(&format!("Projects ({})", res.projects.len()));

    if res.projects.is_empty() {
        println!("  No projects yet. Run: axiom projects create");
        return Ok(());
    }

    display::table(
        &["", "Name", "App key", "Env", "Status", "Created"],
        res.projects
            .iter()
            .map(|p| {
                let active = current.as_deref() == Some(p.id.as_str());
                vec![
                    if active { "▶".to_string() } else { " ".to_string() },
                    p.name.clone(),
                    p.app_key.clone(),
                    p.env.clone(),
                    display::status_color(&p.status),
                    p.created_at[..10].to_string(),
                ]
            })
            .collect(),
    );

    if let Some(id) = &current {
        println!("  {} active project: {}", "▶".truecolor_str(255, 140, 0), id);
    } else {
        println!("  {} Run {} to set a project context.",
            "tip:".truecolor_str(100, 100, 100),
            "axiom projects use <id>".truecolor_str(255, 140, 0)
        );
    }
    Ok(())
}

// ── create ────────────────────────────────────────────────────────────────────

async fn create(name: Option<String>, app_key: Option<String>, env: Option<String>) -> Result<()> {
    let name: String = match name {
        Some(n) => n,
        None => Input::new()
            .with_prompt("Project display name")
            .validate_with(|v: &String| {
                if v.trim().is_empty() { Err("Required") } else { Ok(()) }
            })
            .interact_text()?,
    };

    let default_key = name.to_lowercase().replace(' ', "_");
    let app_key: String = match app_key {
        Some(k) => k,
        None => Input::new()
            .with_prompt("App key (slug)")
            .default(default_key)
            .validate_with(|v: &String| {
                if v.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
                    Ok(())
                } else {
                    Err("Lowercase letters, digits, underscores only")
                }
            })
            .interact_text()?,
    };

    let env: String = match env {
        Some(e) => e,
        None => {
            let envs = ["prod", "staging", "dev"];
            let i = Select::new()
                .with_prompt("Environment")
                .items(&envs)
                .default(0)
                .interact()?;
            envs[i].to_string()
        }
    };

    let confirm = Confirm::new()
        .with_prompt(format!(
            "Create project \"{app_key}-{env}\" on {}?",
            config::load().base_url
        ))
        .default(true)
        .interact()?;
    if !confirm { return Ok(()); }

    let sp = art::pulse_spinner("Provisioning database on VPS… this may take a moment");
    let res: CreateProjectResponse =
        api::post("/projects", &CreateProjectRequest { name, app_key, env }).await?;
    sp.finish_and_clear();

    // Auto-set as current project
    config::set_current_project(&res.project.id)?;

    display::header("Project created");
    display::kv(&[
        ("Project ID",       res.project.id.clone()),
        ("Slug",             res.project.slug.clone()),
        ("Name",             res.project.name.clone()),
        ("Database",         res.database.database_name.clone()),
        ("Runtime key",      res.database.runtime_key.clone()),
        ("Direct key",       res.database.direct_key.clone()),
    ]);
    art::step_ok(&format!(
        "Project set as active context  ({})",
        res.project.id.truecolor_str(150, 150, 150)
    ));
    Ok(())
}

// ── get ───────────────────────────────────────────────────────────────────────

async fn get(project_id: Option<String>) -> Result<()> {
    let id = config::resolve_project(project_id.as_deref())?;
    let sp = art::spinner("Loading project…");
    let res: ProjectDetailResponse = api::get(&format!("/projects/{id}")).await?;
    sp.finish_and_clear();

    display::header(&format!("Project: {}", res.project.name));
    display::kv(&[
        ("ID",          res.project.id.clone()),
        ("App key",     res.project.app_key.clone()),
        ("Environment", res.project.env.clone()),
        ("Status",      display::status_color(&res.project.status)),
        ("Created",     res.project.created_at[..19].to_string()),
    ]);

    if !res.databases.is_empty() {
        println!();
        display::table(
            &["Database", "Runtime key", "Direct key"],
            res.databases
                .iter()
                .map(|d| vec![d.database_name.clone(), d.runtime_key.clone(), d.direct_key.clone()])
                .collect(),
        );
    }
    Ok(())
}

// ── use / current / unset ─────────────────────────────────────────────────────

fn use_project(project_id: String) -> Result<()> {
    config::set_current_project(&project_id)?;
    display::ok(&format!("Active project set to {project_id}"));
    println!("  You can now omit the project ID in all commands.");
    Ok(())
}

fn current_project() -> Result<()> {
    let cfg = config::load();
    match cfg.current_project {
        Some(id) => {
            art::section("Active project");
            display::kv(&[("Project ID", id), ("Server", cfg.base_url)]);
            println!();
        }
        None => {
            display::info("No active project. Run: axiom projects use <id>");
        }
    }
    Ok(())
}

fn unset_project() -> Result<()> {
    config::clear_current_project()?;
    display::ok("Active project context cleared.");
    Ok(())
}

// ── Colour helper for &str ────────────────────────────────────────────────────

trait TrueColorStr {
    fn truecolor_str(&self, r: u8, g: u8, b: u8) -> String;
}

impl TrueColorStr for str {
    fn truecolor_str(&self, r: u8, g: u8, b: u8) -> String {
        use colored::Colorize;
        self.truecolor(r, g, b).to_string()
    }
}
