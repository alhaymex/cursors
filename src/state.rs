use crate::config::DEFAULT_SIZE;
use crate::system::command::CommandRunner;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorState {
    pub theme: String,
    pub size: u32,
}

pub fn current_state(commands: &dyn CommandRunner) -> CursorState {
    let theme = commands
        .output(
            "gsettings",
            &["get", "org.gnome.desktop.interface", "cursor-theme"],
        )
        .ok()
        .flatten()
        .map(|theme| theme.trim_matches('\'').to_string())
        .filter(|theme| !theme.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    let size = commands
        .output(
            "gsettings",
            &["get", "org.gnome.desktop.interface", "cursor-size"],
        )
        .ok()
        .flatten()
        .and_then(|size| size.parse::<u32>().ok())
        .unwrap_or(DEFAULT_SIZE);

    CursorState { theme, size }
}
