use crate::error::*;
use crate::ini::Ini;
use crate::sync::check_metadata;
use crate::Command;
use owo_colors::OwoColorize;
use std::path::{Path, PathBuf};

pub fn set(path: &mut PathBuf, command: Command, dry_run: bool) -> Result<()> {
    let Command::Set {
        name,
        icon,
        info_tip,
        tag,
        remove_tag,
        clear_tag,
        command,
        args,
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

    if let Some(tip) = info_tip {
        ini.set_info_tip(tip);
    }

    if !tag.is_empty() {
        ini.add_tags(
            &tag
                .iter()
                .flat_map(|s| s.split(','))
                .map(str::to_string)
                .collect::<Vec<_>>(),
        );
    }

    if !remove_tag.is_empty() {
        ini.remove_tags(
            &remove_tag
                .iter()
                .flat_map(|s| s.split(','))
                .map(str::to_string)
                .collect::<Vec<_>>(),
        );
    }

    if clear_tag {
        ini.clear_tags();
    }

    if let Some(command) = command {
        ini.set_execution(command);
        ini.set_directory_class();
    }

    ini.set_args(&args);

    if confirm {
        ini.set_confirm_execution(true);
    } else if ini.confirm_execution().is_some() {
        ini.set_confirm_execution(false);
    }

    if dry_run {
        println!(
            "{}\n{:?}",
            "Simulation mode. Will write content below:".yellow(), ini
        );
        Ok(())
    } else {
        ini.write_to(path)?;
        println!("{} {}", "desktop.ini updated at".green(), path.display());
        Ok(())
    }
}


fn valid_icon_resource(s: &str) -> bool {
    match s.rsplit_once(",") {
        Some((exe, pos)) if pos.parse::<u32>().is_ok() => Path::new(exe).is_file(),
        None => Path::new(s).is_file(),
        _ => false,
    }
}
