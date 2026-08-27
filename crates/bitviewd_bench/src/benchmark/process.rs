use std::{
    fs::File,
    io::{self, BufWriter, Error, Write},
    path::Path,
};

#[cfg(target_os = "linux")]
use std::{fs, io::ErrorKind};

#[cfg(target_os = "macos")]
use libproc::pid_rusage::{RUsageInfoV4, pidrusage};

pub struct ProcessMonitor {
    pid: u32,
    memory: BufWriter<File>,
    io: BufWriter<File>,
}

impl ProcessMonitor {
    pub fn new(pid: u32, path: &Path) -> io::Result<Self> {
        let mut memory = BufWriter::new(File::create(path.join("memory.csv"))?);
        writeln!(memory, "timestamp_ms,physical_bytes,peak_physical_bytes")?;
        let mut io = BufWriter::new(File::create(path.join("io.csv"))?);
        writeln!(io, "timestamp_ms,read_bytes,written_bytes")?;
        Ok(Self { pid, memory, io })
    }

    #[cfg(target_os = "macos")]
    pub fn record(&mut self, elapsed_ms: u128) -> io::Result<()> {
        let info = pidrusage::<RUsageInfoV4>(self.pid as i32)
            .map_err(|_| Error::other("Failed to read process usage"))?;
        writeln!(
            self.memory,
            "{elapsed_ms},{},{}",
            info.ri_phys_footprint, info.ri_lifetime_max_phys_footprint,
        )?;
        writeln!(
            self.io,
            "{elapsed_ms},{},{}",
            info.ri_diskio_bytesread, info.ri_diskio_byteswritten,
        )
    }

    #[cfg(target_os = "linux")]
    pub fn record(&mut self, elapsed_ms: u128) -> io::Result<()> {
        let (physical, peak) = self.memory()?;
        let (read, written) = self.io()?;
        writeln!(self.memory, "{elapsed_ms},{physical},{peak}")?;
        writeln!(self.io, "{elapsed_ms},{read},{written}")
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.memory.flush()?;
        self.io.flush()
    }

    #[cfg(target_os = "linux")]
    fn memory(&self) -> io::Result<(u64, u64)> {
        let status = fs::read_to_string(format!("/proc/{}/status", self.pid))?;
        let mut physical = None;
        let mut peak = None;
        for line in status.lines() {
            let (field, value) = line.split_once(':').unwrap_or_default();
            let bytes = || {
                value
                    .split_whitespace()
                    .next()
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(|value| value * 1024)
            };
            match field {
                "VmRSS" => physical = bytes(),
                "VmHWM" => peak = bytes(),
                _ => {}
            }
        }
        physical
            .zip(peak)
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Invalid process memory data"))
    }

    #[cfg(target_os = "linux")]
    fn io(&self) -> io::Result<(u64, u64)> {
        let usage = fs::read_to_string(format!("/proc/{}/io", self.pid))?;
        let mut read = None;
        let mut written = None;
        for line in usage.lines() {
            let (field, value) = line.split_once(':').unwrap_or_default();
            let value = || value.trim().parse::<u64>().ok();
            match field {
                "read_bytes" => read = value(),
                "write_bytes" => written = value(),
                _ => {}
            }
        }
        read.zip(written)
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Invalid process I/O data"))
    }
}
