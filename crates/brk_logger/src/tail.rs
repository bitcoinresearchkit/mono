use std::{collections::VecDeque, sync::LazyLock};

use jiff::Timestamp;
use parking_lot::Mutex;
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::{Layer, layer::Context};

use crate::format::text;

/// Lines kept in memory.
const KEPT: usize = 1_000;

/// One logged line.
#[derive(Clone)]
pub struct Line {
    /// Its place in this process's log, from 0 with no gaps.
    pub seq: u64,
    pub time: Timestamp,
    pub level: Level,
    /// What the log files show after the level.
    pub text: String,
}

/// The last lines logged, oldest first.
pub struct Recent {
    /// When this process began its log: a `seq` means something only with it.
    pub started: Timestamp,
    pub lines: Vec<Line>,
}

/// When this process began its log.
pub fn started() -> Timestamp {
    TAIL.lock().started
}

/// The kept lines after `after` (a `seq`), or all of them; lines that left memory meanwhile are
/// skipped, as the jump in `seq` shows.
pub fn recent(after: Option<u64>) -> Recent {
    let tail = TAIL.lock();
    let first = tail.next - tail.lines.len() as u64;
    // (Saturating: `after` comes from requests, and a panic here would hold the lock its own log line waits on.)
    let skip = after.map_or(0, |after| after.saturating_add(1).saturating_sub(first));
    let skip = usize::try_from(skip).unwrap_or(usize::MAX);
    Recent {
        started: tail.started,
        lines: tail.lines.iter().skip(skip).cloned().collect(),
    }
}

struct Tail {
    started: Timestamp,
    next: u64,
    lines: VecDeque<Line>,
}

static TAIL: LazyLock<Mutex<Tail>> = LazyLock::new(|| {
    Mutex::new(Tail {
        started: Timestamp::now(),
        next: 0,
        lines: VecDeque::with_capacity(KEPT),
    })
});

/// Keeps the last `KEPT` events the filter admits.
pub struct TailLayer;

impl<S: Subscriber> Layer<S> for TailLayer {
    fn on_event(&self, event: &Event<'_>, _: Context<'_, S>) {
        let text = text(event);
        let time = Timestamp::now();
        let level = *event.metadata().level();
        let mut tail = TAIL.lock();
        let seq = tail.next;
        tail.next += 1;
        if tail.lines.len() == KEPT {
            tail.lines.pop_front();
        }
        tail.lines.push_back(Line { seq, time, level, text });
    }
}
