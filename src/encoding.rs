use crate::error::{IoReason, Result};
use encoding_rs::Encoding;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetACP() -> u32; // Get system ANSI code page
}

fn current_encoding() -> &'static Encoding {
    #[cfg(windows)]
    match unsafe { GetACP() } {
        936 => encoding_rs::GBK,
        950 => encoding_rs::BIG5,
        932 => encoding_rs::SHIFT_JIS,
        1250 => encoding_rs::WINDOWS_1250,
        1251 => encoding_rs::WINDOWS_1251,
        1252 => encoding_rs::WINDOWS_1252,
        1253 => encoding_rs::WINDOWS_1253,
        1254 => encoding_rs::WINDOWS_1254,
        1255 => encoding_rs::WINDOWS_1255,
        1256 => encoding_rs::WINDOWS_1256,
        1257 => encoding_rs::WINDOWS_1257,
        1258 => encoding_rs::WINDOWS_1258,
        // intentional
        65001 | _ => encoding_rs::UTF_8,
    }
    #[cfg(not(windows))]
    encoding_rs::UTF_8
}

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
