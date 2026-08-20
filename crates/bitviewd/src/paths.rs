use std::path::{Path, PathBuf};

pub fn default_bitview_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap();
    Path::new(&home).join(".bitview")
}

pub fn default_logs_dir() -> PathBuf {
    default_bitview_dir().join("logs")
}

pub fn fix_user_path(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/").or(path.strip_prefix("$HOME/"))
        && let Ok(home) = std::env::var("HOME")
    {
        return PathBuf::from(home).join(rest);
    }
    PathBuf::from(path)
}
