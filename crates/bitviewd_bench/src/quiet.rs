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
    time::{Duration, Instant},
};

// Shared by all data directories and users on this machine. Never unlink a lock file:
// another process could otherwise lock a different inode at the same path.
const LOCK_PATH: &str = "/tmp/bitviewd-bench.lock";
/// Heavy by what it is: any process named `bitview*` (bitviewd serving or not, bitview_mcp, a
/// test server, another bench) or a Rust build, whatever it does at the moment sampled.
const BUILDERS: [&str; 5] = ["cargo", "rustc", "rustup", "clippy-driver", "cargo-clippy"];
/// Heavy by what it does, over a window (3 s at the start, the minute between two watch
/// readings mid-run, so a burst averages out and only sustained load counts): a process using
/// at least a quarter of a core, or moving at least 20 MB/s to or from disk (the disk reading
/// covers the processes this user can inspect; the CPU covers all). Bitcoin Core is what the
/// bench runs beside.
const START_WINDOW: Duration = Duration::from_secs(3);
const CPU_CORES: f64 = 0.25;
const DISK_BYTES_PER_S: f64 = 20e6;
/// Heavy by what it does in total: the whole machine's CPU over the window, minus this bench and
/// Bitcoin Core, at least one core's worth. It sees what per-process rates cannot: processes born
/// and dead inside the window (a dd loop, a compile of many `cc1`). The clean reference run
/// (1fcbd0e99) had at most 65% of a core in its three busiest non-bench, non-bitcoind rows of any
/// minute (PerfPowerServices 47%, launchd 37%, WindowServer 26%, never together), so a full core
/// is above that noise and below any real job (a build uses all cores; one dd loop one).
const MACHINE_CORES: f64 = 1.0;
/// At the start only: a process other than Bitcoin Core holding this much memory is a server
/// someone left running, however it is named (a test server is a binary named by its sha).
const IDLE_RSS_KB: u64 = 1 << 20;
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

    let before = Snapshot::take()?;
    thread::sleep(START_WINDOW);
    let after = Snapshot::take()?;
    let (mut heavy, busiest) = assess(&before, &after);
    heavy.extend(large(&after));
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

/// The minute-by-minute watch: each `check` judges the work since the previous one.
pub struct Watch(Snapshot);

impl Watch {
    pub fn new() -> io::Result<Self> {
        Snapshot::take().map(Self)
    }

    /// The heavy work (named, with why) since the last reading.
    pub fn check(&mut self) -> io::Result<Vec<String>> {
        let now = Snapshot::take()?;
        let heavy = assess(&self.0, &now).0;
        self.0 = now;
        Ok(heavy)
    }
}

struct Reading {
    pid: u32,
    name: String,
    cpu_s: f64,
    rss_kb: u64,
    disk: Option<u64>,
}

/// Every process, and the machine's total busy CPU seconds, at one moment.
struct Snapshot {
    at: Instant,
    busy_s: f64,
    readings: Vec<Reading>,
}

impl Snapshot {
    fn take() -> io::Result<Self> {
        Ok(Self {
            at: Instant::now(),
            busy_s: machine_busy_s()?,
            readings: readings()?,
        })
    }
}

/// The heavy processes between two snapshots, and the busiest ones with their CPU share.
fn assess(before: &Snapshot, after: &Snapshot) -> (Vec<String>, String) {
    let secs = after.at.duration_since(before.at).as_secs_f64();
    let this = id();
    let mut heavy = Vec::new();
    let mut busy = Vec::new();
    // This bench and Bitcoin Core: what the machine-wide total leaves out.
    let mut allowed_cpu = 0.0;
    for now in &after.readings {
        let named = now.name.starts_with("bitview") || BUILDERS.contains(&now.name.as_str());
        let then = before
            .readings
            .iter()
            .find(|r| r.pid == now.pid && r.name == now.name);
        let cpu = then.map_or(0.0, |then| (now.cpu_s - then.cpu_s) / secs);
        if now.pid == this || now.name == ALLOWED {
            allowed_cpu += cpu;
            continue;
        }
        if named {
            heavy.push(format!("{} (PID {})", now.name, now.pid));
        }
        // A process that appeared during the window has no rate: the machine-wide total sees it.
        let Some(then) = then else { continue };
        let disk = match (now.disk, then.disk) {
            (Some(a), Some(b)) => a.saturating_sub(b) as f64 / secs,
            _ => 0.0,
        };
        busy.push((cpu, &now.name));
        if named {
            continue;
        }
        if cpu >= CPU_CORES {
            heavy.push(format!(
                "{} (PID {}) at {:.0}% of a core",
                now.name,
                now.pid,
                cpu * 100.0
            ));
        } else if disk >= DISK_BYTES_PER_S {
            heavy.push(format!(
                "{} (PID {}) at {:.0} MB/s of disk",
                now.name,
                now.pid,
                disk / 1e6
            ));
        }
    }
    let others = (after.busy_s - before.busy_s) / secs - allowed_cpu;
    if others >= MACHINE_CORES {
        heavy.push(format!(
            "the machine at {others:.1} cores besides this bench and Bitcoin Core"
        ));
    }
    busy.sort_by(|a, b| b.0.total_cmp(&a.0));
    let busiest = busy
        .iter()
        .take(BUSIEST)
        .map(|(cpu, name)| format!("{name} {:.0}%", cpu * 100.0))
        .collect::<Vec<_>>()
        .join(", ");
    (heavy, busiest)
}

