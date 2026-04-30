use anyhow::Result;
use clap::Subcommand;
use serde::Deserialize;
use serde_json::Value;

use crate::{api, art, display};

#[derive(Subcommand)]
pub enum AuditCmd {
    /// List recent audit events
    #[clap(alias = "ls")]
    List {
        #[arg(short, long, default_value_t = 50)]
        limit: u32,
    },
}

#[derive(Deserialize)]
struct AuditEvent {
    actor_user_id: Option<String>,
    action: String,
    target_type: String,
    target_id: Option<String>,
    created_at: String,
}

#[derive(Deserialize)]
struct AuditListResponse {
    events: Vec<AuditEvent>,
}

pub async fn run(cmd: AuditCmd) -> Result<()> {
    match cmd {
        AuditCmd::List { limit } => list(limit).await,
    }
}

async fn list(limit: u32) -> Result<()> {
    let sp = art::spinner("Loading audit log…");
    let res: AuditListResponse = api::get(&format!("/audit?limit={limit}")).await?;
    sp.finish_and_clear();

    display::header(&format!("Audit log ({} events)", res.events.len()));
    display::table(
        &["Time", "Actor", "Action", "Target"],
        res.events
            .iter()
            .map(|e| {
                let ts = e.created_at.get(..19).unwrap_or(&e.created_at).to_string();
                let actor = e
                    .actor_user_id
                    .as_deref()
                    .map(|id| id.get(..8).unwrap_or(id).to_string())
                    .unwrap_or_else(|| "—".to_string());
                let target = match &e.target_id {
                    Some(id) => format!("{}:{}", e.target_type, id.get(..8).unwrap_or(id)),
                    None => e.target_type.clone(),
                };
                vec![ts, actor, e.action.clone(), target]
            })
            .collect(),
    );
    Ok(())
}

// ── Jobs ──────────────────────────────────────────────────────────────────────

#[derive(Subcommand)]
pub enum JobsCmd {
    /// Show status and output of a job
    Get { job_id: String },
}

#[derive(Deserialize)]
struct Job {
    id: String,
    action: String,
    status: String,
    error_text: Option<String>,
    output: Value,
    created_at: String,
    finished_at: Option<String>,
}

pub async fn run_jobs(cmd: JobsCmd) -> Result<()> {
    match cmd {
        JobsCmd::Get { job_id } => get_job(job_id).await,
    }
}

async fn get_job(job_id: String) -> Result<()> {
    let sp = art::spinner("Fetching job…");
    let job: Job = api::get(&format!("/jobs/{job_id}")).await?;
    sp.finish_and_clear();

    display::header(&format!("Job {}", job.id));
    display::kv(&[
        ("Action", job.action),
        ("Status", display::status_color(&job.status)),
        ("Created", job.created_at[..19].to_string()),
        (
            "Finished",
            job.finished_at
                .as_deref()
                .map(|s| s.get(..19).unwrap_or(s).to_string())
                .unwrap_or_else(|| "—".to_string()),
        ),
        ("Error", display::opt(job.error_text.as_deref())),
    ]);

    if !job.output.is_null() && job.output != serde_json::json!({}) {
        println!("\nOutput:");
        println!("{}", serde_json::to_string_pretty(&job.output)?);
    }
    Ok(())
}
