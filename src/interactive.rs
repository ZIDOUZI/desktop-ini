use crate::error::{IoReason, Result};
use crate::ini::Ini;
use crate::sync::check_metadata;
use owo_colors::OwoColorize;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn interactive(path: &mut PathBuf, dry_run: bool) -> Result<()> {
    println!("{} {}", "Target directory:".cyan(), path.display());

    check_metadata(path, dry_run)?;

    let mut ini_path = path.clone();
    let mut ini = Ini::read_from(&mut ini_path)?;

    let name = prompt_value(
        "目录显示名称 (LocalizedResourceName)",
        ini.localized_resource_name(),
    )?;
    if let Some(v) = name {
        ini.set_localized_resource_name(v);
    }

    let icon = prompt_value("图标 (IconResource)", ini.icon_resource())?;
    if let Some(icon) = icon {
        if icon.trim().is_empty() {
            ini.set_icon_resource(icon);
        } else if valid_icon_resource(&icon) {
            ini.set_icon_resource(icon);
        } else {
            println!(
                "{}",
                "图标路径无效，已跳过本项。你可以稍后用 set 子命令再设置。".yellow()
            );
        }
    }

    let info_tip = prompt_value("悬停提示 (InfoTip)", ini.info_tip())?;
    if let Some(v) = info_tip {
        ini.set_info_tip(v);
    }

    let title = prompt_value("Title (Prop2)", ini.title())?;
    if let Some(v) = title {
        ini.set_title(v);
    }

    let subject = prompt_value("Subject (Prop3)", ini.subject())?;
    if let Some(v) = subject {
        ini.set_subject(v);
    }

    let author = prompt_value("Author (Prop4)", ini.author())?;
    if let Some(v) = author {
        ini.set_author(v);
    }

    let comments = prompt_value("Comments (Prop6)", ini.comments())?;
    if let Some(v) = comments {
        ini.set_comments(v);
    }

    if let Some(mut tags) = ini.tags() {
        tags.sort();
        println!(
            "{} {}",
            "当前标签:".cyan(),
            if tags.is_empty() {
                "(无)".yellow().to_string()
            } else {
                tags.join(", ")
            }
        );
    }

    let clear_tags = prompt_yes_no_keep("是否清空所有标签", Some(false))?;
    if let Some(true) = clear_tags {
        ini.clear_tags();
    } else {
        let add_tags = prompt_list("添加标签 (逗号分隔)")?;
        if !add_tags.is_empty() {
            ini.add_tags(&add_tags);
        }

        let remove_tags = prompt_list("删除标签 (逗号分隔)")?;
        if !remove_tags.is_empty() {
            ini.remove_tags(&remove_tags);
        }
    }

    let exec = prompt_value("自定义执行命令 (Target)", ini.execution())?;
    if let Some(v) = exec {
        ini.set_execution(v);
        if ini.execution().is_some() {
            ini.set_directory_class();
        } else {
            ini.remove("[.ShellClassInfo]", "DirectoryClass");
        }
    }

    let args = prompt_value("自定义执行参数 (Args，一行输入)", ini.get("[.CustomExecution]", "Args"))?;
    if let Some(v) = args {
        if v.trim().is_empty() {
            ini.remove("[.CustomExecution]", "Args");
        } else {
            ini.set("[.CustomExecution]", "Args", v);
        }
    }

    let confirm = prompt_yes_no_keep("执行前二次确认 (ConfirmExecution)", ini.confirm_execution())?;
    match confirm {
        Some(true) => {
            ini.set_confirm_execution(true);
        }
        Some(false) => {
            if ini.confirm_execution().is_some() {
                ini.set_confirm_execution(false);
            }
        }
        None => {}
    }

    let apply = prompt_yes_no_keep("确认写入 desktop.ini", Some(true))?.unwrap_or(true);
    if !apply {
        println!("{}", "已取消，未写入。".yellow());
        return Ok(());
    }

    if dry_run {
        println!(
            "{}\n{:?}",
            "Simulation mode. Will write content below:".yellow(),
            ini
        );
        Ok(())
    } else {
        ini.write_to(&mut ini_path)?;
        check_metadata(path, false)?;
        println!("{} {}", "desktop.ini updated at".green(), ini_path.display());
        Ok(())
    }
}

fn prompt_value(label: &str, current: Option<String>) -> Result<Option<String>> {
    let current_show = current
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("(No Value)");
    print!(
        "{} {}{}",
        label.cyan(),
        format!("[Current: {current_show}] ").dimmed(),
        "Enter new value; Press Enter to keep; Enter - to clear: ".cyan()
    );
    io::stdout().flush().reason(|| "flush stdout", None)?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .reason(|| "read user input", None)?;

    let s = input.trim_end().trim();
    if s.is_empty() {
        Ok(None)
    } else if s == "-" {
        Ok(Some(String::new()))
    } else {
        Ok(Some(s.to_string()))
    }
}

fn prompt_list(label: &str) -> Result<Vec<String>> {
    print!(
        "{}{}",
        label.cyan(),
        "；回车跳过: ".cyan()
    );
    io::stdout().flush().reason(|| "flush stdout", None)?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .reason(|| "read user input", None)?;

    let s = input.trim_end().trim();
    if s.is_empty() {
        return Ok(Vec::new());
    }

    Ok(s
        .split(',')
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .collect())
}

fn prompt_yes_no_keep(label: &str, current: Option<bool>) -> Result<Option<bool>> {
    let current_show = match current {
        Some(true) => "on",
        Some(false) => "off",
        None => "(未设置)",
    };
    print!(
        "{} {}{}",
        label.cyan(),
        format!("[当前: {current_show}] ").dimmed(),
        "y/n；回车保持: ".cyan()
    );
    io::stdout().flush().reason(|| "flush stdout", None)?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .reason(|| "read user input", None)?;

    match input.trim().to_lowercase().as_str() {
        "" => Ok(None),
        "y" | "yes" => Ok(Some(true)),
        "n" | "no" => Ok(Some(false)),
        _ => Ok(None),
    }
}

fn valid_icon_resource(s: &str) -> bool {
    match s.rsplit_once(',') {
        Some((exe, pos)) if pos.parse::<u32>().is_ok() => Path::new(exe).is_file(),
        None => Path::new(s).is_file(),
        _ => false,
    }
}