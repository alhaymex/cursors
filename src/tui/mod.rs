pub mod event;
pub mod ui;

use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self as ct_event, Event};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::app::App;
use crate::config::Paths;
use crate::system::command::RealCommandRunner;

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> Result<Self> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

pub fn run(mut app: App, paths: Paths, commands: RealCommandRunner) -> Result<()> {
    let _guard = TerminalGuard::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    while !app.should_quit {
        terminal.draw(|frame| ui::draw(frame, &mut app))?;

        if ct_event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = ct_event::read()? {
                event::handle_key(key, &mut app, &paths, &commands)?;
            }
        }
    }

    Ok(())
}
