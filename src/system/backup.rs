use std::cmp::Reverse;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};

use crate::system::fs::atomic_write;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupSession {
    pub id: String,
    pub dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupFile {
    pub target: String,
    pub original_path: PathBuf,
    pub content_path: PathBuf,
    pub existed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupSet {
    pub id: String,
    pub theme: String,
    pub size: u32,
    pub dir: PathBuf,
    pub created_at: u128,
    pub files: Vec<BackupFile>,
}

impl BackupSession {
    pub fn new(backup_dir: &Path, theme: &str, size: u32) -> Result<Self> {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("system clock is before unix epoch")?
            .as_nanos();
        let id = format!("{created_at}-{}-{size}", sanitize_target(theme));
        let dir = backup_dir.join(&id);

        Ok(Self { id, dir })
    }

    pub fn backup_file(&self, target: &str, original_path: &Path) -> Result<Option<BackupFile>> {
        let file_dir = self.dir.join("files").join(sanitize_target(target));
        fs::create_dir_all(&file_dir)
            .with_context(|| format!("failed to create {}", file_dir.display()))?;

        let content_path = file_dir.join("content");
        let existed = original_path.exists();
        if existed {
            fs::copy(original_path, &content_path)
                .with_context(|| format!("failed to back up {}", original_path.display()))?;
        }
        fs::write(file_dir.join("target"), target).with_context(|| {
            format!("failed to write backup metadata in {}", file_dir.display())
        })?;
        fs::write(file_dir.join("path"), original_path.display().to_string()).with_context(
            || format!("failed to write backup metadata in {}", file_dir.display()),
        )?;
        fs::write(file_dir.join("existed"), existed.to_string()).with_context(|| {
            format!("failed to write backup metadata in {}", file_dir.display())
        })?;

        Ok(Some(BackupFile {
            target: target.to_string(),
            original_path: original_path.to_path_buf(),
            content_path,
            existed,
        }))
    }

    pub fn finish_metadata(&self, theme: &str, size: u32) -> Result<()> {
        fs::create_dir_all(self.dir.join("files"))
            .with_context(|| format!("failed to create {}", self.dir.display()))?;

        fs::write(self.dir.join("theme"), theme).with_context(|| {
            format!("failed to write backup metadata in {}", self.dir.display())
        })?;
        fs::write(self.dir.join("size"), size.to_string()).with_context(|| {
            format!("failed to write backup metadata in {}", self.dir.display())
        })?;
        Ok(())
    }
}

pub fn list_backups(backup_dir: &Path) -> Result<Vec<BackupSet>> {
    if !backup_dir.exists() {
        return Ok(Vec::new());
    }

    let mut sets = Vec::new();
    for entry in fs::read_dir(backup_dir)
        .with_context(|| format!("failed to read {}", backup_dir.display()))?
    {
        let entry =
            entry.with_context(|| format!("failed to read entry in {}", backup_dir.display()))?;
        if !entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
            continue;
        }

        let set_dir = entry.path();
        let files_dir = set_dir.join("files");
        if !files_dir.is_dir() {
            continue;
        }

        let id = entry.file_name().to_string_lossy().to_string();
        let created_at = id
            .split_once('-')
            .and_then(|(prefix, _)| prefix.parse::<u128>().ok())
            .unwrap_or(0);
        let theme = fs::read_to_string(set_dir.join("theme"))
            .map(|theme| theme.trim().to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        let size = fs::read_to_string(set_dir.join("size"))
            .ok()
            .and_then(|size| size.trim().parse::<u32>().ok())
            .unwrap_or(0);
        let files = list_backup_files(&files_dir)?;

        sets.push(BackupSet {
            id,
            theme,
            size,
            dir: set_dir,
            created_at,
            files,
        });
    }

    sets.sort_by_key(|set| Reverse(set.created_at));
    Ok(sets)
}

pub fn restore_backup(set: &BackupSet) -> Result<usize> {
    let mut restored = 0;
    for file in &set.files {
        if file.existed {
            let content = fs::read_to_string(&file.content_path)
                .with_context(|| format!("failed to read {}", file.content_path.display()))?;
            atomic_write(&file.original_path, &content)?;
        } else if file.original_path.exists() {
            fs::remove_file(&file.original_path)
                .with_context(|| format!("failed to remove {}", file.original_path.display()))?;
        }
        restored += 1;
    }
    Ok(restored)
}

fn list_backup_files(files_dir: &Path) -> Result<Vec<BackupFile>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(files_dir)
        .with_context(|| format!("failed to read {}", files_dir.display()))?
    {
        let entry =
            entry.with_context(|| format!("failed to read entry in {}", files_dir.display()))?;
        if !entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
            continue;
        }

        let file_dir = entry.path();
        let target =
            fs::read_to_string(file_dir.join("target")).unwrap_or_else(|_| "unknown".to_string());
        let original_path = fs::read_to_string(file_dir.join("path"))
            .map(|path| PathBuf::from(path.trim()))
            .unwrap_or_else(|_| PathBuf::from("unknown"));
        let content_path = file_dir.join("content");
        let existed = fs::read_to_string(file_dir.join("existed"))
            .map(|value| value.trim() == "true")
            .unwrap_or_else(|_| content_path.exists());

        files.push(BackupFile {
            target: target.trim().to_string(),
            original_path,
            content_path,
            existed,
        });
    }

    files.sort_by_key(|file| file.target.clone());
    Ok(files)
}

