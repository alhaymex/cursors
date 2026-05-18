# cursors

A small Rust/Ratatui TUI for managing Linux cursor themes.

Run `cursors`, search installed cursor themes, choose a size, apply a theme, and restore previous cursor states from built-in backups.

## Install

```bash
./install.sh
```

This builds the release binary and installs it to:

```text
~/.local/bin/cursors
```

If `~/.local/bin` is not on your `PATH`, add this to your shell config:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

You can also use Cargo directly:

```bash
cargo install --path .
```

## Usage

```bash
cursors
```

The app is TUI-only. The only non-interactive commands are:

```bash
cursors --help
cursors --version
```

## Keys

Main theme list:

```text
j / Down     move down
k / Up       move up
/            search themes
Enter        confirm/apply selected theme
Space        open selected theme directory
+ / -        change cursor size
d            reset size to 24
r            rescan themes
b            open backups page
?            help
q            quit
```

Backups page:

```text
j / Down     move down
k / Up       move up
/            search backups
Enter / u    confirm/restore selected backup
h / Esc      return to themes
?            help
q            quit
```

## What It Changes

When applying a cursor, `cursors` may update:

```text
~/.icons/default/index.theme
~/.Xresources
~/.config/hypr/cursor.conf
```

It also runs user-level commands when available:

```text
gsettings
xrdb
hyprctl
flatpak override --user
```

Missing tools are skipped.

## Backups

Before applying a cursor, the app creates a restore point under:

```text
~/.local/state/cursors/backups/
```

Restore points are visible from the `b` backups page. Restoring a backup restores managed files and reapplies the saved cursor theme/size to the running environment.

## Development

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo build --release
```
