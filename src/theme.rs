use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::config::Paths;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThemeSource {
    LocalShare,
    Icons,
    System,
}

impl ThemeSource {
    pub fn label(&self) -> &'static str {
        match self {
            Self::LocalShare => "~/.local/share/icons",
            Self::Icons => "~/.icons",
            Self::System => "/usr/share/icons",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorTheme {
    pub name: String,
    pub display_name: String,
    pub source_dir: PathBuf,
    pub source_kind: ThemeSource,
}

pub fn discover_themes(paths: &Paths) -> Result<Vec<CursorTheme>> {
    let mut themes = BTreeMap::<String, CursorTheme>::new();

    for (index, root) in paths.icon_dirs.iter().enumerate() {
        let source_kind = match index {
            0 => ThemeSource::LocalShare,
            1 => ThemeSource::Icons,
            _ => ThemeSource::System,
        };

        if !root.exists() {
            continue;
        }

        let entries =
            fs::read_dir(root).with_context(|| format!("failed to read {}", root.display()))?;
        for entry in entries {
            let entry =
                entry.with_context(|| format!("failed to read entry in {}", root.display()))?;
            let file_type = entry
                .file_type()
                .with_context(|| format!("failed to stat {}", entry.path().display()))?;
            if !file_type.is_dir() {
                continue;
            }

            let theme_dir = entry.path();
            if !is_cursor_theme(&theme_dir)? {
                continue;
            }

            let name = entry.file_name().to_string_lossy().to_string();
            themes.entry(name.clone()).or_insert_with(|| CursorTheme {
                display_name: read_display_name(&theme_dir).unwrap_or_else(|| name.clone()),
                name,
                source_dir: theme_dir,
                source_kind: source_kind.clone(),
            });
        }
    }

    Ok(themes.into_values().collect())
}

pub fn family_name(theme: &str) -> String {
    if theme.starts_with("Yaru") {
        return "Yaru".to_string();
    }

    if theme.starts_with("breeze") || theme.starts_with("Breeze") {
        return "breeze".to_string();
    }

    theme
        .split(['_', '-'])
        .next()
        .filter(|part| !part.is_empty())
        .unwrap_or(theme)
        .to_string()
}

fn is_cursor_theme(theme_dir: &Path) -> Result<bool> {
    if theme_dir.join("cursors").is_dir() {
        return Ok(true);
    }

    let index_theme = theme_dir.join("index.theme");
    if !index_theme.exists() {
        return Ok(false);
    }

    let content = fs::read_to_string(&index_theme)
        .with_context(|| format!("failed to read {}", index_theme.display()))?;
    Ok(content
        .lines()
        .any(|line| line.trim_start().starts_with("Inherits=")))
}

fn read_display_name(theme_dir: &Path) -> Option<String> {
    let content = fs::read_to_string(theme_dir.join("index.theme")).ok()?;
    content
        .lines()
        .find_map(|line| line.strip_prefix("Name=").map(ToOwned::to_owned))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;
    use crate::config::Paths;

    #[test]
    fn discovers_themes_from_cursors_dir_and_inherits() {
        let dir = tempdir().unwrap();
        let home = dir.path().join("home");
        let local = home.join(".local/share/icons");
        fs::create_dir_all(local.join("with-cursors/cursors")).unwrap();
        fs::create_dir_all(local.join("with-inherits")).unwrap();
        fs::write(
            local.join("with-inherits/index.theme"),
            "[Icon Theme]\nInherits=Adwaita\n",
        )
        .unwrap();

        let paths = Paths::from_home(home);
        let themes = discover_themes(&paths).unwrap();
        let names = themes
            .into_iter()
            .map(|theme| theme.name)
            .collect::<Vec<_>>();

        assert!(names.contains(&"with-cursors".to_string()));
        assert!(names.contains(&"with-inherits".to_string()));
    }

    #[test]
    fn deduplicates_by_source_priority() {
        let dir = tempdir().unwrap();
        let home = dir.path().join("home");
        fs::create_dir_all(home.join(".local/share/icons/shared/cursors")).unwrap();
        fs::create_dir_all(home.join(".icons/shared/cursors")).unwrap();

        let paths = Paths::from_home(home);
        let themes = discover_themes(&paths).unwrap();
        let theme = themes.iter().find(|theme| theme.name == "shared").unwrap();

        assert_eq!(theme.source_kind, ThemeSource::LocalShare);
    }

    #[test]
    fn groups_theme_families() {
        assert_eq!(family_name("Yaru-blue-dark"), "Yaru");
        assert_eq!(family_name("mono_black"), "mono");
        assert_eq!(family_name("black-point-outside"), "black");
    }
}
