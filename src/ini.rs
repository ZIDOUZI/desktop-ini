use crate::encoding::{read_to_string_system, write_string_system};
use crate::error::{IoReason, Result};
use crate::DIRECTORY_CLASS;
use owo_colors::OwoColorize;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::path::PathBuf;

fn parse_args(input: &str, path: &str) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    let mut iter = input.chars().peekable();
    while let Some(ch) = iter.next() {
        match ch {
            '"' => {
                if in_quotes && iter.peek().is_none_or(|c| c.is_whitespace()) {
                    in_quotes = false;
                } else {
                    current.push(ch);
                }
            }
            '\\' if in_quotes => match iter.peek() {
                Some('"') => current.push(iter.next().unwrap_or('"')),
                Some('\\') => current.push(iter.next().unwrap_or('\\')),
                _ => current.push('\\'),
            },
            '%' => match iter.peek() {
                Some('1') => {
                    iter.next();
                    current.push_str(path);
                }
                Some('%') => current.push(iter.next().unwrap_or('%')),
                _ => current.push('%'),
            },
            c if c.is_whitespace() && !in_quotes => {
                if !current.is_empty() {
                    args.push(std::mem::take(&mut current));
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

fn join_args(input: &[String]) -> String {
    let mut output = String::new();
    for (idx, arg) in input.iter().enumerate() {
        if idx > 0 {
            output.push(' ');
        }

        if arg.chars().any(char::is_whitespace) {
            output.push('"');
            for ch in arg.chars() {
                if ch == '"' {
                    output.push('\\');
                    output.push('"');
                } else {
                    output.push(ch);
                }
            }
            output.push('"');
        } else {
            output.push_str(arg);
        }
    }
    output
}

pub struct Ini {
    dictionary: HashMap<String, HashMap<String, String>>,
}

macro_rules! accessor {
    ($set_fn:ident, $section:expr, $key:expr, $in_type:ty) => {
        pub fn $set_fn(&mut self, value: $in_type) -> Option<String> {
            if value.trim().is_empty() {
                self.remove($section, $key)
            } else {
                self.set($section, $key, value)
            }
        }
    };

    ($get_fn:ident, $set_fn:ident, $section:expr, $key:expr, $in_type:ty) => {
        pub fn $get_fn(&self) -> Result<$in_type> {
            self.dictionary
                .get($section)
                .and_then(|map| map.get($key))
                .ok_or(Error::NoValue)
                .and_then(|s| $in_type::from_str(s))
        }

        accessor!($set_fn, $section, $key, $in_type);
    };

    ($set_fn:ident, $section:expr, $key:expr) => {
        accessor!($set_fn, $section, $key, String);
    };

    ($get_fn:ident, $set_fn:ident, $section:expr, $key:expr) => {
        pub fn $get_fn(&self) -> Option<String> {
            self.get($section, $key)
        }

        accessor!($set_fn, $section, $key);
    };
}

impl Ini {
    pub fn new() -> Ini {
        Ini {
            dictionary: HashMap::new(),
        }
    }

    pub fn get(&self, section: &str, key: &str) -> Option<String> {
        self.dictionary
            .get(section)
            .and_then(|map| map.get(key))
            .cloned()
    }

    pub fn set<T: Display>(&mut self, section: &str, key: &str, value: T) -> Option<String> {
        if let Some(map) = self.dictionary.get_mut(section) {
            map.insert(key.to_string(), value.to_string())
        } else {
            let mut new = HashMap::new();
            new.insert(key.to_string(), value.to_string());
            self.dictionary.insert(section.to_string(), new);
            None
        }
    }

    pub fn remove(&mut self, section: &str, key: &str) -> Option<String> {
        self.dictionary
            .get_mut(section)
            .and_then(|map| map.remove(key))
    }

    pub fn read_from(path: &mut PathBuf) -> Result<Ini> {
        let mut ini = Ini::new();

        if path.is_dir() {
            path.push("desktop.ini")
        };

        if !fs::exists(&path).reason(|| "file system error on", Some(path))? {
            return Ok(ini);
        }

        let string = read_to_string_system(path)?;
        let mut section = "";
        for line in string.lines() {
            let line = line.trim();
            if line.starts_with("#") || line.is_empty() {
                continue;
            } else if line.starts_with("[") {
                section = line;
            } else if let Some((tag, value)) = line.split_once("=") {
                ini.set(section, tag, value);
            }
        }

        Ok(ini)
    }

    pub fn write_to(&self, path: &mut PathBuf) -> Result<()> {
        if path.is_dir() {
            path.push("desktop.ini")
        };

        write_string_system(path, &self.to_string())
    }

    accessor!(info_tip, set_info_tip, "[.ShellClassInfo]", "InfoTip");
    accessor!(execution, set_execution, "[.CustomExecution]", "Target");
    accessor!(
        icon_resource,
        set_icon_resource,
        "[.ShellClassInfo]",
        "IconResource"
    );
    accessor!(
        localized_resource_name,
        set_localized_resource_name,
        "[.ShellClassInfo]",
        "LocalizedResourceName"
    );
    accessor!(
        title,
        set_title,
        "[{F29F85E0-4FF9-1068-AB91-08002B27B3D9}]",
        "Prop2"
    );
    accessor!(
        subject,
        set_subject,
        "[{F29F85E0-4FF9-1068-AB91-08002B27B3D9}]",
        "Prop3"
    );
    accessor!(
        author,
        set_author,
        "[{F29F85E0-4FF9-1068-AB91-08002B27B3D9}]",
        "Prop4"
    );
    accessor!(
        comments,
        set_comments,
        "[{F29F85E0-4FF9-1068-AB91-08002B27B3D9}]",
        "Prop6"
    );

    pub fn confirm_execution(&self) -> Option<bool> {
        self.get("[.CustomExecution]", "ConfirmExecution")
            .map(|s| s != "0")
    }

    pub fn set_confirm_execution(&mut self, value: bool) -> Option<String> {
        self.set(
            "[.CustomExecution]",
            "ConfirmExecution",
            if value { "1" } else { "0" },
        )
    }

    pub fn set_directory_class(&mut self) -> Option<String> {
        self.set("[.ShellClassInfo]", "DirectoryClass", DIRECTORY_CLASS)
    }

    pub fn tags(&self) -> Option<Vec<String>> {
        let tags = self.get("[{F29F85E0-4FF9-1068-AB91-08002B27B3D9}]", "Prop5")?;

        match tags
            .split_once(',')
            .map_or(tags.as_str(), |(_, tags)| tags)
            .trim()
        {
            "" => Some(Vec::new()),
            tags => Some(tags.split(",").map(|s| s.trim().to_string()).collect()),
        }
    }

    pub fn set_tags(&mut self, tags: &[String]) {
        if tags.is_empty() {
            self.remove("[{F29F85E0-4FF9-1068-AB91-08002B27B3D9}]", "Prop5");
        } else {
            self.set(
                "[{F29F85E0-4FF9-1068-AB91-08002B27B3D9}]",
                "Prop5",
                format!("31,{}", tags.join(";")),
            );
        }
    }

    pub fn add_tags(&mut self, tags: &[String]) {
        let mut new_tags = self.tags().unwrap_or_default();
        new_tags.extend_from_slice(tags);
        new_tags.sort();
        new_tags.dedup();
        self.set_tags(&new_tags);
    }

    pub fn remove_tags(&mut self, tags: &[String]) {
        let new_tags: Vec<String> = self
            .tags()
            .unwrap_or_default()
            .into_iter()
            .filter(|s| !tags.contains(s))
            .collect();
        self.set_tags(&new_tags);
    }

    pub fn clear_tags(&mut self) -> Option<String> {
        self.remove("[{F29F85E0-4FF9-1068-AB91-08002B27B3D9}]", "Prop5")
    }

    pub fn args(&self, path: &str) -> Option<Vec<String>> {
        Some(parse_args(&self.get("[.CustomExecution]", "Args")?, path))
    }

    pub fn set_args(&mut self, args: &[String]) {
        if args.is_empty() {
            self.remove("[.CustomExecution]", "Args");
        } else {
            self.set("[.CustomExecution]", "Args", join_args(args));
        }
    }

    fn ordered(&self) -> Vec<&String> {
        let preferred_sections = [
            "[.ShellClassInfo]",
            "[{F29F85E0-4FF9-1068-AB91-08002B27B3D9}]",
        ];

        let mut sections: Vec<&String> = self.dictionary.keys().collect();
        sections.sort();

        let mut ordered_sections: Vec<&String> = Vec::with_capacity(sections.len());

        for name in preferred_sections {
            if let Some(s) = sections.extract_if(.., |s| s == &name).next() {
                ordered_sections.push(s)
            }
        }

        ordered_sections.extend(sections);

        ordered_sections
    }
}

impl Display for Ini {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for section in self.ordered() {
            if let Some(map) = self.dictionary.get(section)
                && !map.is_empty()
            {
                writeln!(f, "\n{section}")?;
                for (key, value) in map {
                    writeln!(f, "{key}={value}")?;
                }
            }
        }

        Ok(())
    }
}

impl Debug for Ini {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let dot = "· ".dimmed().to_string();
        macro_rules! print_info {
            ($key: expr, $repeat: expr, $value: expr) => {
                if let Some(s) = $value {
                    writeln!(f, "    {} {} {s}", $key.cyan(), dot.repeat($repeat))?;
                }
            };
        }

        writeln!(
            f,
            "{}\n\n{}",
            "INI brief information".bold().cyan(),
            "Shell Class Info:".bright_magenta()
        )?;
        print_info!("LocalizedResourceName: ", 5, self.localized_resource_name());
        print_info!("InfoTip: ", 12, self.info_tip());
        print_info!("IconResource:", 10, self.icon_resource());

        if let Some(tags) = self.tags() {
            let sep = ", ".bright_yellow().bold().to_string();
            writeln!(f, "{}\n    {}", "Tags:".bright_magenta(), tags.join(&sep))?;
        }

        writeln!(f, "{}", "Other Summary Information:".bright_magenta())?;
        print_info!("Title: ", 13, self.title());
        print_info!("Subject: ", 12, self.subject());
        print_info!("Author:", 13, self.author());
        print_info!("Comments:", 12, self.comments());

        if let Some(execution) = self.execution() {
            writeln!(
                f,
                "{}\n    {} {} {execution}",
                "Execution:".bright_magenta(),
                "Target:".cyan(),
                dot.repeat(13)
            )?;
            let confirm_str = if self.confirm_execution().unwrap_or_default() {
                "on".green().to_string()
            } else {
                "off".yellow().to_string()
            };
            print_info!("Second Confirmation: ", 6, Some(confirm_str));
        }

        writeln!(f, "\n{}", "Raw INI file content:".bold().cyan())?;
        Display::fmt(&self, f)?;

        Ok(())
    }
}
