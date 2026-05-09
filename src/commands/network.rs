use anyhow::{bail, Result};
use clap::Subcommand;
use colored::Colorize;
use dialoguer::Confirm;
use serde::{Deserialize, Serialize};

use crate::{api, art, display, utils};

#[derive(Subcommand)]
pub enum NetworkCmd {
    /// Show the public IP AxiomDB sees for this machine
    #[clap(visible_alias = "ip")]
    CurrentIp,
    /// List network rules for a project
    #[clap(visible_alias = "ls")]
    List {
        /// Project name/ID (omit to use active context)
        project: Option<String>,
    },
    /// Allow a CIDR, or this machine IP with --current
    #[clap(visible_alias = "add")]
    Allow {
        /// Project name/ID (omit to use active context)
        #[arg(short, long)]
        project: Option<String>,
        /// Allow this machine's public IP as /32
        #[arg(long)]
        current: bool,
        /// CIDR to allow, for example 203.0.113.10/32
        cidr: Option<String>,
        /// Ports to open: runtime, direct, both
        #[arg(long, default_value = "both")]
        ports: String,
        /// Label shown in the dashboard
        #[arg(short, long)]
        label: Option<String>,
        /// TTL: 24h, 7d, 30d, 1m, 1y, forever
        #[arg(long, default_value = "7d")]
        expires_in: String,
        /// Skip confirmation
        #[arg(short, long)]
        yes: bool,
    },
    /// Revoke a configured network rule
    #[clap(visible_alias = "rm", visible_alias = "delete")]
    Revoke {
        /// Rule ID
        rule_id: String,
        /// Project name/ID (omit to use active context)
        #[arg(short, long)]
        project: Option<String>,
        /// Skip confirmation
        #[arg(short, long)]
        yes: bool,
    },
    /// Set network mode: restricted, public_runtime, public_all
    #[clap(visible_alias = "mode")]
    PublicMode {
        mode: String,
        /// Project name/ID (omit to use active context)
        #[arg(short, long)]
        project: Option<String>,
        /// Skip confirmation
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(Deserialize)]
struct CurrentIpResponse {
    ip: String,
    suggested_cidr: String,
}

#[derive(Deserialize)]
struct RuleListResponse {
    policy: serde_json::Value,
    rules: Vec<NetworkRule>,
}

#[derive(Deserialize)]
struct NetworkRule {
    id: String,
    cidr: String,
    label: String,
    ports: String,
    scope: String,
    expires_at: Option<String>,
}

#[derive(Serialize)]
struct CreateRuleRequest {
    cidr: String,
    label: String,
    ports: String,
    scope: String,
    expires_in: String,
}

#[derive(Serialize)]
struct PublicModeRequest {
    mode: String,
    confirm: Option<String>,
}

pub async fn run(cmd: NetworkCmd) -> Result<()> {
    match cmd {
        NetworkCmd::CurrentIp => current_ip().await,
        NetworkCmd::List { project } => list(project).await,
        NetworkCmd::Allow {
            project,
            current,
            cidr,
            ports,
            label,
            expires_in,
            yes,
        } => allow(project, current, cidr, ports, label, expires_in, yes).await,
        NetworkCmd::Revoke {
            rule_id,
            project,
            yes,
        } => revoke(project, rule_id, yes).await,
        NetworkCmd::PublicMode { mode, project, yes } => public_mode(project, mode, yes).await,
    }
}

async fn current_ip() -> Result<()> {
    let sp = art::spinner("Detecting current IP…");
    let response: CurrentIpResponse = api::get("/network/current-ip").await?;
    sp.finish_and_clear();
    display::header("Current machine IP");
    display::kv(&[("IP", response.ip), ("CIDR", response.suggested_cidr)]);
    Ok(())
}

async fn list(project: Option<String>) -> Result<()> {
    let project_id = utils::resource::resolve_project(project.as_deref()).await?;
    let sp = art::spinner("Loading network policy…");
    let response: RuleListResponse =
        api::get(&format!("/projects/{project_id}/network/rules")).await?;
    sp.finish_and_clear();

    display::header("Network access");
    let mode = response
        .policy
        .get("mode")
        .and_then(|value| value.as_str())
        .unwrap_or("restricted");
    display::kv(&[("Project", project_id), ("Mode", mode.to_string())]);

    display::table(
        &["Rule", "CIDR", "Ports", "Scope", "Expires"],
        response
            .rules
            .iter()
            .map(|rule| {
                vec![
                    format!("{} ({})", rule.label, short_id(&rule.id)),
                    rule.cidr.clone(),
                    rule.ports.clone(),
                    rule.scope.clone(),
                    rule.expires_at
                        .as_deref()
                        .map(|value| value.get(..19).unwrap_or(value).to_string())
                        .unwrap_or_else(|| "never".to_string()),
                ]
            })
            .collect(),
    );
    Ok(())
}

async fn allow(
    project: Option<String>,
    current: bool,
    cidr: Option<String>,
    ports: String,
    label: Option<String>,
    expires_in: String,
    yes: bool,
) -> Result<()> {
    let project_id = utils::resource::resolve_project(project.as_deref()).await?;
    let cidr = if current {
        utils::ip::current_cidr_from_gateway_or_ipify().await?
    } else {
        cidr.ok_or_else(|| anyhow::anyhow!("Pass a CIDR or use --current"))?
    };
    let label = label.unwrap_or_else(|| {
        if current {
            "My current IP".to_string()
        } else {
            "CLI allowlist".to_string()
        }
    });

    if !yes {
        let ok = Confirm::new()
            .with_prompt(format!(
                "Allow {} on {} ports for project {}?",
                cidr.truecolor(255, 195, 60),
                ports,
                project_id
            ))
            .default(true)
            .interact()?;
        if !ok {
            return Ok(());
        }
    }

    let sp = art::spinner("Applying network access…");
    let _: serde_json::Value = api::post(
        &format!("/projects/{project_id}/network/rules"),
        &CreateRuleRequest {
            cidr: cidr.clone(),
            label,
            ports,
            scope: "project".to_string(),
            expires_in,
        },
    )
    .await?;
    sp.finish_and_clear();
    display::ok(&format!(
        "Allowed {cidr}. Prisma can now reach AxiomDB from this network."
    ));
    Ok(())
}

async fn revoke(project: Option<String>, rule_id: String, yes: bool) -> Result<()> {
    let project_id = utils::resource::resolve_project(project.as_deref()).await?;
    if !yes {
        let ok = Confirm::new()
            .with_prompt(format!("Revoke network rule {}?", short_id(&rule_id)))
            .default(false)
            .interact()?;
        if !ok {
            return Ok(());
        }
    }
    let sp = art::spinner("Revoking network access…");
    api::delete_req(&format!("/projects/{project_id}/network/rules/{rule_id}")).await?;
    sp.finish_and_clear();
    display::ok("Network rule revoked.");
    Ok(())
}

async fn public_mode(project: Option<String>, mode: String, yes: bool) -> Result<()> {
    let project_id = utils::resource::resolve_project(project.as_deref()).await?;
    let confirm = match mode.as_str() {
        "restricted" => None,
        "public_runtime" => Some("make runtime public".to_string()),
        "public_all" => Some("make direct public".to_string()),
        _ => bail!("mode must be restricted, public_runtime, or public_all"),
    };
    if !yes {
        let ok = Confirm::new()
            .with_prompt(format!("Set project {project_id} network mode to {mode}?"))
            .default(mode == "restricted")
            .interact()?;
        if !ok {
            return Ok(());
        }
    }
    let sp = art::spinner("Applying network mode…");
    let _: serde_json::Value = api::post(
        &format!("/projects/{project_id}/network/public-mode"),
        &PublicModeRequest { mode, confirm },
    )
    .await?;
    sp.finish_and_clear();
    display::ok("Network mode updated.");
    Ok(())
}

fn short_id(value: &str) -> String {
    value.get(..8).unwrap_or(value).to_string()
}
