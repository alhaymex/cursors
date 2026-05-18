use anyhow::Result;

use crate::apply::ApplyReport;
use crate::config::Paths;
use crate::system::backup::BackupSession;
use crate::system::fs::write_managed_file;
use crate::theme::CursorTheme;

pub fn apply(theme: &CursorTheme, paths: &Paths, backup: &BackupSession) -> Result<ApplyReport> {
    let report = write_managed_file(
        &paths.default_icon_theme,
        &format!("[Icon Theme]\nInherits={}\n", theme.name),
        backup,
        "default_icon",
    )?;

    if report.changed {
        let backup = report
            .backup
            .as_ref()
            .map(|backup| format!("; backup {}", backup.target))
            .unwrap_or_default();
        Ok(ApplyReport::changed(
            "default_icon",
            format!("wrote {}{}", report.path.display(), backup),
        ))
    } else {
        Ok(ApplyReport::unchanged("default_icon", "already current"))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use tempfile::tempdir;

    use super::*;
    use crate::config::Paths;
    use crate::theme::ThemeSource;

    #[test]
    fn writes_default_icon_theme() {
        let dir = tempdir().unwrap();
        let paths = Paths::from_home(dir.path().to_path_buf());
        let theme = CursorTheme {
            name: "Yaru".to_string(),
            display_name: "Yaru".to_string(),
            source_dir: PathBuf::from("/usr/share/icons/Yaru"),
            source_kind: ThemeSource::System,
        };

        let backup = BackupSession::new(&paths.backup_dir, &theme.name, 24).unwrap();
        apply(&theme, &paths, &backup).unwrap();

        assert_eq!(
            fs::read_to_string(paths.default_icon_theme).unwrap(),
            "[Icon Theme]\nInherits=Yaru\n"
        );
    }
}
