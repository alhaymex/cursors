use thiserror::Error;

#[derive(Debug, Error)]
pub enum CursorError {
    #[error("home directory is unavailable")]
    MissingHome,
    #[error("no cursor themes found")]
    NoThemes,
}
