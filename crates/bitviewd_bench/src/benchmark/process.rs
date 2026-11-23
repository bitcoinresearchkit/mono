use std::{
    fs::File,
    io::{self, BufWriter, Error, Write},
    mem::MaybeUninit,
    path::Path,
};

use bitview_default::serial::Counts;

#[cfg(target_os = "linux")]
use std::{fs, io::ErrorKind};

/// Samples this process's memory, disk I/O and CPU time, plus system swap.
pub struct ProcessMonitor {
    pid: u32,
    memory: BufWriter<File>,
    io: BufWriter<File>,
    cpu: BufWriter<File>,
}

/// The peak column: macOS's lifetime peak footprint; Linux's `VmHWM`, the peak of the whole
/// resident size, as Linux keeps no peak of anonymous and shared memory.
#[cfg(target_os = "macos")]
const PEAK: &str = "peak_physical_bytes";
#[cfg(target_os = "linux")]
const PEAK: &str = "peak_resident_bytes";

/// `cpu.csv`'s counters after the CPU times: the process's retired instructions and cycles
/// (macOS's `proc_pid_rusage`; Linux has no such counter without `perf_event_open`).
#[cfg(target_os = "macos")]
const COUNTERS: &str = ",instructions,cycles";
#[cfg(target_os = "linux")]
const COUNTERS: &str = "";

struct Sample {
    physical: u64,
    peak: u64,
    resident: u64,
    read: u64,
    written: u64,
    counters: Option<(u64, u64)>,
}

impl ProcessMonitor {
    pub fn new(pid: u32, path: &Path) -> io::Result<Self> {
        let mut memory = BufWriter::new(File::create(path.join("memory.csv"))?);
        writeln!(
            memory,
            "timestamp_ms,physical_bytes,{PEAK},resident_bytes,swap_used_bytes"
        )?;
        let mut io = BufWriter::new(File::create(path.join("io.csv"))?);
        writeln!(io, "timestamp_ms,read_bytes,written_bytes,page_ins")?;
        let mut cpu = BufWriter::new(File::create(path.join("cpu.csv"))?);
        writeln!(cpu, "timestamp_ms,user_ms,system_ms{COUNTERS}")?;
        Ok(Self {
            pid,
            memory,
            io,
            cpu,
        })
    }

    /// Writes one row per file and flushes, so a crash loses at most the sample in progress.
    pub fn record(&mut self, elapsed_ms: u128) -> io::Result<()> {
        let sample = self.sample()?;
        let usage = usage()?;
        let swap = swap_used()?;
        writeln!(
            self.memory,
            "{elapsed_ms},{},{},{},{swap}",
            sample.physical, sample.peak, sample.resident
        )?;
        writeln!(
            self.io,
            "{elapsed_ms},{},{},{}",
            sample.read, sample.written, usage.ru_majflt
        )?;
        let counters = sample
            .counters
            .map_or(String::new(), |(instructions, cycles)| format!(",{instructions},{cycles}"));
        writeln!(
            self.cpu,
            "{elapsed_ms},{},{}{counters}",
            millis(usage.ru_utime),
            millis(usage.ru_stime)
        )?;
        self.memory.flush()?;
        self.io.flush()?;
        self.cpu.flush()
    }

    #[cfg(target_os = "macos")]
    fn sample(&self) -> io::Result<Sample> {
        let info = rusage_v4(self.pid)?;
        Ok(Sample {
            physical: info.ri_phys_footprint,
            peak: info.ri_lifetime_max_phys_footprint,
            resident: info.ri_resident_size,
            read: info.ri_diskio_bytesread,
            written: info.ri_diskio_byteswritten,
            counters: Some((info.ri_instructions, info.ri_cycles)),
        })
    }

