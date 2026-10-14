use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Disk usage of the indexed data
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DiskUsage {
    /// Human-readable brk data size (e.g., "48.8 GiB")
    pub brk: String,
    /// brk data size in bytes
    pub brk_bytes: u64,
    /// Human-readable Bitcoin blocks directory size
    pub bitcoin: String,
    /// Bitcoin blocks directory size in bytes
    pub bitcoin_bytes: u64,
    /// Ratio of BRK bytes to Bitcoin bytes; zero when Bitcoin bytes are zero.
    pub ratio: f64,
}

impl DiskUsage {
    pub fn new(brk_bytes: u64, bitcoin_bytes: u64) -> Self {
        let ratio = if bitcoin_bytes > 0 {
            brk_bytes as f64 / bitcoin_bytes as f64
        } else {
            0.0
        };
        Self {
            brk: format_bytes(brk_bytes),
            brk_bytes,
            bitcoin: format_bytes(bitcoin_bytes),
            bitcoin_bytes,
            ratio,
        }
    }
}

fn format_bytes(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = KIB * 1024;
    const GIB: u64 = MIB * 1024;
    const TIB: u64 = GIB * 1024;

    if bytes >= TIB {
        format!("{:.2} TiB", bytes as f64 / TIB as f64)
    } else if bytes >= GIB {
        format!("{:.2} GiB", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.2} MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.2} KiB", bytes as f64 / KIB as f64)
    } else {
        format!("{} B", bytes)
    }
}
