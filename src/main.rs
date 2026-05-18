mod app;
mod apply;
mod config;
mod error;
mod state;
mod system;
mod theme;
mod tui;

use anyhow::Result;
use clap::Parser;

use crate::config::Paths;
use crate::error::CursorError;
use crate::state::current_state;
use crate::system::command::RealCommandRunner;
use crate::theme::discover_themes;

#[derive(Parser, Debug)]
#[command(
    name = "cursors",
    version,
    about = "Full-screen TUI cursor theme manager for Linux",
    long_about = "cursors opens a full-screen TUI for searching installed cursor themes, changing cursor size, and applying the selected cursor theme. All management happens inside the TUI."
)]
struct Cli {}

fn main() -> Result<()> {
    let _cli = Cli::parse();
    let paths = Paths::new()?;
    let commands = RealCommandRunner;
    let themes = discover_themes(&paths)?;
    if themes.is_empty() {
        return Err(CursorError::NoThemes.into());
    }
    let current = current_state(&commands);
    let app = app::App::new(themes, current);

    tui::run(app, paths, commands)
}
