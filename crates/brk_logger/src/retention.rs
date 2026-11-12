use std::{fs, path::Path, thread, time::Duration};

use crate::rate_limit::log_file_date;

const DAY: Duration = Duration::from_secs(24 * 60 * 60);
/// How long a log file is kept, from its last write.
const MAX_AGE: Duration = Duration::from_secs(7 * DAY.as_secs());

/// Deletes the log files in `dir` last written more than `MAX_AGE` ago, now and then once a day, on a thread of its own:
/// however long a server runs, busy or quiet, it keeps about a week of them. Without the thread
/// (the system refusing one), logging goes on and the files stay, with a warning. Called once
/// the logger is up.
pub fn delete_old_daily(dir: &Path) {
    let dir = dir.to_path_buf();
    let spawned = thread::Builder::new()
        .name("log-retention".into())
        .spawn(move || {
            loop {
                delete_old(&dir);
                thread::sleep(DAY);
            }
        });
    if let Err(error) = spawned {
        tracing::warn!(%error, "old log files won't be deleted: no thread for it");
    }
}

fn delete_old(dir: &Path) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if entry.file_name().to_str().is_some_and(|name| log_file_date(name).is_some())
            && let Ok(meta) = entry.metadata()
            && let Ok(modified) = meta.modified()
            && modified.elapsed().is_ok_and(|age| age > MAX_AGE)
        {
            let _ = fs::remove_file(entry.path());
        }
    }
}
