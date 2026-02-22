use crate::error::{IoReason, Result};
use crate::platform::current_encoding;
use std::fs;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;

#[cfg(windows)]
use std::os::windows::fs::OpenOptionsExt;

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

    let existed = path.exists();

    let mut file = {
        let mut opt = OpenOptions::new();
        opt.write(true).create(true).truncate(true);
        #[cfg(windows)]
        {
            // FILE_ATTRIBUTE_HIDDEN(0x2) | FILE_ATTRIBUTE_SYSTEM(0x4)
            opt.attributes(0x6);
        }
        opt.open(path).reason(|| "open file", Some(path))?
    };

    file.write_all(&bytes).reason(|| "write file", Some(path))?;

    if !existed && let Some(parent) = path.parent() {
        let meta = fs::metadata(parent).reason(|| "read metadata", Some(&parent.to_path_buf()))?;
        let mut perms = meta.permissions();
        perms.set_readonly(true);
        fs::set_permissions(parent, perms)
            .reason(|| "set permissions", Some(&parent.to_path_buf()))?;
    }

    Ok(())
}
