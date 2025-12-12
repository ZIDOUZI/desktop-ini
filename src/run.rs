use crate::error::{IoReason, Result};
use crate::ini::Ini;
use crate::platform::shell_execute_runas;
use owo_colors::OwoColorize;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

pub fn run(path: &mut PathBuf) -> Result<()> {
    let ini = Ini::read_from(path)?;

    path.pop();

    if !ini.confirm_execution().unwrap_or(false)
        || loop {
            println!("{}", "Confirm Execution.".yellow());
            print!(
                "{}",
                "Execute custom command? [y]es / [o]pen folder / open dekstop.ini [f]ile / [n]o: "
                    .cyan()
            );
            io::stdout().flush().reason(|| "flush stdout", None)?;

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .reason(|| "read user input", None)?;

            match input.trim().to_lowercase().as_str() {
                "y" | "yes" | "" => break true,
                "n" | "no" => break false,
                "o" | "open" => {
                    Command::new("explorer")
                        .arg(&path)
                        .spawn()
                        .reason(|| "use Explorer to open", Some(path))?;
                    println!("{}", "Opening folder...".cyan());
                    return Ok(());
                }
                "f" | "file" => {
                    path.push("desktop.ini");
                    Command::new("explorer")
                        .arg(&path)
                        .spawn()
                        .reason(|| "use Explorer to open", Some(path))?;
                    println!("{}", "Opening desktop.ini...".cyan());
                    return Ok(());
                }
                // invalid input, loop again
                _ => continue,
            }
        }
    {
        let parts = match ini.execution() {
            Some(ref cmd) => parse_command(cmd),
            None => return Ok(()),
        };

        if let [exe, args @ ..] = parts.as_slice() {
            let exe_path = path.join(exe);

            if let Err(e) = Command::new(&exe_path)
                .args(args)
                .current_dir(&path)
                .spawn()
            {
                if let Some(740) = dbg!(e.raw_os_error()) {
                    shell_execute_runas(&exe_path, args, path)
                        .reason(|| "execute custom command with elevation", Some(path))?;
                } else {
                    return Err(e.reason(|| "execute custom command", Some(path)));
                }
            }
        }
    }

    Ok(())
}

fn parse_command(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut escape = false;

    for ch in input.chars() {
        if escape {
            current.push(ch);
            escape = false;
            continue;
        }

        match ch {
            '\\' if in_quotes => escape = true,
            '"' => in_quotes = !in_quotes,
            c if c.is_whitespace() && !in_quotes => {
                if !current.is_empty() {
                    args.push(current);
                    current = String::new();
                }
            }
            c => current.push(c),
        }
    }

    if !current.is_empty() {
        args.push(current);
    }

    args
}
