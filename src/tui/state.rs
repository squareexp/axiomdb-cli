use anyhow::Result;
use serde::Deserialize;

use crate::{api, utils::resource};

#[derive(Debug, Clone, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub app_key: String,
    pub env: String,
    pub status: String,
}

#[derive(Deserialize)]
struct ProjectListResponse {
    projects: Vec<Project>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct MonitoringSummary {
    pub database: String,
    pub cpu_percent: f64,
    pub mem_used_mb: u64,
    pub mem_total_mb: u64,
    pub pg_active_connections: i64,
    pub smoke_ok: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuditEvent {
    pub action: String,
    pub target_type: String,
    pub created_at: String,
}

#[derive(Deserialize)]
struct AuditListResponse {
    events: Vec<AuditEvent>,
}

#[derive(Debug, Default)]
pub struct TuiState {
    pub projects: Vec<Project>,
    pub selected: usize,
    pub metrics: Option<MonitoringSummary>,
    pub events: Vec<AuditEvent>,
    pub error: Option<String>,
}

impl TuiState {
    pub async fn load() -> Result<Self> {
        let projects: ProjectListResponse = api::get("/projects").await?;
        let mut state = Self {
            projects: projects.projects,
            selected: 0,
            metrics: None,
            events: Vec::new(),
            error: None,
        };
        state.refresh_selected().await;
        Ok(state)
    }

    pub async fn refresh_selected(&mut self) {
        self.error = None;
        let Some(project) = self.projects.get(self.selected).cloned() else {
            return;
        };

        match api::get::<MonitoringSummary>(&format!("/projects/{}/monitoring/summary", project.id))
            .await
        {
            Ok(metrics) => self.metrics = Some(metrics),
            Err(error) => self.error = Some(error.to_string()),
        }

        match api::get::<AuditListResponse>("/audit?limit=10").await {
            Ok(events) => self.events = events.events,
            Err(error) => self.error = Some(error.to_string()),
        }
    }

    pub async fn select_next(&mut self) {
        if self.projects.is_empty() {
            return;
        }
        self.selected = (self.selected + 1).min(self.projects.len() - 1);
        self.refresh_selected().await;
    }

    pub async fn select_previous(&mut self) {
        if self.projects.is_empty() {
            return;
        }
        self.selected = self.selected.saturating_sub(1);
        self.refresh_selected().await;
    }

    pub fn selected_project(&self) -> Option<&Project> {
        self.projects.get(self.selected)
    }

    pub async fn select_active_project(&mut self) {
        if let Ok(active) = resource::resolve_project(None).await {
            if let Some(index) = self
                .projects
                .iter()
                .position(|project| project.id == active)
            {
                self.selected = index;
                self.refresh_selected().await;
            }
        }
    }
}
