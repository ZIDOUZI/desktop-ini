#[cfg(windows)]
mod windows {
    use encoding_rs::Encoding;
    use std::ffi::OsStr;
    use std::io;
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;
    use windows_sys::Win32::Globalization::GetACP;
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    pub fn current_encoding() -> &'static Encoding {
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
    }

    fn quote_windows_arg(arg: &str) -> String {
        if arg.is_empty() {
            return "\"\"".to_string();
        }

        let needs_quotes = arg.chars().any(|c| c.is_whitespace() || c == '"');
        if !needs_quotes {
            return arg.to_string();
        }

        let mut out = String::with_capacity(arg.len() + 2);
        out.push('"');
        for ch in arg.chars() {
            if ch == '"' {
                out.push('\\');
            }
            out.push(ch);
        }
        out.push('"');
        out
    }

    fn to_wide_null(s: &OsStr) -> Vec<u16> {
        let mut v: Vec<u16> = s.encode_wide().collect();
        v.push(0);
        v
    }

    pub fn shell_execute_runas(
        exe_path: &Path,
        args: &[String],
        work_dir: &Path,
    ) -> io::Result<()> {
        let verb = to_wide_null(OsStr::new("runas"));
        let file = to_wide_null(exe_path.as_os_str());

        let params_str = args
            .iter()
            .map(|a| quote_windows_arg(a))
            .collect::<Vec<String>>()
            .join(" ");

        let params = if params_str.is_empty() {
            Vec::from([0u16])
        } else {
            to_wide_null(OsStr::new(&params_str))
        };

        let dir = to_wide_null(work_dir.as_os_str());

        let result = unsafe {
            ShellExecuteW(
                std::ptr::null_mut(),
                verb.as_ptr(),
                file.as_ptr(),
                params.as_ptr(),
                dir.as_ptr(),
                SW_SHOWNORMAL,
            )
        };

        let code = result as isize;
        if code <= 32 {
            Err(io::Error::from_raw_os_error(code as i32))
        } else {
            Ok(())
        }
    }
}

#[cfg(not(windows))]
mod not_windows {
    use encoding_rs::Encoding;
    use std::io::{Error, ErrorKind, Result};
    use std::path::Path;

    pub fn shell_execute_runas(_exe_path: &Path, _args: &[String], _work_dir: &Path) -> Result<()> {
        Err(Error::new(
            ErrorKind::Other,
            "ShellExecute elevation is only supported on Windows",
        ))
    }

    pub fn current_encoding() -> &'static Encoding {
        encoding_rs::UTF_8
    }
}

#[cfg(windows)]
pub use windows::*;

#[cfg(not(windows))]
pub use not_windows::*;
