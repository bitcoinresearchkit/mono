use std::path::{Path, PathBuf};

pub(crate) fn dot_brk_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap();
    Path::new(&home).join(".brk")
}

pub(crate) fn dot_brk_log_path() -> PathBuf {
    dot_brk_path().join("logs")
}

pub(crate) fn default_brk_path() -> PathBuf {
    dot_brk_path()
}

pub(crate) fn fix_user_path(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/").or(path.strip_prefix("$HOME/"))
        && let Ok(home) = std::env::var("HOME")
    {
        return PathBuf::from(home).join(rest);
    }
    PathBuf::from(path)
}
