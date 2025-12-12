use crate::error::{IoReason, Result};
use crate::platform::current_encoding;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

pub fn read_to_string_system(path: &PathBuf) -> Result<String> {
    let mut file = fs::File::open(path).reason(|| "open file", Some(path))?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .reason(|| "read file", Some(path))?;

    let enc = current_encoding();
    let (text, _, _) = enc.decode(&buf);
    Ok(text.into_owned())
}

pub fn write_string_system(path: &PathBuf, content: &str) -> Result<()> {
    let enc = current_encoding();
    let (bytes, _, _) = enc.encode(content);

    let mut file = fs::File::create(path).reason(|| "open file", Some(path))?;
    file.write_all(&bytes).reason(|| "write file", Some(path))
}
