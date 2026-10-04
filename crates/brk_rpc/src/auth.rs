use std::{
    io,
    path::{Path, PathBuf},
};

use brk_error::Error;

#[derive(Clone, Debug)]
pub enum Auth {
    None,
    UserPass(String, String),
    CookieFile(PathBuf),
}

/// A cookie read failure that names the file (a bare "Permission denied" says nothing).
pub(crate) fn cookie_error(path: &Path, error: io::Error) -> Error {
    Error::IO(io::Error::new(
        error.kind(),
        format!("cannot read the RPC cookie {}: {error}", path.display()),
    ))
}