fn sanitize_target(target: &str) -> String {
    target
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn creates_one_restore_point_with_multiple_files() {
        let dir = tempdir().unwrap();
        let backup_dir = dir.path().join("backups");
        let xresources = dir.path().join(".Xresources");
        let hyprland = dir.path().join("cursor.conf");
        fs::write(&xresources, "x original\n").unwrap();
        fs::write(&hyprland, "h original\n").unwrap();

        let session = BackupSession::new(&backup_dir, "Yaru", 24).unwrap();
        session.backup_file("xresources", &xresources).unwrap();
        session.backup_file("hyprland", &hyprland).unwrap();
        session.finish_metadata("Yaru", 24).unwrap();
        fs::write(&xresources, "x changed\n").unwrap();
        fs::write(&hyprland, "h changed\n").unwrap();

        let backups = list_backups(&backup_dir).unwrap();
        assert_eq!(backups.len(), 1);
        assert_eq!(backups[0].theme, "Yaru");
        assert_eq!(backups[0].size, 24);
        assert_eq!(backups[0].files.len(), 2);

        let restored = restore_backup(&backups[0]).unwrap();
        assert_eq!(restored, 2);
        assert_eq!(fs::read_to_string(xresources).unwrap(), "x original\n");
        assert_eq!(fs::read_to_string(hyprland).unwrap(), "h original\n");
    }

    #[test]
    fn creates_restore_point_without_files_for_runtime_state() {
        let dir = tempdir().unwrap();
        let backup_dir = dir.path().join("backups");

        let session = BackupSession::new(&backup_dir, "Adwaita", 24).unwrap();
        session.finish_metadata("Adwaita", 24).unwrap();

        let backups = list_backups(&backup_dir).unwrap();
        assert_eq!(backups.len(), 1);
        assert_eq!(backups[0].theme, "Adwaita");
        assert_eq!(backups[0].size, 24);
        assert!(backups[0].files.is_empty());
    }

    #[test]
    fn restores_missing_file_by_removing_managed_file() {
        let dir = tempdir().unwrap();
        let backup_dir = dir.path().join("backups");
        let managed = dir.path().join("cursor.conf");

        let session = BackupSession::new(&backup_dir, "Adwaita", 24).unwrap();
        session.backup_file("hyprland", &managed).unwrap();
        session.finish_metadata("Adwaita", 24).unwrap();
        fs::write(&managed, "managed\n").unwrap();

        let backups = list_backups(&backup_dir).unwrap();
        assert_eq!(restore_backup(&backups[0]).unwrap(), 1);
        assert!(!managed.exists());
    }
}
