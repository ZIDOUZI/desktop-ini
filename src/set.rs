use crate::Command;
use crate::error::*;
use crate::ini::{Ini, valid_icon_resource};
use crate::sync::check_metadata;
use owo_colors::OwoColorize;
use std::path::PathBuf;

pub fn set(path: &mut PathBuf, command: Command, dry_run: bool) -> Result<()> {
    let Command::Set {
        name,
        icon,
        tip,
        add_tag,
        remove_tag,
        clear_tag,
        run,
        confirm,
    } = command
    else {
        unreachable!("other enum entry shouldn't passed in");
    };

    // need before read: check_metadata will add ini file itself.
    check_metadata(path, dry_run)?;

    println!("{} {}", "Target directory:".cyan(), path.display());

    let mut ini = Ini::read_from(path)?;

    if let Some(name) = name {
        ini.set_localized_resource_name(name);
    }

    if let Some(icon) = icon
        && valid_icon_resource(&icon)
    {
        ini.set_icon_resource(icon);
    }

    if let Some(tip) = tip {
        ini.set_info_tip(tip);
    }

    if let Some(tags) = add_tag {
        ini.add_tags(&tags);
    }

    if let Some(tags) = remove_tag {
        ini.remove_tags(&tags);
    }

    if clear_tag {
        ini.set_tags(&[])
    }

    if let Some(run) = run {
        ini.set_execution(run);
        ini.set_directory_class();
    }

    if confirm {
        ini.set_confirm_execution(true);
    } else if ini.confirm_execution().is_some() {
        ini.set_confirm_execution(false);
    }

    if dry_run {
        println!("{}\n{ini}", "Simulation mode. Will write content below:".yellow());
        Ok(())
    } else {
        ini.write_to(path)?;
        println!("{} {}", "desktop.ini updated at".green(), path.display());
        Ok(())
    }
}
