use std::path::PathBuf;

use anyhow::{anyhow, Result};
use directories::BaseDirs;

pub const DEFAULT_SIZE: u32 = 24;
pub const MIN_SIZE: u32 = 8;
pub const MAX_SIZE: u32 = 128;

#[derive(Debug, Clone)]
pub struct Paths {
    pub home: PathBuf,
    pub icon_dirs: Vec<PathBuf>,
    pub xresources: PathBuf,
    pub hypr_cursor_conf: PathBuf,
    pub default_icon_theme: PathBuf,
    pub backup_dir: PathBuf,
}

impl Paths {
    pub fn new() -> Result<Self> {
        let base_dirs =
            BaseDirs::new().ok_or_else(|| anyhow!(crate::error::CursorError::MissingHome))?;
        Ok(Self::from_home(base_dirs.home_dir().to_path_buf()))
    }

    pub fn from_home(home: PathBuf) -> Self {
        Self {
            icon_dirs: vec![
                home.join(".local/share/icons"),
                home.join(".icons"),
                PathBuf::from("/usr/share/icons"),
            ],
            xresources: home.join(".Xresources"),
            hypr_cursor_conf: home.join(".config/hypr/cursor.conf"),
            default_icon_theme: home.join(".icons/default/index.theme"),
            backup_dir: home.join(".local/state/cursors/backups"),
            home,
        }
    }
}

pub fn clamp_size(size: u32) -> u32 {
    size.clamp(MIN_SIZE, MAX_SIZE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_cursor_size() {
        assert_eq!(clamp_size(1), MIN_SIZE);
        assert_eq!(clamp_size(24), 24);
        assert_eq!(clamp_size(999), MAX_SIZE);
    }
}
