use std::{
    ops::Add,
    sync::{
        LazyLock,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use brk_types::Timestamp;

/// A moment the graveyard keeps: nanoseconds since the Unix epoch, as the wall clock read once at
/// start and the monotonic clock since tell it. A clock step never moves it back within a run, it
/// reads the same on every call for one moment (a tree's time served twice is one time), and any
/// moment a file holds is one, however long before the machine booted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Moment(u64);

/// How far [`Moment::now`] runs ahead of the wall clock read at start: past what a run whose
/// clock was ahead (set back since) left on disk, so what happens now comes after it.
static AHEAD: AtomicU64 = AtomicU64::new(0);

impl Moment {
    pub fn now() -> Self {
        static START: LazyLock<(Instant, u64)> = LazyLock::new(|| {
            let wall = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |since| since.as_nanos() as u64);
            (Instant::now(), wall)
        });
        let (instant, wall) = *START;
        Self(wall + AHEAD.load(Ordering::Relaxed) + instant.elapsed().as_nanos() as u64)
    }

    /// [`Self::now`] from then on later than `moment`: one read back from disk.
    pub fn pass(moment: Self) {
        let behind = moment.0.saturating_sub(Self::now().0);
        if behind > 0 {
            AHEAD.fetch_add(behind + 1, Ordering::Relaxed);
        }
    }

    pub fn from_unix_nanos(nanos: u64) -> Self {
        Self(nanos)
    }

    pub fn unix_nanos(self) -> u64 {
        self.0
    }

    /// Its whole second.
    pub fn timestamp(self) -> Timestamp {
        Timestamp::from((self.0 / 1_000_000_000) as usize)
    }
}

impl Add<Duration> for Moment {
    type Output = Self;

    fn add(self, duration: Duration) -> Self {
        Self(self.0.saturating_add(duration.as_nanos() as u64))
    }
}
