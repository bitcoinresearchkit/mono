use std::{
    fs::{File, OpenOptions, create_dir_all},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use jiff::{Timestamp, civil::Date, tz};
use tracing::{Level, Metadata};
use tracing_subscriber::fmt::MakeWriter;

mod writer;
pub use writer::FileWriter;

const MAX_WRITES_PER_SEC: u64 = 100;
const LEVELS: usize = 5;
/// Level names, as file suffixes and in log lines.
pub const LEVEL_NAMES: [&str; LEVELS] = ["error", "warn", "info", "debug", "trace"];

const fn level_index(level: Level) -> usize {
    match level {
        Level::ERROR => 0,
        Level::WARN => 1,
        Level::INFO => 2,
        Level::DEBUG => 3,
        Level::TRACE => 4,
    }
}

/// The day of a file this writer produces, `YYYY-MM-DD.txt` (combined) or
/// `YYYY-MM-DD_<level>.txt`: `Some("YYYY-MM-DD")`, `None` for any other name.
pub fn log_file_date(name: &str) -> Option<&str> {
    let (date, rest) = name.strip_suffix(".txt")?.split_at_checked(10)?;
    date.parse::<Date>().ok()?;
    (rest.is_empty()
        || rest
            .strip_prefix('_')
            .is_some_and(|level| LEVEL_NAMES.contains(&level)))
    .then_some(date)
}

struct Cached {
    date: String,
    file: File,
}

/// Today's file for one destination, reopened when the date changes.
#[derive(Default)]
struct DailyFile(Option<Cached>);

fn open_append(path: &Path) -> io::Result<File> {
    OpenOptions::new().create(true).append(true).open(path)
}

impl DailyFile {
    fn write(
        &mut self,
        dir: &Path,
        date: &str,
        buf: &[u8],
        path: impl FnOnce() -> PathBuf,
    ) -> io::Result<()> {
        let cached = match self.0.as_mut() {
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
                self.0.insert(Cached {
                    date: date.to_string(),
                    file,
                })
            }
        };
        cached.file.write_all(buf)
    }
}

/// One level's file and its rate limit, kept under one mutex so the
/// per-second rollover and the admission count change together.
#[derive(Default)]
struct LevelSlot {
    file: DailyFile,
    second: u64,
    count: u64,
}

impl LevelSlot {
    fn admit(&mut self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if now != self.second {
            self.second = now;
            self.count = 0;
        }
        self.count += 1;
        self.count <= MAX_WRITES_PER_SEC
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
    combined: Mutex<DailyFile>,
    levels: [Mutex<LevelSlot>; LEVELS],
}

/// Rate-limited daily log files: one combined file plus one per tracing level.
/// Each level has its own 100/sec limiter so a chatty level cannot starve the
/// others; the combined file is a true superset, written iff a per-level write
/// is permitted (or unconditionally for events with no associated level).
pub struct RateLimitedFile(Inner);

impl RateLimitedFile {
    pub fn new(dir: &Path) -> io::Result<Self> {
        create_dir_all(dir)?;
        Ok(Self(Inner {
            dir: dir.to_path_buf(),
            combined: Mutex::default(),
            levels: Default::default(),
        }))
    }
}

impl<'a> MakeWriter<'a> for RateLimitedFile {
    type Writer = FileWriter<'a>;

    /// Fallback used only by callers that bypass `make_writer_for`. The fmt
    /// layer always provides metadata, so this path is unused in practice; it
    /// writes to the combined file only and skips per-level routing.
    fn make_writer(&'a self) -> Self::Writer {
        FileWriter {
            inner: &self.0,
            level: None,
        }
    }

    fn make_writer_for(&'a self, meta: &Metadata<'_>) -> Self::Writer {
        FileWriter {
            inner: &self.0,
            level: Some(*meta.level()),
        }
    }
}
