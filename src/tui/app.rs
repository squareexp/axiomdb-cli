use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};

use crate::tui::{state::TuiState, widgets};

pub async fn run() -> Result<()> {
    let mut state = TuiState::load().await?;
    state.select_active_project().await;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, &mut state).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut TuiState,
) -> Result<()> {
    loop {
        terminal.draw(|frame| widgets::render(frame, state))?;

        if event::poll(Duration::from_millis(250))? {
            match event::read()? {
                Event::Key(key) if key.code == KeyCode::Char('q') => break,
                Event::Key(key) if key.code == KeyCode::Char('r') => state.refresh_selected().await,
                Event::Key(key) if key.code == KeyCode::Down => state.select_next().await,
                Event::Key(key) if key.code == KeyCode::Up => state.select_previous().await,
                _ => {}
            }
        }
    }
    Ok(())
}
