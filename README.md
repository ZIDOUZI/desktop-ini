# desktop-ini

[![Crates.io](https://img.shields.io/crates/v/desktop-ini.svg)](https://crates.io/crates/desktop-ini)
[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

[English](./README.md) | [简体中文](./README_zh.md)

A small CLI tool for working with directory `desktop.ini` files on Windows.

It helps you inspect and edit `desktop.ini` fields, manage tags, configure custom execution commands, and sync folder attributes.

## Features

- **Inspect** a directory's `desktop.ini` and show a concise, human-friendly summary.
- **Set** common fields such as title, icon, InfoTip, tags (Prop5) and custom execution commands.
- **Run** the custom command defined in `desktop.ini`, with optional second confirmation.
- **Sync** all subdirectories that contain `desktop.ini`, making the folders read-only.
- **Registry integration**: register a custom directory class so Explorer can delegate opening to this program.

There is also a Chinese README: [`README_zh.md`](docs/README_zh.md).

## Install

```bash
# From the project root
cargo install --path .
```

After installation, the compiled binary (usually named `desktop-ini`, depending on your environment) will be placed in Cargo's bin directory.

## Global options

All subcommands share these global options:

- `--path <DIR>`: Target directory. Defaults to the current working directory.
- `--error-action <MODE>`: Error handling strategy, one of:
  - `Continue`: Print error and continue (default).
  - `Inquire`: Pause on error and wait for Enter.
  - `SilentlyContinue`: Ignore errors without printing messages.
  - `Stop`: Abort immediately on error.
- `--dry-run`: Simulation mode. Print what would be written, but do not actually modify files or attributes.

## Subcommands

### View directory info

```bash
desktop-ini show --path <DIR>
```

Prints a compact summary of the `desktop.ini` contents, including:

- Title (`LocalizedResourceName`)
- InfoTip
- Icon (`IconResource`)
- Tags (Prop5 parsed into a tag list)
- Custom execution command and its confirmation status

### Edit desktop.ini

```bash
desktop-ini set \
  --path <DIR> \
  --name "Example folder" \
  --icon "shell32.dll,4" \
  --tip "This is an example" \
  --add-tag work --add-tag rust \
  --run "code ." \
  --confirm
```

Common options:

- `--name`: Set directory display name (`LocalizedResourceName`).
- `--icon`: Set icon (`IconResource`), for example `"shell32.dll,4"`.
- `--tip`: Set hover text (`InfoTip`).
- `--add-tag` / `--remove-tag` / `--clear-tag`: Manage tags stored in Prop5.
- `--run`: Set a custom execution command (executed via `cmd /C` in the target directory).
- `--confirm`: Enable second confirmation before running the command.
  - If not provided and confirmation was previously enabled, the flag will be turned off.

Use `--dry-run` to preview changes and the final `desktop.ini` content without actually writing.

### Execute the configured command

```bash
desktop-ini run --path <DIR>
```

- If confirmation is enabled, you will be prompted:
  - `y/yes`: Execute the command.
  - `n/no`: Do nothing and exit.
  - `o/open`: Open the folder in Explorer and exit.
- If no `Target` is configured in `desktop.ini`, the command simply returns without doing anything.

### Batch make folders read-only

```bash
desktop-ini sync --path <ROOT> --depth <N>
```

- Recursively walk from `ROOT`. For each directory that contains `desktop.ini`, set the folder itself to read-only.
- `--depth` controls the maximum recursion depth. If omitted, the depth is effectively infinite.
- Combine with `--dry-run` to see how many folders would be affected without applying changes.

### Registry setup

```bash
desktop-ini setup
```

Creates a custom directory class under the current user's registry:

- `Software\Classes\INI.CustomExecution` (name controlled by the `DIRECTORY_CLASS` constant).
- Sets its `Shell\open\command` to the current executable's `run` subcommand.

Once this is configured, you can use `DirectoryClass=INI.CustomExecution` plus a `[.CustomExecution]` section in `desktop.ini` to have Explorer invoke this program when opening that folder.

### Generate shell completions

```bash
desktop-ini completion | Out-String | Invoke-Expression
```

For example, this generates a PowerShell completion script and loads it into the current session. Write this into `$PROFILE` to make it permanent.

## Development

```bash
# Run tests
cargo test

# Build the project
cargo build
```

Main dependencies:

- `clap` / `clap_complete`: CLI parsing and completions
- `owo-colors`: colored terminal output
- `encoding_rs`: read/write text using the system ANSI code page
- `thiserror`: error type definitions
- `winreg`: Windows registry access
