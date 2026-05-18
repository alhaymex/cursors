pub mod default_icon;
pub mod flatpak;
pub mod gtk;
pub mod hyprland;
pub mod xresources;

use anyhow::Result;

use crate::config::Paths;
use crate::state::current_state;
use crate::system::backup::BackupSession;
use crate::system::command::CommandRunner;
use crate::theme::{CursorTheme, ThemeSource};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyReport {
    pub target: &'static str,
    pub changed: bool,
    pub skipped: bool,
    pub message: String,
}

impl ApplyReport {
    pub fn changed(target: &'static str, message: impl Into<String>) -> Self {
        Self {
            target,
            changed: true,
            skipped: false,
            message: message.into(),
        }
    }

    pub fn unchanged(target: &'static str, message: impl Into<String>) -> Self {
        Self {
            target,
            changed: false,
            skipped: false,
            message: message.into(),
        }
    }

    pub fn skipped(target: &'static str, message: impl Into<String>) -> Self {
        Self {
            target,
            changed: false,
            skipped: true,
            message: message.into(),
        }
    }
}

pub fn apply_cursor(
    theme: &CursorTheme,
    size: u32,
    paths: &Paths,
    commands: &dyn CommandRunner,
) -> Result<Vec<ApplyReport>> {
    let previous = current_state(commands);
    let backup = BackupSession::new(&paths.backup_dir, &previous.theme, previous.size)?;
    let reports = vec![
        default_icon::apply(theme, paths, &backup)?,
        gtk::apply(theme, size, commands)?,
        xresources::apply(theme, size, paths, commands, &backup)?,
        hyprland::apply(theme, size, paths, commands, &backup)?,
        flatpak::apply(theme, size, paths, commands)?,
    ];
    backup.finish_metadata(&previous.theme, previous.size)?;
    Ok(reports)
}

pub fn apply_runtime_state(
    theme: &str,
    size: u32,
    paths: &Paths,
    commands: &dyn CommandRunner,
) -> Result<Vec<ApplyReport>> {
    let theme = CursorTheme {
        name: theme.to_string(),
        display_name: theme.to_string(),
        source_dir: paths.home.join(".icons").join(theme),
        source_kind: ThemeSource::Icons,
    };

    Ok(vec![
        gtk::apply(&theme, size, commands)?,
        xresources::apply_runtime(paths, commands)?,
        hyprland::apply_runtime(&theme.name, size, commands)?,
        flatpak::apply(&theme, size, paths, commands)?,
    ])
}

pub fn summarize_apply(theme: &str, size: u32, reports: &[ApplyReport]) -> String {
    let skipped = reports
        .iter()
        .filter(|report| report.skipped)
        .map(|report| report.target)
        .collect::<Vec<_>>();

    if skipped.is_empty() {
        format!("Applied {theme} at size {size}")
    } else {
        format!("Applied {theme}; skipped {}", skipped.join(", "))
    }
}
