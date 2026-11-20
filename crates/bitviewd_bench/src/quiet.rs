//! The clean-machine rule of a full bench (perf-work.md): only bitcoind running, the data on the
//! external disk, no other Bitview, both disks holding the minimum. A run that broke it is not
//! comparable; this refuses to start one, and a run that is broken into later says so in its
//! `metadata.txt` (`comparable=false`).
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    process::{Command, id},
};

use crate::benchmark::metadata;

// Shared by all data directories and users on this machine. Never unlink a lock file:
// another process could otherwise lock a different inode at the same path.
const LOCK_PATH: &str = "/tmp/bitviewd-bench.lock";
/// Any process whose name starts with `bitview` (bitviewd, bitview_mcp, another bench, a test
/// binary) or is one of these breaks the run: a Rust build competes for the disk and the CPU.
const BUILDERS: [&str; 5] = ["cargo", "rustc", "rustup", "clippy-driver", "cargo-clippy"];
/// What a mini's home keeps (macOS's folders and the bench's own upload folder); the rest is a
/// leftover sandbox, clone or bench tree. Dotfiles are the system's and the toolchain's.
const HOME: [&str; 11] = [
    "Applications",
    "Desktop",
    "Developer",
    "Documents",
    "Downloads",
    "Library",
    "Movies",
    "Music",
    "Pictures",
    "Public",
    "mo-bench",
];
/// How many of the busiest processes the recording names.
const BUSIEST: usize = 6;

/// The machine lock, held for the run, and the busiest processes at the start.
pub struct Quiet {
    _lock: File,
    pub busiest: String,
}

/// Takes the lock and refuses when any other Bitview or Rust build runs.
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

    let mut processes = processes()?;
    let foreign = foreign(&processes);
    if !foreign.is_empty() {
        return Err(io::Error::other(format!(
            "Not a clean machine: {} running; a full bench runs with only bitcoind (on a mini: ~/Developer/dots/shared/ship stop bitviewd mcp, stop other benchmarks and builds)",
            foreign.join(", ")
        )));
    }
    processes.sort_by(|a, b| b.cpu.total_cmp(&a.cpu));
    let busiest = processes
        .iter()
        .take(BUSIEST)
        .map(|p| format!("{} {:.1}%", p.name, p.cpu))
        .collect::<Vec<_>>()
        .join(", ");
    Ok(Quiet {
        _lock: lock,
        busiest,
    })
}

/// Refuses unless the data is on the external disk, which holds only Bitcoin Core's directory
/// and the data set, and the internal disk holds none of the work's leftovers.
pub fn check_disks(data_path: &Path, blocks_path: &Path) -> io::Result<()> {
    let data = resolve(data_path)?;
    let class = metadata::storage(&nearest(&data)).class;
    if !class.starts_with("external") {
        return Err(io::Error::other(format!(
            "Not a clean machine: the data directory {} is on {class} storage; a full bench keeps its data on the external disk",
            data.display()
        )));
    }
    let root = volume_root(&data)?;
    let kept: Vec<_> = [&data, &resolve(blocks_path)?]
        .iter()
        .filter_map(|path| path.strip_prefix(&root).ok()?.components().next())
        .map(|top| top.as_os_str().to_owned())
        .collect();
    let extra = leftovers(&root, |name| kept.iter().any(|k| k == name))?;
    if !extra.is_empty() {
        return Err(io::Error::other(format!(
            "Not a clean machine: {} holds {} besides Bitcoin Core's directory and the data set; delete them first",
            root.display(),
            extra.join(", ")
        )));
    }
    if cfg!(target_os = "macos") {
        let home = std::env::home_dir().ok_or_else(|| io::Error::other("no home directory"))?;
        let mut extra = leftovers(&home, |name| HOME.iter().any(|h| name == *h))?;
        extra.extend(leftovers(&home.join("Developer"), |name| name == "dots")?);
        if home.join("mo-bench/target").exists() {
            extra.push("mo-bench/target".to_owned());
        }
        if !extra.is_empty() {
            return Err(io::Error::other(format!(
                "Not a clean machine: the internal disk holds {} (leftover sandboxes, clones, bench trees or builds); delete them first",
                extra.join(", ")
            )));
        }
    }
    Ok(())
}

/// The names of the directory's entries that are neither a dotfile nor allowed; none when it is absent.
fn leftovers(dir: &Path, allowed: impl Fn(&std::ffi::OsStr) -> bool) -> io::Result<Vec<String>> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Ok(Vec::new());
    };
    let mut extra = Vec::new();
    for entry in entries {
        let name = entry?.file_name();
        if !name.as_encoded_bytes().starts_with(b".") && !allowed(&name) {
            extra.push(name.to_string_lossy().into_owned());
        }
    }
    extra.sort();
    Ok(extra)
}

/// The path with its nearest existing ancestor resolved: the data directory may not exist yet.
fn resolve(path: &Path) -> io::Result<PathBuf> {
    let near = nearest(path);
    Ok(near
        .canonicalize()?
        .join(path.strip_prefix(&near).unwrap_or(Path::new(""))))
}

fn nearest(path: &Path) -> PathBuf {
    path.ancestors()
        .find(|p| p.exists())
        .unwrap_or(Path::new("/"))
        .to_owned()
}

/// The mount point of the volume holding the path: the top of its run of equal device ids.
fn volume_root(path: &Path) -> io::Result<PathBuf> {
    let mut root = nearest(path);
    let device = fs::metadata(&root)?.dev();
    while let Some(parent) = root.parent() {
        if fs::metadata(parent)?.dev() != device {
            break;
        }
        root = parent.to_owned();
    }
    Ok(root)
}

/// The foreign processes now, named with their PIDs, for the minute-by-minute watch during a run.
pub fn overlap() -> io::Result<Vec<String>> {
    Ok(foreign(&processes()?))
}

struct Process {
    pid: u32,
    cpu: f32,
    name: String,
}

fn foreign(processes: &[Process]) -> Vec<String> {
    let this = id();
    processes
        .iter()
        .filter(|p| {
            p.pid != this && (p.name.starts_with("bitview") || BUILDERS.contains(&p.name.as_str()))
        })
        .map(|p| format!("{} (PID {})", p.name, p.pid))
        .collect()
}

/// `ps` rows (PID, CPU share, command), the command by its file name.
fn processes() -> io::Result<Vec<Process>> {
    let output = Command::new("ps")
        .args(["-Ao", "pid=,pcpu=,comm="])
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
            let (cpu, command) = rest.trim_start().split_once(char::is_whitespace)?;
            Some(Process {
                pid: pid.parse().ok()?,
                cpu: cpu.parse().ok()?,
                name: Path::new(command.trim())
                    .file_name()?
                    .to_string_lossy()
                    .into_owned(),
            })
        })
        .collect())
}
