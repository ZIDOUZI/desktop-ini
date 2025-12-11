use std::fs;
use std::path::PathBuf;
use owo_colors::OwoColorize;
use crate::ErrorAction;
use crate::error::{Error, IoReason, Result, ResultHandle};

pub fn check_metadata(dir: &PathBuf, dry_run: bool) -> Result<bool> {
    if dir.join("desktop.ini").is_file() {
        let meta = fs::metadata(dir).map_err(|source| Error::PermissionDenied {
            path: dir.display().to_string(),
            source,
        })?;

        let mut perms = meta.permissions();
        if !perms.readonly() {
            perms.set_readonly(true);
            if dry_run {
                println!("Writing folder {} into readonly...", dir.display());
            } else {
                fs::set_permissions(dir, perms).map_err(|source| Error::PermissionDenied {
                    path: dir.display().to_string(),
                    source,
                })?;
            }
            return Ok(true);
        }
    }
    Ok(false)
}

fn walk(
    dir: &PathBuf,
    depth: u32,
    changed: &mut u64,
    error_action: ErrorAction,
    dry_run: bool,
) -> Result<()> {
    if let Some(true) = check_metadata(dir, dry_run).decide(error_action)? {
        *changed += 1;
    }

    if depth == 0 {
        return Ok(());
    }

    let Some(entries) = fs::read_dir(dir)
        .reason(|| "read dir", Some(dir))
        .decide(error_action)?
    else {
        return Ok(());
    };

    for entry in entries {
        let Some(entry) = entry
            .reason(|| "read entry in", Some(dir))
            .decide(error_action)?
        else {
            continue;
        };

        let path = entry.path();
        if path.is_dir() {
            walk(&path, depth - 1, changed, error_action, dry_run)
                .decide(error_action)?;
        }
    }

    Ok(())
}

pub fn sync(
    root: &PathBuf,
    max_depth: Option<u32>,
    action: ErrorAction,
    dry_run: bool,
) -> Result<u64> {
    println!("{} {}", "Root path:".cyan(), root.display());
    match max_depth {
        Some(i) => println!("{} {}", "Max depth:".cyan(), i),
        None => println!("{} {}", "Max depth:".cyan(), "Infinity".yellow()),
    }
    let mut changed = 0;
    walk(root, max_depth.unwrap_or(u32::MAX), &mut changed, action, dry_run).decide(action)?;
    Ok(changed)
}
