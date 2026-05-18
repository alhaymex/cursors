use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, BrowserView, PendingAction, StatusMessage, VisibleItem};
use crate::apply::{apply_cursor, apply_runtime_state, summarize_apply};
use crate::config::Paths;
use crate::state::CursorState;
use crate::system::backup;
use crate::system::command::{CommandRunner, RealCommandRunner};
use crate::theme::discover_themes;

pub fn handle_key(
    key: KeyEvent,
    app: &mut App,
    paths: &Paths,
    commands: &RealCommandRunner,
) -> Result<()> {
    if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('c')) {
        app.should_quit = true;
        return Ok(());
    }

    if app.show_help && key.code != KeyCode::Char('?') {
        app.show_help = false;
        return Ok(());
    }

    if app.pending_action.is_some() {
        handle_confirmation_key(key, app, paths, commands)?;
        return Ok(());
    }

    if app.filter_mode {
        handle_filter_key(key, app);
        return Ok(());
    }

    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Down | KeyCode::Char('j') => app.move_down(),
        KeyCode::Up | KeyCode::Char('k') => app.move_up(),
        KeyCode::Char('/') => app.filter_mode = true,
        KeyCode::Char('?') => app.show_help = !app.show_help,
        KeyCode::Char('+') | KeyCode::Char('=') => app.increase_size(),
        KeyCode::Char('-') => app.decrease_size(),
        KeyCode::Char('d') => app.reset_size(),
        KeyCode::Char('r') => rescan(app, paths)?,
        KeyCode::Char('b') => show_backups(app, paths)?,
        KeyCode::Char('u') => restore_selected_backup(app),
        KeyCode::Char(' ') => open_selected_dir(app, commands)?,
        KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('h') => app.back(),
        KeyCode::Char('l') => enter_or_apply(app),
        KeyCode::Enter => {
            if matches!(app.view, BrowserView::Backups) {
                restore_selected_backup(app);
            } else {
                enter_or_apply(app);
            }
        }
        _ => {}
    }

    app.clamp_selection();
    Ok(())
}

fn handle_filter_key(key: KeyEvent, app: &mut App) {
    match key.code {
        KeyCode::Esc | KeyCode::Enter => app.filter_mode = false,
        KeyCode::Backspace => {
            app.filter.pop();
        }
        KeyCode::Char(char) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.filter.push(char);
        }
        _ => {}
    }

    app.clamp_selection();
}

fn enter_or_apply(app: &mut App) {
    if let Some(item @ VisibleItem::Theme(_)) = app.selected_item() {
        if let Some(theme) = app.theme_for_item(item).cloned() {
            app.pending_action = Some(PendingAction::ApplyTheme {
                theme_index: match item {
                    VisibleItem::Theme(index) => index,
                },
                size: app.size,
            });
            app.status =
                StatusMessage::info(format!("Confirm applying {} @ {}", theme.name, app.size));
        }
    }
}

fn rescan(app: &mut App, paths: &Paths) -> Result<()> {
    let themes = discover_themes(paths)?;
    let count = themes.len();
    app.replace_themes(themes);
    app.status = StatusMessage::info(format!("Rescanned {count} cursor themes"));
    Ok(())
}

fn show_backups(app: &mut App, paths: &Paths) -> Result<()> {
    let backups = backup::list_backups(&paths.backup_dir)?;
    let count = backups.len();
    app.show_backups(backups);
    app.status = StatusMessage::info(format!("Loaded {count} backups"));

    Ok(())
}

fn restore_selected_backup(app: &mut App) {
    if !matches!(app.view, BrowserView::Backups) {
        app.status = StatusMessage::info("Press b to view backups");
        return;
    }

    let Some(index) = app.selected_backup_index() else {
        app.status = StatusMessage::info("No backup selected");
        return;
    };
    let Some(entry) = app.backups.get(index) else {
        app.status = StatusMessage::info("No backup selected");
        return;
    };

    app.pending_action = Some(PendingAction::RestoreBackup {
        backup_index: index,
    });
    app.status = StatusMessage::info(format!("Confirm restoring {}", entry.id));
}

fn apply_theme_now(
    app: &mut App,
    theme_index: usize,
    size: u32,
    paths: &Paths,
    commands: &RealCommandRunner,
) {
    let Some(theme) = app.themes.get(theme_index).cloned() else {
        app.status = StatusMessage::error("Selected theme is no longer available");
        return;
    };

    match apply_cursor(&theme, size, paths, commands) {
        Ok(reports) => {
            app.current = CursorState {
                theme: theme.name.clone(),
                size,
            };
            app.status = StatusMessage::info(summarize_apply(&theme.name, size, &reports));
        }
        Err(error) => {
            app.status = StatusMessage::error(format!("Failed: {error}"));
        }
    }
}

fn restore_backup_now(
    app: &mut App,
    backup_index: usize,
    paths: &Paths,
    commands: &RealCommandRunner,
) {
    let Some(entry) = app.backups.get(backup_index).cloned() else {
        app.status = StatusMessage::error("Selected backup is no longer available");
        return;
    };

    match backup::restore_backup(&entry).and_then(|restored| {
        apply_runtime_state(&entry.theme, entry.size, paths, commands).map(|_| restored)
    }) {
        Ok(restored) => {
            app.current = CursorState {
                theme: entry.theme.clone(),
                size: entry.size,
            };
            app.status = StatusMessage::info(format!(
                "Restored {restored} files and applied {} @ {}",
                entry.theme, entry.size
            ));
        }
        Err(error) => app.status = StatusMessage::error(format!("Restore failed: {error}")),
    }
}

fn handle_confirmation_key(
    key: KeyEvent,
    app: &mut App,
    paths: &Paths,
    commands: &RealCommandRunner,
) -> Result<()> {
    match key.code {
        KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
            if let Some(action) = app.pending_action.take() {
                match action {
                    PendingAction::ApplyTheme { theme_index, size } => {
                        apply_theme_now(app, theme_index, size, paths, commands);
                    }
                    PendingAction::RestoreBackup { backup_index } => {
                        restore_backup_now(app, backup_index, paths, commands);
                    }
                }
            }
        }
        KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') => {
            app.clear_pending();
            app.status = StatusMessage::info("Cancelled");
        }
        _ => {}
    }
    Ok(())
}

fn open_selected_dir(app: &mut App, commands: &RealCommandRunner) -> Result<()> {
    let Some(item) = app.selected_item() else {
        app.status = StatusMessage::info("Select a theme to open its directory");
        return Ok(());
    };
    let Some(theme) = app.theme_for_item(item) else {
        app.status = StatusMessage::info("Select a theme first");
        return Ok(());
    };
    let path = theme.source_dir.to_string_lossy().to_string();

    if commands.spawn_detached("xdg-open", &[&path])? {
        app.status = StatusMessage::info(format!("Opened {}", theme.name));
    } else {
        app.status = StatusMessage::error("xdg-open is unavailable");
    }

    Ok(())
}
