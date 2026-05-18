use crate::config::{clamp_size, DEFAULT_SIZE};
use crate::state::CursorState;
use crate::system::backup::BackupSet;
use crate::theme::CursorTheme;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserView {
    Themes,
    Backups,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingAction {
    ApplyTheme { theme_index: usize, size: u32 },
    RestoreBackup { backup_index: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusMessage {
    pub text: String,
    pub is_error: bool,
}

impl StatusMessage {
    pub fn ready() -> Self {
        Self {
            text: "Ready".to_string(),
            is_error: false,
        }
    }

    pub fn info(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_error: false,
        }
    }

    pub fn error(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_error: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct App {
    pub themes: Vec<CursorTheme>,
    pub backups: Vec<BackupSet>,
    pub view: BrowserView,
    pub selected: usize,
    pub filter: String,
    pub filter_mode: bool,
    pub size: u32,
    pub current: CursorState,
    pub status: StatusMessage,
    pub show_help: bool,
    pub pending_action: Option<PendingAction>,
    pub should_quit: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisibleItem {
    Theme(usize),
}

impl App {
    pub fn new(themes: Vec<CursorTheme>, current: CursorState) -> Self {
        Self {
            themes,
            backups: Vec::new(),
            view: BrowserView::Themes,
            selected: 0,
            filter: String::new(),
            filter_mode: false,
            size: DEFAULT_SIZE,
            current,
            status: StatusMessage::ready(),
            show_help: false,
            pending_action: None,
            should_quit: false,
        }
    }

    pub fn visible_items(&self) -> Vec<VisibleItem> {
        let needle = self.filter.to_lowercase();

        match &self.view {
            BrowserView::Themes => self
                .themes
                .iter()
                .enumerate()
                .filter(|(_, theme)| {
                    needle.is_empty()
                        || theme.name.to_lowercase().contains(&needle)
                        || theme.display_name.to_lowercase().contains(&needle)
                })
                .map(|(index, _)| VisibleItem::Theme(index))
                .collect(),
            BrowserView::Backups => Vec::new(),
        }
    }

    pub fn visible_backups(&self) -> Vec<usize> {
        let needle = self.filter.to_lowercase();
        self.backups
            .iter()
            .enumerate()
            .filter(|(_, backup)| {
                needle.is_empty()
                    || backup.theme.to_lowercase().contains(&needle)
                    || backup.id.to_lowercase().contains(&needle)
                    || backup.files.iter().any(|file| {
                        file.target.to_lowercase().contains(&needle)
                            || file
                                .original_path
                                .display()
                                .to_string()
                                .to_lowercase()
                                .contains(&needle)
                    })
            })
            .map(|(index, _)| index)
            .collect()
    }

    pub fn selected_item(&self) -> Option<VisibleItem> {
        if matches!(self.view, BrowserView::Backups) {
            return None;
        }

        let items = self.visible_items();
        items.get(self.selected).copied()
    }

    pub fn selected_backup(&self) -> Option<&BackupSet> {
        let items = self.visible_backups();
        items
            .get(self.selected)
            .and_then(|index| self.backups.get(*index))
    }

    pub fn selected_backup_index(&self) -> Option<usize> {
        let items = self.visible_backups();
        items.get(self.selected).copied()
    }

    pub fn theme_for_item(&self, item: VisibleItem) -> Option<&CursorTheme> {
        match item {
            VisibleItem::Theme(index) => self.themes.get(index),
        }
    }

    pub fn move_down(&mut self) {
        let count = self.visible_count();
        if count == 0 {
            self.selected = 0;
        } else {
            self.selected = (self.selected + 1).min(count - 1);
        }
    }

    pub fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn back(&mut self) {
        match self.view {
            BrowserView::Themes => {
                self.filter_mode = false;
                self.filter.clear();
            }
            BrowserView::Backups => {
                self.view = BrowserView::Themes;
                self.selected = 0;
                self.filter_mode = false;
                self.filter.clear();
            }
        }
    }

    pub fn show_backups(&mut self, backups: Vec<BackupSet>) {
        self.backups = backups;
        self.view = BrowserView::Backups;
        self.selected = 0;
        self.filter.clear();
        self.filter_mode = false;
    }

    pub fn increase_size(&mut self) {
        self.size = clamp_size(self.size.saturating_add(1));
    }

    pub fn decrease_size(&mut self) {
        self.size = clamp_size(self.size.saturating_sub(1));
    }

    pub fn reset_size(&mut self) {
        self.size = DEFAULT_SIZE;
    }

    pub fn replace_themes(&mut self, themes: Vec<CursorTheme>) {
        self.themes = themes;
        self.view = BrowserView::Themes;
        self.selected = 0;
        self.filter.clear();
        self.filter_mode = false;
    }

    pub fn clamp_selection(&mut self) {
        let count = self.visible_count();
        if count == 0 {
            self.selected = 0;
        } else if self.selected >= count {
            self.selected = count - 1;
        }
    }

    pub fn clear_pending(&mut self) {
        self.pending_action = None;
    }

    fn visible_count(&self) -> usize {
        match self.view {
            BrowserView::Backups => self.visible_backups().len(),
            _ => self.visible_items().len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::theme::{CursorTheme, ThemeSource};

    fn theme(name: &str) -> CursorTheme {
        CursorTheme {
            name: name.to_string(),
            display_name: name.to_string(),
            source_dir: PathBuf::from("/tmp").join(name),
            source_kind: ThemeSource::System,
        }
    }

    fn app() -> App {
        App::new(
            vec![theme("mono"), theme("mono_black"), theme("custom")],
            CursorState {
                theme: "mono".to_string(),
                size: 24,
            },
        )
    }

    #[test]
    fn searches_theme_list() {
        let mut app = app();
        app.filter = "mono".to_string();
        assert_eq!(
            app.visible_items(),
            vec![VisibleItem::Theme(0), VisibleItem::Theme(1)]
        );
    }

    #[test]
    fn filters_theme_list() {
        let mut app = app();
        app.filter = "black".to_string();
        assert_eq!(app.visible_items(), vec![VisibleItem::Theme(1)]);
    }
}
