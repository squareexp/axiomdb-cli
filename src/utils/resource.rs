use anyhow::{bail, Result};
use serde::Deserialize;
use uuid::Uuid;

use crate::{api, config, utils::disambiguation};

#[derive(Debug, Clone, Deserialize)]
pub struct ProjectRef {
    pub id: String,
    pub name: String,
    pub app_key: String,
    pub env: String,
}

#[derive(Deserialize)]
struct ProjectLookupResponse {
    matches: Vec<ProjectRef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BranchRef {
    pub id: String,
    pub branch_name: String,
    pub database_name: String,
}

#[derive(Deserialize)]
struct BranchLookupResponse {
    matches: Vec<BranchRef>,
}

pub async fn resolve_project(input: Option<&str>) -> Result<String> {
    let candidate = match input.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => value.to_string(),
        None => config::load().current_project.ok_or_else(|| {
            anyhow::anyhow!(
                "No project selected. Pass a project name/ID or run:\n  axm projects use <name>"
            )
        })?,
    };

    if Uuid::parse_str(&candidate).is_ok() {
        return Ok(candidate);
    }

    let response: ProjectLookupResponse = api::get(&format!(
        "/projects/lookup?name={}",
        url::form_urlencoded::byte_serialize(candidate.as_bytes()).collect::<String>()
    ))
    .await?;

    let options = response
        .matches
        .into_iter()
        .map(|project| disambiguation::DisambiguationOption {
            id: project.id,
            name: project.name,
            scope: format!("{} / {}", project.app_key, project.env),
        })
        .collect::<Vec<_>>();

    disambiguation::choose(&format!("Project {candidate:?}"), &options)
}

pub async fn resolve_branch(project_id: &str, input: &str) -> Result<BranchRef> {
    let input = input.trim();
    if input.is_empty() {
        bail!("branch name or ID is required");
    }

    let path = if Uuid::parse_str(input).is_ok() {
        format!("/projects/{project_id}/branches/lookup?id={input}")
    } else {
        format!(
            "/projects/{project_id}/branches/lookup?name={}",
            url::form_urlencoded::byte_serialize(input.as_bytes()).collect::<String>()
        )
    };
    let response: BranchLookupResponse = api::get(&path).await?;

    match response.matches.len() {
        0 => bail!("Branch {input:?} not found in project {project_id}"),
        1 => Ok(response.matches.into_iter().next().expect("one branch")),
        _ => {
            let options = response
                .matches
                .iter()
                .map(|branch| disambiguation::DisambiguationOption {
                    id: branch.id.clone(),
                    name: branch.branch_name.clone(),
                    scope: branch.database_name.clone(),
                })
                .collect::<Vec<_>>();
            let selected = disambiguation::choose(&format!("Branch {input:?}"), &options)?;
            response
                .matches
                .into_iter()
                .find(|branch| branch.id == selected)
                .ok_or_else(|| anyhow::anyhow!("selected branch disappeared"))
        }
    }
}
