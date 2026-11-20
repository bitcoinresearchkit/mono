//! The quiet-machine rule of a full bench (perf-work.md): nothing power-hungry beside Bitcoin Core.
//! Ordinary system and light background processes pass. This refuses to start a run beside heavy
//! work, and a run that meets heavy work later says so in its `metadata.txt` (`comparable=false`).
//! It only reports; it never stops a process.
use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    path::Path,
    process::{Command, id},
    thread,
    time::Duration,
};

// Shared by all data directories and users on this machine. Never unlink a lock file:
// another process could otherwise lock a different inode at the same path.
const LOCK_PATH: &str = "/tmp/bitviewd-bench.lock";
/// Heavy by what it is: any process named `bitview*` (bitviewd serving or not, bitview_mcp, a
/// test server, another bench) or a Rust build, whatever it does at the moment sampled.
const BUILDERS: [&str; 5] = ["cargo", "rustc", "rustup", "clippy-driver", "cargo-clippy"];
/// Heavy by what it does: over this sample, a process using at least a quarter of a core, or
/// moving at least 20 MB/s to or from disk (the disk reading covers the processes this user can
/// inspect; the CPU covers all). Bitcoin Core is what the bench runs beside.
const SAMPLE: Duration = Duration::from_secs(3);
const CPU_CORES: f64 = 0.25;
const DISK_BYTES_PER_S: f64 = 20e6;
const ALLOWED: &str = "bitcoind";
/// How many of the busiest processes the recording names.
const BUSIEST: usize = 6;

/// The machine lock, held for the run, and the busiest processes at the start.
pub struct Quiet {
    _lock: File,
    pub busiest: String,
}

/// Takes the lock and refuses when heavy work runs.
pub fn acquire() -> io::Result<Quiet> {
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
    writeln!(lock, "{}", id())?;

    let (mut heavy, busiest) = watch()?;
    if !heavy.is_empty() {
        heavy.sort();
        return Err(io::Error::other(format!(
            "Not a quiet machine: {} running; a full bench runs beside Bitcoin Core only (on a mini: ~/Developer/dots/shared/ship stop bitviewd mcp, wait for builds and other benchmarks; nothing is stopped for you)",
            heavy.join(", ")
        )));
    }
    Ok(Quiet {
        _lock: lock,
        busiest,
    })
}

/// The heavy processes now (named, with why), for the start and for the minute-by-minute watch.
pub fn overlap() -> io::Result<Vec<String>> {
    Ok(watch()?.0)
}

struct Reading {
    pid: u32,
    name: String,
    cpu_s: f64,
    disk: Option<u64>,
}

/// Two readings `SAMPLE` apart: the heavy processes, and the busiest ones with their CPU share.
fn watch() -> io::Result<(Vec<String>, String)> {
    let before = readings()?;
    thread::sleep(SAMPLE);
    let after = readings()?;
    let secs = SAMPLE.as_secs_f64();
    let this = id();
    let mut heavy = Vec::new();
    let mut busy = Vec::new();
    for now in &after {
        if now.pid == this {
            continue;
        }
        let named = now.name.starts_with("bitview") || BUILDERS.contains(&now.name.as_str());
        // A process that appeared during the sample has no rate: the next reading sees it.
        let Some(then) = before
            .iter()
            .find(|r| r.pid == now.pid && r.name == now.name)
        else {
            if named {
                heavy.push(format!("{} (PID {})", now.name, now.pid));
            }
            continue;
        };
        let cpu = (now.cpu_s - then.cpu_s) / secs;
        let disk = match (now.disk, then.disk) {
            (Some(a), Some(b)) => a.saturating_sub(b) as f64 / secs,
            _ => 0.0,
        };
        busy.push((cpu, &now.name));
        if named {
            heavy.push(format!("{} (PID {})", now.name, now.pid));
        } else if now.name != ALLOWED && cpu >= CPU_CORES {
            heavy.push(format!(
                "{} (PID {}) at {:.0}% of a core",
                now.name,
                now.pid,
                cpu * 100.0
            ));
        } else if now.name != ALLOWED && disk >= DISK_BYTES_PER_S {
            heavy.push(format!(
                "{} (PID {}) at {:.0} MB/s of disk",
                now.name,
                now.pid,
                disk / 1e6
            ));
        }
    }
    busy.sort_by(|a, b| b.0.total_cmp(&a.0));
    let busiest = busy
        .iter()
        .take(BUSIEST)
        .map(|(cpu, name)| format!("{name} {:.0}%", cpu * 100.0))
        .collect::<Vec<_>>()
        .join(", ");
    Ok((heavy, busiest))
}

/// `ps` rows (PID, cumulative CPU time, command by its file name), with each one's disk bytes.
fn readings() -> io::Result<Vec<Reading>> {
    let output = Command::new("ps")
        .args(["-Ao", "pid=,time=,comm="])
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "ps failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    let text = String::from_utf8(output.stdout).map_err(io::Error::other)?;
    Ok(text
        .lines()
        .filter_map(|line| {
            let (pid, rest) = line.trim_start().split_once(char::is_whitespace)?;
            let (time, command) = rest.trim_start().split_once(char::is_whitespace)?;
            let pid: u32 = pid.parse().ok()?;
            Some(Reading {
                pid,
                name: Path::new(command.trim())
                    .file_name()?
                    .to_string_lossy()
                    .into_owned(),
                cpu_s: seconds(time)?,
                disk: disk_bytes(pid),
            })
        })
        .collect())
}

/// `[days-][hours:]minutes:seconds[.fraction]` as `ps` prints cumulative CPU time.
fn seconds(time: &str) -> Option<f64> {
    let (days, clock) = time.split_once('-').unwrap_or(("0", time));
    let mut total = 0.0;
    for part in clock.split(':') {
        total = total * 60.0 + part.parse::<f64>().ok()?;
    }
    Some(days.parse::<f64>().ok()? * 86400.0 + total)
}

/// Bytes the process has read and written, when this user may inspect it.
#[cfg(target_os = "macos")]
fn disk_bytes(pid: u32) -> Option<u64> {
    let mut info = std::mem::MaybeUninit::<libc::rusage_info_v4>::zeroed();
    // SAFETY: `RUSAGE_INFO_V4` fills a `rusage_info_v4` on success.
    let result = unsafe {
        libc::proc_pid_rusage(pid as i32, libc::RUSAGE_INFO_V4, info.as_mut_ptr().cast())
    };
    // SAFETY: initialized by the successful call.
    let info = (result == 0).then(|| unsafe { info.assume_init() })?;
    Some(info.ri_diskio_bytesread + info.ri_diskio_byteswritten)
}

#[cfg(target_os = "linux")]
fn disk_bytes(pid: u32) -> Option<u64> {
    let io = std::fs::read_to_string(format!("/proc/{pid}/io")).ok()?;
    let field = |key: &str| {
        io.lines()
            .find_map(|l| l.strip_prefix(key)?.trim().parse::<u64>().ok())
    };
    Some(field("read_bytes:")? + field("write_bytes:")?)
}
