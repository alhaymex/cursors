use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};

use crate::system::backup::{BackupFile, BackupSession};

#[derive(Debug, Clone)]
pub struct WriteReport {
    pub path: PathBuf,
    pub backup: Option<BackupFile>,
    pub changed: bool,
}

pub fn write_managed_file(
    path: &Path,
    content: &str,
    backup: &BackupSession,
    target: &str,
) -> Result<WriteReport> {
    let existing = fs::read_to_string(path).ok();
    if existing.as_deref() == Some(content) {
        return Ok(WriteReport {
            path: path.to_path_buf(),
            backup: None,
            changed: false,
        });
    }

    let backup_file = backup.backup_file(target, path)?;
    atomic_write(path, content)?;

    Ok(WriteReport {
        path: path.to_path_buf(),
        backup: backup_file,
        changed: true,
    })
}

pub fn update_lines_file(
    path: &Path,
    remove_prefixes: &[&str],
    append_lines: &[String],
    backup: &BackupSession,
    target: &str,
) -> Result<WriteReport> {
    let existing = fs::read_to_string(path).unwrap_or_default();
    let mut kept = existing
        .lines()
        .filter(|line| {
            !remove_prefixes
                .iter()
                .any(|prefix| line.starts_with(prefix))
        })
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    kept.extend(append_lines.iter().cloned());

    let content = format!(
        "{}\n",
        kept.into_iter()
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    );
    write_managed_file(path, &content, backup, target)
}

pub(crate) fn atomic_write(path: &Path, content: &str) -> Result<()> {
    let parent = path
        .parent()
        .context("managed path has no parent directory")?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before unix epoch")?
        .as_nanos();
    let tmp_path = parent.join(format!(
        ".{}.tmp.{timestamp}",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("cursors")
    ));

    {
        let mut file = fs::File::create(&tmp_path)
            .with_context(|| format!("failed to create {}", tmp_path.display()))?;
        file.write_all(content.as_bytes())
            .with_context(|| format!("failed to write {}", tmp_path.display()))?;
        file.sync_all()
            .with_context(|| format!("failed to sync {}", tmp_path.display()))?;
    }

    fs::rename(&tmp_path, path).with_context(|| format!("failed to replace {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn updates_xresources_lines_without_removing_unrelated_lines() {
        let dir = tempdir().unwrap();
        let path = dir.path().join(".Xresources");
        let backup = BackupSession::new(&dir.path().join("backups"), "Yaru", 24).unwrap();
        fs::write(
            &path,
            "URxvt.font: x\nXcursor.theme: old\nXcursor.size: 12\nXft.dpi: 96\n",
        )
        .unwrap();

        update_lines_file(
            &path,
            &["Xcursor.theme:", "Xcursor.size:"],
            &[
                "Xcursor.theme: Yaru".to_string(),
                "Xcursor.size: 24".to_string(),
            ],
            &backup,
            "xresources",
        )
        .unwrap();

        let updated = fs::read_to_string(path).unwrap();
        assert!(updated.contains("URxvt.font: x"));
        assert!(updated.contains("Xft.dpi: 96"));
        assert!(updated.contains("Xcursor.theme: Yaru"));
        assert!(updated.contains("Xcursor.size: 24"));
        assert!(!updated.contains("Xcursor.theme: old"));
    }

    #[test]
    fn writes_into_the_given_backup_session() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("cursor.conf");
        let backup = BackupSession::new(&dir.path().join("backups"), "Yaru", 24).unwrap();

        fs::write(&path, "original\n").unwrap();
        let first = write_managed_file(&path, "changed\n", &backup, "hyprland").unwrap();

        assert!(first.backup.is_some());
        assert!(backup.dir.join("files/hyprland/content").exists());
    }
}
