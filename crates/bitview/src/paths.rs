use std::{env::home_dir, path::PathBuf};

/// `~/.bitview`; none without a home directory.
pub fn default_bitview_dir() -> Option<PathBuf> {
    Some(home_dir()?.join(".bitview"))
}

pub fn expand_home(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/").or(path.strip_prefix("$HOME/"))
        && let Some(home) = home_dir()
    {
        return home.join(rest);
    }
    PathBuf::from(path)
}
