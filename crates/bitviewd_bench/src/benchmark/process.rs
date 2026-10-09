use std::{
    fs::File,
    io::{self, BufWriter, Error, Write},
    mem::MaybeUninit,
    path::Path,
};

#[cfg(target_os = "linux")]
use std::{fs, io::ErrorKind};

#[cfg(target_os = "macos")]
use libproc::pid_rusage::{RUsageInfoV4, pidrusage};

/// Samples this process's memory, disk I/O and CPU time, plus system swap.
pub struct ProcessMonitor {
    pid: u32,
    memory: BufWriter<File>,
    io: BufWriter<File>,
    cpu: BufWriter<File>,
}

struct Sample {
    physical: u64,
    peak: u64,
    resident: u64,
    read: u64,
    written: u64,
}

impl ProcessMonitor {
    pub fn new(pid: u32, path: &Path) -> io::Result<Self> {
        let mut memory = BufWriter::new(File::create(path.join("memory.csv"))?);
        writeln!(
            memory,
            "timestamp_ms,physical_bytes,peak_physical_bytes,resident_bytes,swap_used_bytes"
        )?;
        let mut io = BufWriter::new(File::create(path.join("io.csv"))?);
        writeln!(io, "timestamp_ms,read_bytes,written_bytes,page_ins")?;
        let mut cpu = BufWriter::new(File::create(path.join("cpu.csv"))?);
        writeln!(cpu, "timestamp_ms,user_ms,system_ms")?;
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
        writeln!(
            self.cpu,
            "{elapsed_ms},{},{}",
            millis(usage.ru_utime),
            millis(usage.ru_stime)
        )?;
        self.memory.flush()?;
        self.io.flush()?;
        self.cpu.flush()
    }

    #[cfg(target_os = "macos")]
    fn sample(&self) -> io::Result<Sample> {
        let info = pidrusage::<RUsageInfoV4>(self.pid as i32)
            .map_err(|_| Error::other("Failed to read process usage"))?;
        Ok(Sample {
            physical: info.ri_phys_footprint,
            peak: info.ri_lifetime_max_phys_footprint,
            resident: info.ri_resident_size,
            read: info.ri_diskio_bytesread,
            written: info.ri_diskio_byteswritten,
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

fn millis(time: libc::timeval) -> u64 {
    time.tv_sec as u64 * 1_000 + time.tv_usec as u64 / 1_000
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
