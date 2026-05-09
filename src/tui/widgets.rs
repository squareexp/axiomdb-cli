use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::tui::state::TuiState;

pub fn render(frame: &mut Frame, state: &TuiState) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(frame.size());

    render_title(frame, root[0]);
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(34), Constraint::Percentage(66)])
        .split(root[1]);
    render_projects(frame, body[0], state);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(9), Constraint::Min(0)])
        .split(body[1]);
    render_metrics(frame, right[0], state);
    render_events(frame, right[1], state);
    render_help(frame, root[2], state);
}

fn render_title(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            "AxiomDB",
            Style::default()
                .fg(Color::Rgb(255, 140, 0))
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" terminal dashboard"),
    ]))
    .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(title, area);
}

fn render_projects(frame: &mut Frame, area: Rect, state: &TuiState) {
    let items = state
        .projects
        .iter()
        .enumerate()
        .map(|(idx, project)| {
            let marker = if idx == state.selected { "▶ " } else { "  " };
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(marker, Style::default().fg(Color::Rgb(255, 140, 0))),
                    Span::styled(&project.name, Style::default().add_modifier(Modifier::BOLD)),
                ]),
                Line::from(Span::styled(
                    format!("{} / {}  {}", project.app_key, project.env, project.status),
                    Style::default().fg(Color::DarkGray),
                )),
            ])
        })
        .collect::<Vec<_>>();
    let list = List::new(items).block(Block::default().title(" Projects ").borders(Borders::ALL));
    frame.render_widget(list, area);
}

fn render_metrics(frame: &mut Frame, area: Rect, state: &TuiState) {
    let project = state
        .selected_project()
        .map(|project| project.name.clone())
        .unwrap_or_else(|| "No project selected".to_string());
    let lines = if let Some(metrics) = &state.metrics {
        vec![
            Line::from(format!("Project       {project}")),
            Line::from(format!("Database      {}", metrics.database)),
            Line::from(format!("CPU           {:.1}%", metrics.cpu_percent)),
            Line::from(format!(
                "Memory        {} / {} MB",
                metrics.mem_used_mb, metrics.mem_total_mb
            )),
            Line::from(format!("Connections   {}", metrics.pg_active_connections)),
            Line::from(format!(
                "Smoke         {}",
                if metrics.smoke_ok { "pass" } else { "fail" }
            )),
        ]
    } else {
        vec![Line::from("Metrics are loading or unavailable.")]
    };
    let widget = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" Live metrics ")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(widget, area);
}

fn render_events(frame: &mut Frame, area: Rect, state: &TuiState) {
    let items = state
        .events
        .iter()
        .map(|event| {
            let ts = event.created_at.get(..19).unwrap_or(&event.created_at);
            ListItem::new(Line::from(vec![
                Span::styled(format!("{ts}  "), Style::default().fg(Color::DarkGray)),
                Span::styled(&event.action, Style::default().fg(Color::Rgb(255, 140, 0))),
                Span::raw(format!("  {}", event.target_type)),
            ]))
        })
        .collect::<Vec<_>>();
    let list = List::new(items).block(
        Block::default()
            .title(" Recent audit ")
            .borders(Borders::ALL),
    );
    frame.render_widget(list, area);
}

fn render_help(frame: &mut Frame, area: Rect, state: &TuiState) {
    let text = state
        .error
        .as_deref()
        .map(|error| format!("↑/↓ select  r refresh  q quit  |  {error}"))
        .unwrap_or_else(|| "↑/↓ select  r refresh  q quit".to_string());
    let widget = Paragraph::new(text).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(widget, area);
}
