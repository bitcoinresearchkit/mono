use std::{
    fs::File,
    io::{self, Write},
    path::Path,
};

#[cfg(target_os = "linux")]
use std::fs;

#[cfg(target_os = "macos")]
use std::process::Command;

pub struct MemoryMonitor {
    pid: u32,
    writer: File,
}

impl MemoryMonitor {
    pub fn new(pid: u32, csv_path: &Path) -> io::Result<Self> {
        let mut writer = File::create(csv_path)?;
        writeln!(writer, "timestamp_ms,phys_footprint,phys_footprint_peak")?;

        Ok(Self { pid, writer })
    }

    /// Record memory usage at the given timestamp
    pub fn record(&mut self, elapsed_ms: u128) -> io::Result<()> {
        if let Ok((footprint, peak)) = self.get_memory_usage() {
            writeln!(self.writer, "{},{},{}", elapsed_ms, footprint, peak)?;
        }
        Ok(())
    }

    /// Get memory usage in bytes
    /// Returns (current_bytes, peak_bytes)
    fn get_memory_usage(&self) -> io::Result<(u64, u64)> {
        #[cfg(target_os = "linux")]
        {
            self.get_memory_usage_linux()
        }

        #[cfg(target_os = "macos")]
        {
            self.get_memory_usage_macos()
        }
    }

    #[cfg(target_os = "linux")]
    fn get_memory_usage_linux(&self) -> io::Result<(u64, u64)> {
        let status_content = fs::read_to_string(format!("/proc/{}/status", self.pid))?;

        let mut vm_rss = None;
        let mut vm_hwm = None;

        for line in status_content.lines() {
            if line.starts_with("VmRSS:") {
                if let Some(value_str) = line.split_whitespace().nth(1) {
                    if let Ok(kb) = value_str.parse::<u64>() {
                        vm_rss = Some(kb * 1024); // KiB to bytes
                    }
                }
            } else if line.starts_with("VmHWM:") {
                if let Some(value_str) = line.split_whitespace().nth(1) {
                    if let Ok(kb) = value_str.parse::<u64>() {
                        vm_hwm = Some(kb * 1024); // KiB to bytes
                    }
                }
            }
        }

        match (vm_rss, vm_hwm) {
            (Some(rss), Some(hwm)) => Ok((rss, hwm)),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Failed to parse memory info from /proc/[pid]/status",
            )),
        }
    }

    #[cfg(target_os = "macos")]
    fn get_memory_usage_macos(&self) -> io::Result<(u64, u64)> {
        let output = Command::new("footprint")
            .args(["-p", &self.pid.to_string()])
            .output()?;

        let stdout = String::from_utf8(output.stdout).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8 from footprint")
        })?;

        parse_footprint_output(&stdout).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Failed to parse footprint output",
            )
        })
    }
}

#[cfg(target_os = "macos")]
fn parse_footprint_output(output: &str) -> Option<(u64, u64)> {
    let mut phys_footprint = None;
    let mut phys_footprint_peak = None;

    for line in output.lines() {
        let line = line.trim();

        if line.starts_with("phys_footprint:") {
            // Format: "phys_footprint: 7072 KB"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                // parts[0] = "phys_footprint:"
                // parts[1] = "7072"
                // parts[2] = "KB"
                phys_footprint = parse_size_to_bytes(parts[1], parts[2]);
            }
        } else if line.starts_with("phys_footprint_peak:") {
            // Format: "phys_footprint_peak: 15 MB"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                phys_footprint_peak = parse_size_to_bytes(parts[1], parts[2]);
            }
        }
    }

    match (phys_footprint, phys_footprint_peak) {
        (Some(f), Some(p)) => Some((f, p)),
        _ => None,
    }
}

#[cfg(target_os = "macos")]
fn parse_size_to_bytes(value: &str, unit: &str) -> Option<u64> {
    let value: f64 = value.parse().ok()?;

    let multiplier = match unit.to_uppercase().as_str() {
        "KB" => 1024.0,                   // KiB to bytes
        "MB" => 1024.0 * 1024.0,          // MiB to bytes
        "GB" => 1024.0 * 1024.0 * 1024.0, // GiB to bytes
        _ => return None,
    };

    Some((value * multiplier) as u64)
}
