use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn help_explains_tui_usage() {
    let mut cmd = Command::cargo_bin("cursors").unwrap();
    cmd.arg("--help").assert().success().stdout(
        predicate::str::contains("full-screen TUI").or(predicate::str::contains("Full-screen TUI")),
    );
}

#[test]
fn version_prints() {
    let mut cmd = Command::cargo_bin("cursors").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}
