use std::ffi::OsStr;
use std::process::{Command, ExitStatus, Stdio};

use anyhow::{Context, Result};

pub trait CommandRunner {
    fn exists(&self, command: &str) -> bool;
    fn output(&self, command: &str, args: &[&str]) -> Result<Option<String>>;
    fn status(&self, command: &str, args: &[&str]) -> Result<Option<ExitStatus>>;
    fn spawn_detached(&self, command: &str, args: &[&str]) -> Result<bool>;
}

#[derive(Debug, Clone, Copy)]
pub struct RealCommandRunner;

impl CommandRunner for RealCommandRunner {
    fn exists(&self, command: &str) -> bool {
        Command::new("sh")
            .args(["-c", &format!("command -v {}", shell_escape(command))])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    fn output(&self, command: &str, args: &[&str]) -> Result<Option<String>> {
        if !self.exists(command) {
            return Ok(None);
        }

        let output = Command::new(command)
            .args(args)
            .output()
            .with_context(|| format!("failed to run {command}"))?;

        if !output.status.success() {
            return Ok(None);
        }

        Ok(Some(
            String::from_utf8_lossy(&output.stdout).trim().to_string(),
        ))
    }

    fn status(&self, command: &str, args: &[&str]) -> Result<Option<ExitStatus>> {
        if !self.exists(command) {
            return Ok(None);
        }

        Command::new(command)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(Some)
            .with_context(|| format!("failed to run {command}"))
    }

    fn spawn_detached(&self, command: &str, args: &[&str]) -> Result<bool> {
        if !self.exists(command) {
            return Ok(false);
        }

        Command::new(command)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .with_context(|| format!("failed to run {command}"))?;

        Ok(true)
    }
}

fn shell_escape(command: &str) -> String {
    if command
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
    {
        command.to_string()
    } else {
        let escaped = command.replace('\'', "'\"'\"'");
        format!("'{escaped}'")
    }
}

#[allow(dead_code)]
fn _os_arg<T: AsRef<OsStr>>(arg: T) -> T {
    arg
}
