use std::path::PathBuf;
use std::{env, io};

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, shells, Shell};
use clap_complete_nushell::Nushell;
use owo_colors::OwoColorize;

mod encoding;
pub mod error;
mod ini;
mod run;
mod set;
mod setup;
mod sync;
mod platform;
mod interactive;

use crate::error::{Error, IoReason};
use crate::ini::Ini;
use crate::set::set;
use crate::setup::current_exe;
use run::run;
use setup::setup;
use sync::sync;

pub const DIRECTORY_CLASS: &str = "INI.CustomExecution";

#[derive(Debug, Copy, Clone, ValueEnum, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ErrorAction {
    Continue,
    Inquire,
    #[value(name = "silently")]
    SilentlyContinue,
    Stop,
}

/// Simple CLI for working with directory desktop.ini (minimal, subcommand-based)
#[derive(Parser, Debug)]
#[command(name = "desktop-ini", version, about, long_about = None)]
struct Cli {
    /// Path to the directory (default: current directory)
    #[arg(short, long, global = true)]
    path: Option<PathBuf>,

    /// Ignore errors and continue running
    #[arg(short, long, global = true, value_enum, default_value_t = ErrorAction::Continue)]
    error_action: ErrorAction,

    /// dry run all write operation
    #[arg(long, global = true)]
    dry_run: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
#[allow(clippy::large_enum_variant)]
enum Command {
    /// Setup registry.
    Setup,

    /// Show info of a directory (placeholder)
    Show,

    /// Execute the operation defined in desktop.ini
    Run,

    /// Sync folder in current directory which contains desktop.ini.
    Sync {
        /// Recursively sync depth (default: Infinity)
        #[arg(short, long)]
        depth: Option<u32>,
    },

    /// Set desktop.ini fields for a directory (placeholder)
    Set {
        /// LocalizedResourceName, aka title of this directory
        #[arg(short, long, alias = "localized-resource-name")]
        name: Option<String>,

        /// IconResource, for example: "shell32.dll,4"
        #[arg(short, long)]
        icon: Option<String>,

        /// InfoTip. show when hover on this directory.
        #[arg(long, value_name = "INFO_TIP", alias = "info", alias = "tip")]
        info_tip: Option<String>,

        /// [{F29F85E0-4FF9-1068-AB91-08002B27B3D9}] Prop5, aka tags/labels.
        #[arg(short, long, alias = "add-tag")]
        tag: Vec<String>,

        /// remove a tag.
        #[arg(short, long)]
        remove_tag: Vec<String>,

        /// clear all tags.
        #[arg(long, conflicts_with_all = ["tag", "remove_tag"])]
        clear_tag: bool,

        /// title
        #[arg(long)]
        title: Option<String>,

        /// subject. you can use it as subtitle.
        #[arg(long)]
        subject: Option<String>,

        /// author
        #[arg(long)]
        author: Option<String>,

        /// comments
        #[arg(long)]
        comments: Option<String>,

        /// execution path
        #[arg(short, long)]
        command: Option<String>,

        /// execution args. use %1 to present the folder where desktop.ini in.
        #[arg(short, long)]
        args: Option<Vec<String>>,

        /// confirm execution
        #[arg(long)]
        confirm: bool,
    },

    /// Interactive set mode
    Interactive,

    /// Generate completions for shell
    Completion {
        #[command(subcommand)]
        shell: CompletionShell
    },
}

#[derive(Subcommand, Debug)]
enum CompletionShell {
    PowerShell,
    Bash,
    NuShell,
}

fn main() {
    let cli = Cli::parse();

    let mut path = match cli.path {
        Some(p) => p,
        None => match env::current_dir() {
            Ok(p) => p,
            Err(_) => {
                eprintln!("{}", "Cannot get cwd. You need to provide --path manually".red());
                return;
            }
        },
    };

    if cli.dry_run {
        println!("{}", "Dry-run is on. all changes will not applied.".yellow());
    }

    let result = match cli.command {
        Some(Command::Setup) => setup(),
        Some(Command::Show) => Ini::read_from(&mut path).map(|ini| println!("{:?}", ini)),
        Some(Command::Run) => run(&mut path),
        Some(Command::Sync { depth }) => sync(&path, depth, cli.error_action, cli.dry_run)
            .map(|count| println!("{}", format!("Updated {count} folder(s) to read-only.").green())),
        Some(s @ Command::Set { .. }) => set(&mut path, s, cli.dry_run),
        None => Cli::command()
            .print_help()
            .reason(|| "print help message", None),
        Some(Command::Interactive) => interactive::interactive(&mut path, cli.dry_run),
        Some(Command::Completion { shell }) => {
            let mut cmd = Cli::command();
            let cmd_name = if let Ok(p) = current_exe()
                && let Some(n) = p.file_stem()
            {
                n.to_string_lossy().to_string()
            } else {
                cmd.get_name().to_string()
            };
            match shell {
                CompletionShell::PowerShell => generate(Shell::PowerShell, &mut cmd, cmd_name, &mut io::stdout()),
                CompletionShell::Bash => generate(Shell::Bash, &mut cmd, cmd_name, &mut io::stdout()),
                CompletionShell::NuShell => generate(Nushell, &mut cmd, cmd_name, &mut io::stdout()),
            }
            Ok(())
        }
    };

    if let Err(error) = result {
        eprintln!("{}", error.to_string().red());
        match error {
            Error::Io { .. } => std::process::exit(74),
            Error::PermissionDenied { .. } => std::process::exit(126),
            _ => std::process::exit(1),
        }
    }
}
