use std::{
    fs::File,
    io::{self, Write},
    path::Path,
};

#[cfg(target_os = "linux")]
use std::fs;

#[cfg(target_os = "macos")]
use libproc::pid_rusage::{RUsageInfoV2, pidrusage};

pub struct IoMonitor {
    pid: u32,
    writer: File,
}

impl IoMonitor {
    pub fn new(pid: u32, csv_path: &Path) -> io::Result<Self> {
        let mut writer = File::create(csv_path)?;
        writeln!(writer, "timestamp_ms,bytes_read,bytes_written")?;

        Ok(Self { pid, writer })
    }

    /// Record I/O usage at the given timestamp
    pub fn record(&mut self, elapsed_ms: u128) -> io::Result<()> {
        if let Ok((read, written)) = self.get_io_usage() {
            writeln!(self.writer, "{},{},{}", elapsed_ms, read, written)?;
        }
        Ok(())
    }

    /// Get I/O usage in bytes
    /// Returns (bytes_read, bytes_written)
    fn get_io_usage(&self) -> io::Result<(u64, u64)> {
        #[cfg(target_os = "linux")]
        {
            self.get_io_usage_linux()
        }

        #[cfg(target_os = "macos")]
        {
            self.get_io_usage_macos()
        }
    }

    #[cfg(target_os = "linux")]
    fn get_io_usage_linux(&self) -> io::Result<(u64, u64)> {
        let io_content = fs::read_to_string(format!("/proc/{}/io", self.pid))?;

        let mut read_bytes = None;
        let mut write_bytes = None;

        for line in io_content.lines() {
            if line.starts_with("read_bytes:") {
                if let Some(value_str) = line.split_whitespace().nth(1) {
                    read_bytes = value_str.parse::<u64>().ok();
                }
            } else if line.starts_with("write_bytes:") {
                if let Some(value_str) = line.split_whitespace().nth(1) {
                    write_bytes = value_str.parse::<u64>().ok();
                }
            }
        }

        match (read_bytes, write_bytes) {
            (Some(r), Some(w)) => Ok((r, w)),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Failed to parse I/O stats from /proc/[pid]/io",
            )),
        }
    }

    #[cfg(target_os = "macos")]
    fn get_io_usage_macos(&self) -> io::Result<(u64, u64)> {
        match pidrusage::<RUsageInfoV2>(self.pid as i32) {
            Ok(info) => Ok((info.ri_diskio_bytesread, info.ri_diskio_byteswritten)),
            Err(_) => Err(io::Error::other("Failed to get process I/O stats")),
        }
    }
}
