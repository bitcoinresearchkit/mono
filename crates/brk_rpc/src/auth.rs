use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
};

use base64::{Engine, engine::general_purpose::STANDARD};

#[derive(Clone)]
pub enum Auth {
    None,
    UserPass(String, String),
    CookieFile(PathBuf),
}

/// The password never shows, wherever an `Auth` (or a client or connection holding one) is
/// formatted.
impl fmt::Debug for Auth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => f.write_str("None"),
            Self::UserPass(user, _) => f.debug_tuple("UserPass").field(user).field(&"…").finish(),
            Self::CookieFile(path) => f.debug_tuple("CookieFile").field(path).finish(),
        }
    }
}

/// A cookie read failure that names the file (a bare "Permission denied" says nothing).
pub(crate) fn cookie_io_error(path: &Path, error: io::Error) -> io::Error {
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
