use std::{
    fs::{File, OpenOptions, create_dir_all},
    io::{self, Write},
    path::{Path, PathBuf},
};

use jiff::civil::Date;
use parking_lot::Mutex;
use tracing::Level;

const MAX_WRITES_PER_SEC: u32 = 100;
const LEVELS: usize = 5;
/// Level names, as log lines show them.
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

/// The day of a file this logger wrote: `YYYY-MM-DD.txt`, or `YYYY-MM-DD_<level>.txt`, the per-level
/// copies it wrote until they were dropped (still named here so the week's cleanup deletes them).
/// `Some("YYYY-MM-DD")`, `None` for any other name.
pub fn log_file_date(name: &str) -> Option<&str> {
    let (date, rest) = name.strip_suffix(".txt")?.split_at_checked(10)?;
    date.parse::<Date>().ok()?;
    (rest.is_empty()
        || rest
            .strip_prefix('_')
            .is_some_and(|level| LEVEL_NAMES.contains(&level)))
    .then_some(date)
}

fn open_append(path: &Path) -> io::Result<File> {
    OpenOptions::new().create(true).append(true).open(path)
}

/// The log file of each day, `YYYY-MM-DD.txt` in `dir`. A level gets `MAX_WRITES_PER_SEC` lines a
/// second, so a chatty one can neither fill the disk nor crowd the others out.
pub struct LogFile {
    dir: PathBuf,
    state: Mutex<State>,
}

#[derive(Default)]
struct State {
    /// The file being written and its day.
    open: Option<(String, File)>,
    /// The second being counted, and each level's lines in it.
    second: i64,
    counts: [u32; LEVELS],
}

impl LogFile {
    pub fn new(dir: &Path) -> io::Result<Self> {
        create_dir_all(dir)?;
        Ok(Self {
            dir: dir.to_path_buf(),
            state: Mutex::default(),
        })
    }

    /// Appends `line`, logged at `second` of `date` (`YYYY-MM-DD`), unless its level has had its
    /// lines for that second. One lock holds the count and the write, so lines land in order.
    pub fn write(&self, second: i64, date: &str, level: Level, line: &str) -> io::Result<()> {
        let state = &mut *self.state.lock();
        if state.second != second {
            state.second = second;
            state.counts = [0; LEVELS];
        }
        let count = &mut state.counts[level_index(level)];
        *count += 1;
        if *count > MAX_WRITES_PER_SEC {
            return Ok(());
        }
        let file = match state.open.as_mut() {
            Some((day, file)) if day == date => file,
            _ => {
                let path = self.dir.join(format!("{date}.txt"));
                let file = match open_append(&path) {
                    Err(e) if e.kind() == io::ErrorKind::NotFound => {
                        create_dir_all(&self.dir)?;
                        open_append(&path)?
                    }
                    file => file?,
                };
                &mut state.open.insert((date.to_owned(), file)).1
            }
        };
        file.write_all(line.as_bytes())
    }
}
