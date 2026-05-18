use anyhow::Result;

use crate::apply::ApplyReport;
use crate::config::Paths;
use crate::system::backup::BackupSession;
use crate::system::command::CommandRunner;
use crate::system::fs::write_managed_file;
use crate::theme::CursorTheme;

pub fn hyprland_content(theme: &str, size: u32) -> String {
    format!(
        "# Managed by cursors\nenv = XCURSOR_THEME,{theme}\nenv = XCURSOR_SIZE,{size}\nenv = HYPRCURSOR_SIZE,{size}\nexec-once = hyprctl setcursor {theme} {size}\n"
    )
}

pub fn apply(
    theme: &CursorTheme,
    size: u32,
    paths: &Paths,
    commands: &dyn CommandRunner,
    backup: &BackupSession,
) -> Result<ApplyReport> {
    let report = write_managed_file(
        &paths.hypr_cursor_conf,
        &hyprland_content(&theme.name, size),
        backup,
        "hyprland",
    )?;

    if commands.exists("hyprctl") {
        commands.status("hyprctl", &["setcursor", &theme.name, &size.to_string()])?;
    }

    if report.changed {
        Ok(ApplyReport::changed(
            "hyprland",
            "updated Hyprland cursor config",
        ))
    } else {
        Ok(ApplyReport::unchanged("hyprland", "already current"))
    }
}

pub fn apply_runtime(theme: &str, size: u32, commands: &dyn CommandRunner) -> Result<ApplyReport> {
    if !commands.exists("hyprctl") {
        return Ok(ApplyReport::skipped("hyprland", "hyprctl unavailable"));
    }

    commands.status("hyprctl", &["setcursor", theme, &size.to_string()])?;

    Ok(ApplyReport::changed("hyprland", "applied Hyprland cursor"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_expected_hyprland_config() {
        assert_eq!(
            hyprland_content("Yaru", 24),
            "# Managed by cursors\nenv = XCURSOR_THEME,Yaru\nenv = XCURSOR_SIZE,24\nenv = HYPRCURSOR_SIZE,24\nexec-once = hyprctl setcursor Yaru 24\n"
        );
    }
}
