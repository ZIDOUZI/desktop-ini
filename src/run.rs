use crate::error::{IoReason, Result};
use owo_colors::OwoColorize;
use crate::ini::Ini;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

pub fn run(path: &mut PathBuf) -> Result<()> {
    let ini = Ini::read_from(path)?;

    if !ini.confirm_execution().unwrap_or(false)
        || loop {
            println!("{}", "Confirm Execution.".yellow());
            print!("{}", "Execute custom command? [y]es / [o]pen folder / [n]o: ".cyan());
            io::stdout().flush().reason(|| "flush stdout", None)?;

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .reason(|| "read user input", None)?;

            match input.trim().to_lowercase().as_str() {
                "y" | "yes" => break true,
                "n" | "no" => break false,
                "o" | "open" => {
                    Command::new("explorer")
                        .arg(&path)
                        .spawn()
                        .reason(|| "use Explorer to open", Some(path))?;
                    println!("{}", "Opening folder...".cyan());
                    return Ok(());
                }
                // invalid input, loop again
                _ => continue,
            }
        }
    {
        let execution = match ini.execution() {
            Some(cmd) => cmd,
            None => return Ok(()),
        };

        Command::new("cmd")
            .arg("/C")
            .arg(&execution)
            .current_dir(&path)
            .spawn()
            .reason(|| "execute custom command", Some(path))?;
    }

    Ok(())
}
