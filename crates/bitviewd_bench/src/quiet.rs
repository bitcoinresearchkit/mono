//! The quiet-machine rule of a full bench (perf-work.md): nothing power-hungry beside Bitcoin Core.
//! Ordinary system and light background processes pass. This refuses to start a run beside heavy
//! work, and a run that meets heavy work later says so in its `metadata.txt` (`comparable=false`).
//! It only reports; it never stops a process.
use std::{
    collections::{BTreeMap, HashMap},
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
/// at least a quarter of a core at the start (a refusal is retryable) or half a core mid-run
/// (system bursts, PerfPowerServices at 47% or launchd at 37% instantaneous, average near a
/// quarter over a minute), or moving at least 20 MB/s to or from disk (the disk reading covers
/// the processes this user can inspect; the CPU covers all). Bitcoin Core is what the bench
/// runs beside, and this bench's own ancestors (the `cargo run` that started it) are not judged.
const START_WINDOW: Duration = Duration::from_secs(3);
const START_CORES: f64 = 0.25;
const WATCH_CORES: f64 = 0.5;
const DISK_BYTES_PER_S: f64 = 20e6;
/// Heavy by what it does in total: the whole machine's USER CPU over the window, minus the user
/// time of this bench, its ancestors and Bitcoin Core, at least one core's worth. It sees what
/// per-process rates cannot: processes born and dead inside the window (a dd loop, a compile of
/// many `cc1`). System time is left out on purpose: the bench itself causes it (kernel_task is
/// invisible to `ps` yet in the host's system ticks: disk writeback, compression and swap
/// measured 0.2 to 0.9 of a core), while foreign work worth refusing (a build, a dd loop, a
/// busy script) burns user time. The clean reference run (1fcbd0e99) had at most 65% of a core
/// in its three busiest non-bench, non-bitcoind rows of any minute (PerfPowerServices 47%,
/// launchd 37%, WindowServer 26%, never together), so a full core is above that noise and
/// below any real job (a build uses all cores; one dd loop one).
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
    let (mut heavy, busiest) = assess(&before, &after, START_CORES);
    large(&after, &mut heavy);
    if !heavy.is_empty() {
        return Err(io::Error::other(format!(
            "Not a quiet machine: {} running; a full bench runs beside Bitcoin Core only (on a mini: ~/Developer/dots/shared/ship stop bitviewd mcp, wait for builds and other benchmarks; nothing is stopped for you)",
            heavy.lines().join(", ")
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
        let heavy = assess(&self.0, &now, WATCH_CORES).0.lines();
        self.0 = now;
        Ok(heavy)
    }
}

struct Reading {
    pid: u32,
    ppid: u32,
    name: String,
    /// User plus system, and user alone.
    cpu_s: f64,
    user_s: f64,
    rss_kb: u64,
    disk: Option<u64>,
}

/// Every process, and the machine's total user CPU seconds, at one moment.
struct Snapshot {
    at: Instant,
    user_s: f64,
    readings: Vec<Reading>,
}

impl Snapshot {
    fn take() -> io::Result<Self> {
        Ok(Self {
            at: Instant::now(),
            user_s: machine_user_s()?,
            readings: readings()?,
        })
    }
}

/// What makes work heavy, one entry per process however many reasons it has, plus the machine.
#[derive(Default)]
struct Heavy {
    processes: BTreeMap<u32, (String, Vec<String>)>,
    machine: Option<String>,
}

impl Heavy {
    fn add(&mut self, reading: &Reading, why: String) {
        self.processes
            .entry(reading.pid)
            .or_insert_with(|| (reading.name.clone(), Vec::new()))
            .1
            .push(why);
    }

    fn is_empty(&self) -> bool {
        self.processes.is_empty() && self.machine.is_none()
    }

    fn lines(self) -> Vec<String> {
        let mut lines: Vec<_> = self
            .processes
            .into_iter()
            .map(|(pid, (name, why))| format!("{name} (PID {pid}) {}", why.join(", ")))
            .collect();
        lines.extend(self.machine);
        lines
    }
}

/// This process and its ancestors (the `cargo run` that started it): never judged.
fn own_chain(readings: &[Reading]) -> Vec<u32> {
    let parents: HashMap<u32, u32> = readings.iter().map(|r| (r.pid, r.ppid)).collect();
    let mut chain = vec![id()];
    while let Some(&parent) = parents.get(chain.last().unwrap_or(&0)) {
        if parent == 0 || chain.contains(&parent) {
            break;
        }
        chain.push(parent);
    }
    chain
}

/// The heavy processes between two snapshots (`cores` a process may use), and the busiest ones
/// with their CPU share.
fn assess(before: &Snapshot, after: &Snapshot, cores: f64) -> (Heavy, String) {
    let secs = after.at.duration_since(before.at).as_secs_f64();
    let own = own_chain(&after.readings);
    let mut heavy = Heavy::default();
    let mut busy = Vec::new();
    // This bench, its ancestors and Bitcoin Core: what the machine-wide total leaves out.
    let mut allowed_user = 0.0;
    for now in &after.readings {
        let named = now.name.starts_with("bitview") || BUILDERS.contains(&now.name.as_str());
        let then = before
            .readings
            .iter()
            .find(|r| r.pid == now.pid && r.name == now.name);
        if own.contains(&now.pid) || now.name == ALLOWED {
            allowed_user += then.map_or(0.0, |then| (now.user_s - then.user_s) / secs);
            continue;
        }
        if named {
            heavy.add(now, "is a bitview or build process".into());
        }
        // A process that appeared during the window has no rate: the machine-wide total sees it.
        let Some(then) = then else { continue };
        let cpu = (now.cpu_s - then.cpu_s) / secs;
        let disk = match (now.disk, then.disk) {
            (Some(a), Some(b)) => a.saturating_sub(b) as f64 / secs,
            _ => 0.0,
        };
        busy.push((cpu, &now.name));
        if named {
            continue;
        }
        if cpu >= cores {
            heavy.add(now, format!("at {:.0}% of a core", cpu * 100.0));
        } else if disk >= DISK_BYTES_PER_S {
            heavy.add(now, format!("at {:.0} MB/s of disk", disk / 1e6));
        }
    }
    let others = (after.user_s - before.user_s) / secs - allowed_user;
    if others >= MACHINE_CORES {
        heavy.machine = Some(format!(
            "the machine at {others:.1} cores of user time besides this bench and Bitcoin Core"
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

/// Adds the processes holding at least `IDLE_RSS_KB`, Bitcoin Core and this bench's chain aside.
fn large(snapshot: &Snapshot, heavy: &mut Heavy) {
    let own = own_chain(&snapshot.readings);
    for r in &snapshot.readings {
        if !own.contains(&r.pid) && r.name != ALLOWED && r.rss_kb >= IDLE_RSS_KB {
            heavy.add(
                r,
                format!("holding {:.1} GiB", r.rss_kb as f64 / f64::from(1 << 20)),
            );
        }
    }
}

/// `ps` rows (PID, parent, cumulative user and total CPU time, resident KiB, command by its
/// file name), with each one's disk bytes.
fn readings() -> io::Result<Vec<Reading>> {
    let output = Command::new("ps")
        .args(["-Ao", "pid=,ppid=,utime=,time=,rss=,comm="])
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
            let ppid: u32 = field()?.parse().ok()?;
            let user_s = seconds(field()?)?;
            let cpu_s = seconds(field()?)?;
            let rss_kb: u64 = field()?.parse().ok()?;
            // The command is the rest of the line, spaces included.
            let command = rest.trim();
            Some(Reading {
                pid,
                ppid,
                name: Path::new(command)
                    .file_name()?
                    .to_string_lossy()
                    .into_owned(),
                cpu_s,
                user_s,
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

/// Seconds of user CPU the whole machine has spent (user and nice, all cores).
#[cfg(target_os = "macos")]
fn machine_user_s() -> io::Result<f64> {
    let mut info = libc::host_cpu_load_info {
        cpu_ticks: [0; libc::CPU_STATE_MAX as usize],
    };
    let mut count = libc::HOST_CPU_LOAD_INFO_COUNT;
    // The deprecations only point at the `mach2` crate, and a dependency is more than these calls.
    #[allow(deprecated)]
    // SAFETY: `mach_host_self` takes no input and returns a send right to the host port, which
    // `host_statistics` only uses and `mach_port_deallocate` gives back (each call adds a
    // reference), with `mach_task_self` naming this task. `HOST_CPU_LOAD_INFO` fills a
    // `host_cpu_load_info` of `count` words.
    let result = unsafe {
        let host = libc::mach_host_self();
        let result = libc::host_statistics(
            host,
            libc::HOST_CPU_LOAD_INFO,
            (&raw mut info).cast(),
            &mut count,
        );
        mach_port_deallocate(libc::mach_task_self(), host);
        result
    };
    if result != libc::KERN_SUCCESS {
        return Err(io::Error::other(format!(
            "host_statistics failed: {result}"
        )));
    }
    let ticks = info.cpu_ticks;
    // Summed as u64: four u32 counters overflow after weeks of uptime.
    let user = u64::from(ticks[libc::CPU_STATE_USER as usize])
        + u64::from(ticks[libc::CPU_STATE_NICE as usize]);
    Ok(user as f64 / ticks_per_s())
}

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn mach_port_deallocate(task: libc::mach_port_t, name: libc::mach_port_t) -> libc::c_int;
}

#[cfg(target_os = "linux")]
fn machine_user_s() -> io::Result<f64> {
    let stat = std::fs::read_to_string("/proc/stat")?;
    let busy = stat
        .lines()
        .next()
        .and_then(|line| line.strip_prefix("cpu "))
        .and_then(|fields| {
            let mut fields = fields.split_whitespace().map(|f| f.parse::<f64>().ok());
            // user, nice
            Some(fields.next()?? + fields.next()??)
        })
        .ok_or_else(|| io::Error::other("unreadable /proc/stat"))?;
    Ok(busy / ticks_per_s())
}

fn ticks_per_s() -> f64 {
    // SAFETY: `sysconf` only reads a constant.
    unsafe { libc::sysconf(libc::_SC_CLK_TCK) as f64 }
}
