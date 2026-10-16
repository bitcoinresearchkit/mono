use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use brk_logger::log_file_date;

use crate::Result;

pub fn discover(location: &Path) -> Result<Vec<PathBuf>> {
    let mut days: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    for entry in fs::read_dir(location)
        .map_err(|e| format!("Cannot read logs at {}: {e}", location.display()))?
    {
        let path = entry?.path();
        if !path.is_file() {
            continue;
        }
        let Some(day) = path
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(log_file_date)
        else {
            continue;
        };
        days.entry(day.to_string())
            .or_default()
            .push(fs::canonicalize(path)?);
    }
    let mut files = Vec::new();
    for (day, mut paths) in days {
        paths.sort();
        if let Some(combined) = paths
            .iter()
            .find(|p| p.file_name().unwrap() == format!("{day}.txt").as_str())
        {
            files.push(combined.clone());
        } else {
            files.extend(paths);
        }
    }
    if files.is_empty() {
        return Err(format!("No daily log files at {}", location.display()).into());
    }
    Ok(files)
}
