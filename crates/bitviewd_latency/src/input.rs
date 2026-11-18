use std::{
    fs,
    path::{Path, PathBuf},
};

use brk_logger::log_file_date;

use crate::Result;

/// Each day's log file, oldest first.
pub fn discover(location: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(location)
        .map_err(|e| format!("Cannot read logs at {}: {e}", location.display()))?
    {
        let path = entry?.path();
        // `<day>.txt` only: the `<day>_<level>.txt` beside it in older logs repeats its lines.
        let is_day = path.file_name().and_then(|n| n.to_str()).is_some_and(|name| {
            log_file_date(name).is_some_and(|day| name.strip_suffix(".txt") == Some(day))
        });
        if is_day && path.is_file() {
            files.push(fs::canonicalize(path)?);
        }
    }
    if files.is_empty() {
        return Err(format!("No daily log files at {}", location.display()).into());
    }
    files.sort();
    Ok(files)
}
