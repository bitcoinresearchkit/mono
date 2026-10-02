use std::{
    fs::{File, OpenOptions, create_dir_all},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

use jiff::{Timestamp, tz};
use tracing::{Level, Metadata};
use tracing_subscriber::fmt::MakeWriter;

mod writer;
pub use writer::FileWriter;

const MAX_WRITES_PER_SEC: u64 = 100;
const LEVELS: usize = 5;
const LEVEL_SUFFIX: [&str; LEVELS] = ["error", "warn", "info", "debug", "trace"];

const fn level_index(level: Level) -> usize {
    match level {
        Level::ERROR => 0,
        Level::WARN => 1,
        Level::INFO => 2,
        Level::DEBUG => 3,
        Level::TRACE => 4,
    }
}

/// Returns true if `name` matches a file produced by this writer:
/// `YYYY-MM-DD.txt` or `YYYY-MM-DD_<level>.txt`.
pub fn is_log_file(name: &str) -> bool {
    let Some(stem) = name.strip_suffix(".txt") else {
        return false;
    };
    let Some((date, rest)) = stem.split_at_checked(10) else {
        return false;
    };
    if !is_date_yyyymmdd(date) {
        return false;
    }
    rest.is_empty()
        || rest
            .strip_prefix('_')
            .is_some_and(|s| LEVEL_SUFFIX.contains(&s))
}

fn is_date_yyyymmdd(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b[0..4].iter().all(u8::is_ascii_digit)
        && b[4] == b'-'
        && b[5..7].iter().all(u8::is_ascii_digit)
        && b[7] == b'-'
        && b[8..10].iter().all(u8::is_ascii_digit)
}

#[derive(Default)]
struct RateLimit {
    count: AtomicU64,
    last_second: AtomicU64,
}

impl RateLimit {
    fn can_write(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let last = self.last_second.load(Ordering::Relaxed);
        if now != last {
            self.last_second.store(now, Ordering::Relaxed);
            self.count.store(1, Ordering::Relaxed);
            true
        } else {
            self.count.fetch_add(1, Ordering::Relaxed) < MAX_WRITES_PER_SEC
        }
    }
}

struct Cached {
    date: String,
    file: File,
}

#[derive(Default)]
struct FileSlot(Mutex<Option<Cached>>);

fn open_append(path: &Path) -> io::Result<File> {
    OpenOptions::new().create(true).append(true).open(path)
}

impl FileSlot {
    fn write(
        &self,
        dir: &Path,
        date: &str,
        buf: &[u8],
        path: impl FnOnce() -> PathBuf,
    ) -> io::Result<()> {
        let mut guard = self.0.lock().unwrap();
        let cached = match guard.as_mut() {
            Some(c) if c.date == date => c,
            _ => {
                let p = path();
                let file = match open_append(&p) {
                    Ok(f) => f,
                    Err(e) if e.kind() == io::ErrorKind::NotFound => {
                        create_dir_all(dir)?;
                        open_append(&p)?
                    }
                    Err(e) => return Err(e),
                };
                guard.insert(Cached {
                    date: date.to_string(),
                    file,
                })
            }
        };
        cached.file.write_all(buf)
    }
}

fn today() -> String {
    Timestamp::now()
        .to_zoned(tz::TimeZone::system())
        .strftime("%Y-%m-%d")
        .to_string()
}

struct Inner {
    dir: PathBuf,
    level_limits: [RateLimit; LEVELS],
    combined_slot: FileSlot,
    level_slots: [FileSlot; LEVELS],
}

/// Rate-limited daily log files: one combined file plus one per tracing level.
/// Each level has its own 100/sec limiter so a chatty level cannot starve the
/// others; the combined file is a true superset, written iff a per-level write
/// is permitted (or unconditionally for events with no associated level).
pub struct RateLimitedFile(Arc<Inner>);

impl RateLimitedFile {
    pub fn new(dir: &Path) -> io::Result<Self> {
        create_dir_all(dir)?;
        Ok(Self(Arc::new(Inner {
            dir: dir.to_path_buf(),
            level_limits: Default::default(),
            combined_slot: FileSlot::default(),
            level_slots: Default::default(),
        })))
    }
}

impl<'a> MakeWriter<'a> for RateLimitedFile {
    type Writer = FileWriter;

    /// Fallback used only by callers that bypass `make_writer_for`. The fmt
    /// layer always provides metadata, so this path is unused in practice; it
    /// writes to the combined file only and skips per-level routing.
    fn make_writer(&'a self) -> Self::Writer {
        FileWriter {
            inner: Arc::clone(&self.0),
            level: None,
        }
    }

    fn make_writer_for(&'a self, meta: &Metadata<'_>) -> Self::Writer {
        FileWriter {
            inner: Arc::clone(&self.0),
            level: Some(*meta.level()),
        }
    }
}
