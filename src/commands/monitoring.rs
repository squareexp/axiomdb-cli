use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;
use futures_util::StreamExt;
use serde::Deserialize;

use crate::{api, art, display, utils::resource};

#[derive(Subcommand)]
pub enum MonitoringCmd {
    /// Show current health metrics for a project
    #[clap(visible_alias = "sum")]
    Summary { project_ref: Option<String> },
    /// Stream live metrics (SSE — Ctrl+C to stop)
    #[clap(visible_alias = "st")]
    Stream { project_ref: Option<String> },
}

#[derive(Deserialize)]
struct MonitoringSummary {
    database: String,
    cpu_percent: f64,
    mem_used_mb: u64,
    mem_total_mb: u64,
    pg_active_connections: i64,
    smoke_ok: bool,
}

pub async fn run(cmd: MonitoringCmd) -> Result<()> {
    match cmd {
        MonitoringCmd::Summary { project_ref } => summary(project_ref).await,
        MonitoringCmd::Stream { project_ref } => stream(project_ref).await,
    }
}

async fn summary(project_ref: Option<String>) -> Result<()> {
    let project_id = resource::resolve_project(project_ref.as_deref()).await?;
    let sp = art::spinner("Fetching metrics…");
    let s: MonitoringSummary =
        api::get(&format!("/projects/{project_id}/monitoring/summary")).await?;
    sp.finish_and_clear();

    display::header(&format!("Monitoring — {}", s.database));
    display::kv(&[
        ("Database", s.database.clone()),
        ("CPU usage", format!("{:.1}%", s.cpu_percent)),
        (
            "Memory used",
            format!("{} / {} MB", s.mem_used_mb, s.mem_total_mb),
        ),
        ("PG active connections", s.pg_active_connections.to_string()),
        (
            "Smoke test",
            if s.smoke_ok {
                "✔ pass".green().to_string()
            } else {
                "✖ fail".red().to_string()
            },
        ),
    ]);
    Ok(())
}

async fn stream(project_ref: Option<String>) -> Result<()> {
    let project_id = resource::resolve_project(project_ref.as_deref()).await?;
    let path = format!("/projects/{project_id}/monitoring/stream");
    display::info(&format!(
        "Streaming metrics for {project_id}  (Ctrl+C to stop)\n"
    ));

    let res = api::stream_sse(&path).await?;
    let mut stream = res.bytes_stream();

    let mut buf = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let text = String::from_utf8_lossy(&chunk);
        buf.push_str(&text);

        // Process complete SSE lines
        while let Some(pos) = buf.find('\n') {
            let line = buf[..pos].trim().to_string();
            buf = buf[pos + 1..].to_string();

            if let Some(data) = line.strip_prefix("data:") {
                let data = data.trim();
                if data == "ping" || data.is_empty() {
                    continue;
                }
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(data) {
                    let ts = v["ts"].as_str().unwrap_or("").get(..19).unwrap_or("");
                    let cpu = v["cpu_percent"].as_f64().unwrap_or(0.0);
                    let mem = v["mem_used_mb"].as_u64().unwrap_or(0);
                    println!(
                        "  {}  {}  {}",
                        ts.dimmed(),
                        format!("CPU {:5.1}%", cpu).green(),
                        format!("MEM {:5} MB", mem).blue()
                    );
                }
            }
        }
    }
    Ok(())
}
