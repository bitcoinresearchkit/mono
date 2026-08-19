use parking_lot::Mutex;
use std::{
    fs,
    io::{self, BufWriter, Write},
    path::Path,
    time::Instant,
};

/// Patterns to match for progress tracking.
const PROGRESS_PATTERNS: &[&str] = &[
    "block ",    // "Indexing block 123..."
    "chain at ", // "Processing chain at 456..."
];

pub struct ProgressionMonitor {
    csv_file: Mutex<BufWriter<fs::File>>,
    start_time: Instant,
}

impl ProgressionMonitor {
    pub fn new(csv_path: &Path) -> io::Result<Self> {
        let mut csv_file = BufWriter::new(fs::File::create(csv_path)?);
        writeln!(csv_file, "timestamp_ms,value")?;

        Ok(Self {
            csv_file: Mutex::new(csv_file),
            start_time: Instant::now(),
        })
    }

    /// Check message for progress patterns and record if found
    #[inline]
    pub fn check_and_record(&self, message: &str) {
        let Some(value) = parse_progress(message) else {
            return;
        };

        if value % 10 != 0 {
            return;
        }

        let elapsed_ms = self.start_time.elapsed().as_millis();
        let _ = writeln!(self.csv_file.lock(), "{},{}", elapsed_ms, value);
    }

    pub fn flush(&self) -> io::Result<()> {
        self.csv_file.lock().flush()
    }
}

/// Parse progress value from message
#[inline]
fn parse_progress(message: &str) -> Option<u64> {
    PROGRESS_PATTERNS
        .iter()
        .find_map(|pattern| parse_number_after(message, pattern))
}

/// Extract number immediately following the pattern
#[inline]
fn parse_number_after(message: &str, pattern: &str) -> Option<u64> {
    let start = message.find(pattern)?;
    let after = &message[start + pattern.len()..];

    let end = after
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(after.len());

    if end == 0 {
        return None;
    }

    after[..end].parse().ok()
}
