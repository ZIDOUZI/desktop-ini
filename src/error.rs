use crate::ErrorAction;
use std::io;
use std::io::Read;
use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("I/O error when {action} {path}: {source}")]
    Io {
        action: String,
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("Registry error: {0}")]
    RegistryError(#[from] io::Error),

    #[error("Permission denied on {path}: {source}")]
    PermissionDenied {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("Failed parse integer: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
    
    #[error("No value founded. this is not an error, when this displayed, somewhere wrong.")]
    NoValue,

    #[error("This operation is only supported on Windows.")]
    UnsupportedOS,
}

pub trait IoReason {
    type Output;
    fn reason<S : AsRef<str>, O: FnOnce() -> S>(self, action: O, path: Option<&PathBuf>) -> Self::Output;
}

impl IoReason for io::Error {
    type Output = Error;
    fn reason<S : AsRef<str>, O: FnOnce() -> S>(self, action: O, path: Option<&PathBuf>) -> Error {
        let path_str = match path {
            None => String::new(),
            Some(p) => p.to_string_lossy().to_string(),
        };
        Error::Io {
            action: action().as_ref().to_string(),
            path: path_str,
            source: self,
        }
    }
}

impl<T> IoReason for io::Result<T> {
    type Output = Result<T>;

    fn reason<S : AsRef<str>, O: FnOnce() -> S>(self, action: O, path: Option<&PathBuf>) -> Self::Output {
        self.map_err(|err| err.reason(action, path))
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait ResultHandle<T> {
    fn decide(self, error_action: ErrorAction) -> Result<Option<T>>;
}

impl<T> ResultHandle<T> for Result<T> {
    fn decide(self, error_action: ErrorAction) -> Result<Option<T>> {
        match self {
            Ok(value) => Ok(Some(value)),
            Err(e) => {
                if error_action == ErrorAction::Stop {
                    return Err(e);
                }

                if error_action != ErrorAction::SilentlyContinue {
                    eprintln!("{e}")
                }

                if error_action == ErrorAction::Inquire {
                    println!("Press Enter to continue...");
                    io::stdin().read_exact(&mut [0]).ok();
                }
                Ok(None)
            }
        }
    }
}