    #[cfg(target_os = "linux")]
    fn sample(&self) -> io::Result<Sample> {
        let status = fs::read_to_string(format!("/proc/{}/status", self.pid))?;
        // Anonymous and shared memory, like macOS's footprint; resident adds mapped file pages.
        let physical = (proc_field(&status, "RssAnon")? + proc_field(&status, "RssShmem")?) * 1024;
        let peak = proc_field(&status, "VmHWM")? * 1024;
        let io = fs::read_to_string(format!("/proc/{}/io", self.pid))?;
        Ok(Sample {
            physical,
            peak,
            resident: proc_field(&status, "VmRSS")? * 1024,
            read: proc_field(&io, "read_bytes")?,
            written: proc_field(&io, "write_bytes")?,
            counters: None,
        })
    }
}

/// This process's resource usage: CPU times and page faults that read from disk.
fn usage() -> io::Result<libc::rusage> {
    let mut usage = MaybeUninit::<libc::rusage>::zeroed();
    // SAFETY: `getrusage` fills the provided `rusage` on success.
    if unsafe { libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) } != 0 {
        return Err(Error::last_os_error());
    }
    // SAFETY: initialized by the successful call above.
    Ok(unsafe { usage.assume_init() })
}

/// A process's resource usage, with the kernel's retired-instruction and cycle counters.
#[cfg(target_os = "macos")]
fn rusage_v4(pid: u32) -> io::Result<libc::rusage_info_v4> {
    let mut info = MaybeUninit::<libc::rusage_info_v4>::zeroed();
    // SAFETY: `RUSAGE_INFO_V4` fills a `rusage_info_v4`.
    let result = unsafe {
        libc::proc_pid_rusage(pid as i32, libc::RUSAGE_INFO_V4, info.as_mut_ptr().cast())
    };
    if result != 0 {
        let error = Error::last_os_error();
        return Err(Error::new(
            error.kind(),
            format!("Failed to read process usage: {error}"),
        ));
    }
    // SAFETY: initialized by the successful call above.
    Ok(unsafe { info.assume_init() })
}

/// What this process has spent so far: the serial recompute's clock.
#[cfg(target_os = "macos")]
pub fn counts() -> io::Result<Counts> {
    let info = rusage_v4(std::process::id())?;
    let usage = usage()?;
    Ok(Counts {
        instructions: info.ri_instructions,
        cycles: info.ri_cycles,
        cpu_ns: (micros(usage.ru_utime) + micros(usage.ru_stime)) * 1_000,
    })
}

#[cfg(target_os = "linux")]
pub fn counts() -> io::Result<Counts> {
    Err(Error::new(
        io::ErrorKind::Unsupported,
        "retired-instruction counters need macOS's proc_pid_rusage",
    ))
}

fn millis(time: libc::timeval) -> u64 {
    micros(time) / 1_000
}

fn micros(time: libc::timeval) -> u64 {
    time.tv_sec as u64 * 1_000_000 + time.tv_usec as u64
}

/// System-wide swap in use.
#[cfg(target_os = "macos")]
fn swap_used() -> io::Result<u64> {
    let mut swap = MaybeUninit::<libc::xsw_usage>::zeroed();
    let mut size = size_of::<libc::xsw_usage>();
    // SAFETY: `vm.swapusage` fills an `xsw_usage` of exactly `size` bytes.
    let result = unsafe {
        libc::sysctlbyname(
            c"vm.swapusage".as_ptr(),
            swap.as_mut_ptr().cast(),
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };
    if result != 0 {
        return Err(Error::last_os_error());
    }
    // SAFETY: initialized by the successful call above.
    Ok(unsafe { swap.assume_init() }.xsu_used)
}

/// System-wide swap in use.
#[cfg(target_os = "linux")]
fn swap_used() -> io::Result<u64> {
    let meminfo = fs::read_to_string("/proc/meminfo")?;
    Ok((proc_field(&meminfo, "SwapTotal")? - proc_field(&meminfo, "SwapFree")?) * 1024)
}

/// The first number of a `field: value` line in a `/proc` file.
#[cfg(target_os = "linux")]
fn proc_field(content: &str, name: &str) -> io::Result<u64> {
    content
        .lines()
        .find_map(|line| {
            let (field, value) = line.split_once(':')?;
            (field == name).then(|| value.split_whitespace().next()?.parse().ok())?
        })
        .ok_or_else(|| Error::new(ErrorKind::InvalidData, format!("Missing {name} in /proc")))
}
