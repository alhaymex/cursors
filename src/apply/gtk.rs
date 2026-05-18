use anyhow::Result;

use crate::apply::ApplyReport;
use crate::system::command::CommandRunner;
use crate::theme::CursorTheme;

pub fn apply(theme: &CursorTheme, size: u32, commands: &dyn CommandRunner) -> Result<ApplyReport> {
    if !commands.exists("gsettings") {
        return Ok(ApplyReport::skipped("gtk", "gsettings unavailable"));
    }

    commands.status(
        "gsettings",
        &[
            "set",
            "org.gnome.desktop.interface",
            "cursor-theme",
            &theme.name,
        ],
    )?;
    commands.status(
        "gsettings",
        &[
            "set",
            "org.gnome.desktop.interface",
            "cursor-size",
            &size.to_string(),
        ],
    )?;

    Ok(ApplyReport::changed(
        "gtk",
        "updated gsettings cursor values",
    ))
}
