use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::io::{self, Write};

mod analysis;
mod app;
mod config;
mod db;
mod history;
mod ui;

use app::App;

fn cleanup_terminal<B: Backend + std::io::Write>(terminal: &mut Terminal<B>) -> Result<()> {
    // Disable raw mode first
    disable_raw_mode()?;

    // Clear the screen completely and reset terminal state
    execute!(
        terminal.backend_mut(),
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
        crossterm::terminal::Clear(crossterm::terminal::ClearType::Purge),
        crossterm::cursor::MoveTo(0, 0),
        LeaveAlternateScreen,
        DisableMouseCapture,
        crossterm::style::ResetColor,
        crossterm::cursor::Show
    )?;

    // Show cursor and flush
    terminal.show_cursor()?;
    terminal.flush()?;

    // Additional cleanup - print reset sequence
    print!("\x1b[0m\x1b[?25h\x1b[2J\x1b[H");
    std::io::stdout().flush()?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let app = App::new().await?;
    let res = run_app(&mut terminal, app).await;

    // Restore terminal - ensure cleanup happens even on error
    let cleanup_result = cleanup_terminal(&mut terminal);

    if let Err(err) = res {
        eprintln!("Application error: {:?}", err);
    }

    if let Err(err) = cleanup_result {
        eprintln!("Terminal cleanup error: {:?}", err);
    }

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    let mut last_analytics_update = std::time::Instant::now();

    loop {
        // Update analytics periodically in background
        let now = std::time::Instant::now();
        if now.duration_since(last_analytics_update).as_secs() > 60 {
            app.update_analytics_background();
            last_analytics_update = now;
        }

        terminal.draw(|f| ui::draw(f, &app))?;

        // Use timeout to allow periodic updates
        if let Ok(event) = event::poll(std::time::Duration::from_millis(100)) {
            if event {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(()),
                        KeyCode::Char('/') => app.go_to_search_tab(),
                        KeyCode::Char('?') => app.toggle_help(),
                        KeyCode::Tab => app.next_tab(),
                        KeyCode::BackTab => app.previous_tab(),
                        KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),
                        KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
                        KeyCode::Left | KeyCode::Char('h') => app.scroll_left(),
                        KeyCode::Right | KeyCode::Char('l') => app.scroll_right(),
                        KeyCode::Enter => app.handle_enter(),
                        KeyCode::Esc => app.handle_escape(),
                        KeyCode::Home => app.scroll_to_top(),
                        KeyCode::End => app.scroll_to_bottom(),
                        KeyCode::PageUp => app.page_up(),
                        KeyCode::PageDown => app.page_down(),
                        KeyCode::Char(c @ '1'..='9') => {
                            let tab_index = (c as u8 - b'1') as usize;
                            app.jump_to_tab(tab_index);
                        }
                        KeyCode::Char('0') => app.jump_to_tab(9), // Packages tab
                        KeyCode::Char('-') => app.jump_to_tab(10), // Experiments tab
                        KeyCode::F(1) => app.handle_function_key(1),
                        KeyCode::F(2) => app.handle_function_key(2),
                        KeyCode::F(3) => app.handle_function_key(3),
                        KeyCode::F(4) => app.handle_function_key(4),
                        KeyCode::F(5) => app.refresh_analytics(), // Manual refresh
                        KeyCode::Char('r') | KeyCode::Char('R') => {
                            app.invalidate_analytics_cache();
                            app.refresh_analytics();
                        }
                        KeyCode::Char(c) => app.handle_char(c),
                        KeyCode::Backspace => app.handle_backspace(),
                        _ => {}
                    }
                }
            }
        }
    }
}
