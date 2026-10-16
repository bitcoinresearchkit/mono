use std::{
    fs, io,
    path::{Path, PathBuf},
};

use base64::{Engine, engine::general_purpose::STANDARD};
#[cfg(feature = "async")]
use brk_error::Error;

#[derive(Clone, Debug)]
pub enum Auth {
    None,
    UserPass(String, String),
    CookieFile(PathBuf),
}

/// A cookie read failure that names the file (a bare "Permission denied" says nothing).
#[cfg(feature = "async")]
pub(crate) fn cookie_error(path: &Path, error: io::Error) -> Error {
    Error::IO(cookie_io_error(path, error))
}

fn cookie_io_error(path: &Path, error: io::Error) -> io::Error {
    io::Error::new(
        error.kind(),
        format!("cannot read the RPC cookie {}: {error}", path.display()),
    )
}

/// The trimmed cookie, or an error that names the file.
pub(crate) fn read_cookie(path: &Path) -> io::Result<String> {
    fs::read_to_string(path)
        .map(|cookie| cookie.trim().to_owned())
        .map_err(|error| cookie_io_error(path, error))
}

/// The `Authorization` header value for `user:password` credentials (a cookie holds them as is).
pub(crate) fn basic(credentials: &str) -> String {
    format!("Basic {}", STANDARD.encode(credentials))
}
