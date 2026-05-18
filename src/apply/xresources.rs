use anyhow::Result;

use crate::apply::ApplyReport;
use crate::config::Paths;
use crate::system::backup::BackupSession;
use crate::system::command::CommandRunner;
use crate::system::fs::update_lines_file;
use crate::theme::CursorTheme;

pub fn apply(
    theme: &CursorTheme,
    size: u32,
    paths: &Paths,
    commands: &dyn CommandRunner,
    backup: &BackupSession,
) -> Result<ApplyReport> {
    let report = update_lines_file(
        &paths.xresources,
        &["Xcursor.theme:", "Xcursor.size:"],
        &[
            format!("Xcursor.theme: {}", theme.name),
            format!("Xcursor.size: {size}"),
        ],
        backup,
        "xresources",
    )?;

    if commands.exists("xrdb") {
        commands.status(
            "xrdb",
            &["-merge", paths.xresources.to_string_lossy().as_ref()],
        )?;
    }

    if report.changed {
        Ok(ApplyReport::changed(
            "xresources",
            "updated Xresources cursor values",
        ))
    } else {
        Ok(ApplyReport::unchanged("xresources", "already current"))
    }
}

pub fn apply_runtime(paths: &Paths, commands: &dyn CommandRunner) -> Result<ApplyReport> {
    if !commands.exists("xrdb") {
        return Ok(ApplyReport::skipped("xresources", "xrdb unavailable"));
    }

    if !paths.xresources.exists() {
        return Ok(ApplyReport::skipped("xresources", "Xresources unavailable"));
    }

    commands.status(
        "xrdb",
        &["-merge", paths.xresources.to_string_lossy().as_ref()],
    )?;

    Ok(ApplyReport::changed("xresources", "merged Xresources"))
}
