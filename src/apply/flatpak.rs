use anyhow::Result;

use crate::apply::ApplyReport;
use crate::config::Paths;
use crate::system::command::CommandRunner;
use crate::theme::CursorTheme;

pub fn apply(
    theme: &CursorTheme,
    size: u32,
    paths: &Paths,
    commands: &dyn CommandRunner,
) -> Result<ApplyReport> {
    if !commands.exists("flatpak") {
        return Ok(ApplyReport::skipped("flatpak", "flatpak unavailable"));
    }

    let icons_mount = format!("--filesystem={}:ro", paths.home.join(".icons").display());
    let env_theme = format!("--env=XCURSOR_THEME={}", theme.name);
    let env_size = format!("--env=XCURSOR_SIZE={size}");

    commands.status("flatpak", &["override", "--user", &icons_mount])?;
    commands.status("flatpak", &["override", "--user", &env_theme, &env_size])?;

    Ok(ApplyReport::changed(
        "flatpak",
        "updated flatpak cursor overrides",
    ))
}
