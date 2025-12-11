use crate::error::{IoReason, Result};
use std::path::PathBuf;

#[cfg(windows)]
mod windows {
    use crate::error::Result;
    use crate::setup::current_exe;
    use crate::DIRECTORY_CLASS;
    use owo_colors::OwoColorize;
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    pub fn setup() -> Result<()> {
        let exe_path = current_exe()?;
        let exe_str = exe_path.to_string_lossy();

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);

        let (class_key, _) = hkcu.create_subkey(format!(r"Software\Classes\{DIRECTORY_CLASS}"))?;

        class_key.set_value("", &"Desktop.ini Custom Open Handler")?;

        let (command_key, _) = class_key.create_subkey(r"Shell\open\command")?;

        command_key.set_value("", &format!(r#""{exe_str}" run --path "%1""#))?;

        println!("{}", "Registry setup completed.".green());
        Ok(())
    }
}

pub fn current_exe() -> Result<PathBuf> {
    std::env::current_exe().reason(|| "get current exe", None)
}

#[cfg(windows)]
pub use windows::setup;

#[cfg(not(windows))]
pub fn setup() -> Result<()> {
    Err(crate::error::Error::UnsupportedOS)
}