/// Processes holding at least `IDLE_RSS_KB`, Bitcoin Core and this bench aside.
fn large(snapshot: &Snapshot) -> Vec<String> {
    let this = id();
    snapshot
        .readings
        .iter()
        .filter(|r| r.pid != this && r.name != ALLOWED && r.rss_kb >= IDLE_RSS_KB)
        .map(|r| {
            format!(
                "{} (PID {}) holding {:.1} GiB",
                r.name,
                r.pid,
                r.rss_kb as f64 / f64::from(1 << 20)
            )
        })
        .collect()
}

/// `ps` rows (PID, cumulative CPU time, resident KiB, command by its file name), with each
/// one's disk bytes.
fn readings() -> io::Result<Vec<Reading>> {
    let output = Command::new("ps")
        .args(["-Ao", "pid=,time=,rss=,comm="])
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
            let mut rest = line;
            let mut field = || {
                let (field, after) = rest.trim_start().split_once(char::is_whitespace)?;
                rest = after;
                Some(field)
            };
            let pid: u32 = field()?.parse().ok()?;
            let cpu_s = seconds(field()?)?;
            let rss_kb: u64 = field()?.parse().ok()?;
            // The command is the rest of the line, spaces included.
            let command = rest.trim();
            Some(Reading {
                pid,
                name: Path::new(command)
                    .file_name()?
                    .to_string_lossy()
                    .into_owned(),
                cpu_s,
                rss_kb,
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

/// Seconds of CPU the whole machine has spent busy (user, system and nice, all cores).
#[cfg(target_os = "macos")]
fn machine_busy_s() -> io::Result<f64> {
    let mut info = libc::host_cpu_load_info {
        cpu_ticks: [0; libc::CPU_STATE_MAX as usize],
    };
    let mut count = libc::HOST_CPU_LOAD_INFO_COUNT;
    // The deprecation only points at the `mach2` crate, and a dependency is more than this call.
    #[allow(deprecated)]
    let host = unsafe { libc::mach_host_self() };
    // SAFETY: `HOST_CPU_LOAD_INFO` fills a `host_cpu_load_info` of `count` words.
    let result = unsafe {
        libc::host_statistics(
            host,
            libc::HOST_CPU_LOAD_INFO,
            (&raw mut info).cast(),
            &mut count,
        )
    };
    if result != libc::KERN_SUCCESS {
        return Err(io::Error::other(format!(
            "host_statistics failed: {result}"
        )));
    }
    let ticks = info.cpu_ticks;
    let busy = ticks[libc::CPU_STATE_USER as usize]
        + ticks[libc::CPU_STATE_SYSTEM as usize]
        + ticks[libc::CPU_STATE_NICE as usize];
    Ok(f64::from(busy) / ticks_per_s())
}

#[cfg(target_os = "linux")]
fn machine_busy_s() -> io::Result<f64> {
    let stat = std::fs::read_to_string("/proc/stat")?;
    let busy = stat
        .lines()
        .next()
        .and_then(|line| line.strip_prefix("cpu "))
        .and_then(|fields| {
            let mut fields = fields.split_whitespace().map(|f| f.parse::<f64>().ok());
            // user, nice, system
            Some(fields.next()?? + fields.next()?? + fields.next()??)
        })
        .ok_or_else(|| io::Error::other("unreadable /proc/stat"))?;
    Ok(busy / ticks_per_s())
}

fn ticks_per_s() -> f64 {
    // SAFETY: `sysconf` only reads a constant.
    unsafe { libc::sysconf(libc::_SC_CLK_TCK) as f64 }
}
