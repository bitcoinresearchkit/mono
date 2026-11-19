use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    process::{Command, id},
};

// Shared by all data directories and users on this machine. Never unlink a lock file:
// another process could otherwise lock a different inode at the same path.
const LOCK_PATH: &str = "/tmp/bitviewd-bench.lock";
const PROCESSES: [&str; 4] = ["bitviewd", "bitview_mcp", "bitviewd_bench", "rustc"];

pub fn acquire() -> io::Result<File> {
    let mut lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(LOCK_PATH)?;
    if let Err(error) = lock.try_lock() {
        let mut holder = String::new();
        lock.read_to_string(&mut holder)?;
        return Err(io::Error::other(format!(
            "Cannot lock {LOCK_PATH}: {error}; recorded holder PID: {}",
            holder.trim()
        )));
    }
    lock.set_len(0)?;
    lock.seek(SeekFrom::Start(0))?;
    writeln!(lock, "{}", id())?;
    lock.flush()?;

    for name in PROCESSES {
        let output = Command::new("pgrep").args(["-x", name]).output()?;
        if !output.status.success() && output.status.code() != Some(1) {
            return Err(io::Error::other(format!(
                "pgrep -x {name} failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }
        let pids = String::from_utf8(output.stdout).map_err(io::Error::other)?;
        let others: Vec<_> = pids
            .lines()
            .filter(|pid| *pid != id().to_string())
            .collect();
        if !others.is_empty() {
            return Err(io::Error::other(format!(
                "Found {name} running (PID {}); stop it before benchmarking (on a mini: ~/Developer/dots/shared/ship stop bitviewd mcp; stop any benchmark separately)",
                others.join(", ")
            )));
        }
    }
    Ok(lock)
}
