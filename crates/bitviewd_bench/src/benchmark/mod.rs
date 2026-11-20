use std::{
    fs,
    io::{Error as IoError, ErrorKind},
    mem,
    path::{Path, PathBuf},
    process::id as process_id,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use bitview::UpdateContext;
use bitview_default::DefaultPlugins;
use brk_error::{Error, Result};
use brk_logger::register_hook;
use brk_types::{Height, Timestamp};
use disk::DiskMonitor;
use parking_lot::Mutex;
use process::ProcessMonitor;
use trace::TraceMonitor;
use tracing::Level;

use crate::quiet;

mod disk;
pub mod metadata;
mod process;
mod replay;
mod trace;

#[derive(Clone)]
pub struct Benchmark(Arc<Inner>);

#[derive(Clone, Copy)]
enum Outcome {
    Complete,
    Failed,
    Aborted,
}

impl Outcome {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Failed => "failed",
            Self::Aborted => "aborted",
        }
    }
}

/// Process samples are cheap; the data-directory scan walks every file, so it runs less often,
/// on its own thread so a slow scan never delays process samples.
const PROCESS_INTERVAL: Duration = Duration::from_secs(5);
const DISK_INTERVAL: Duration = Duration::from_secs(60);

struct Inner {
    path: PathBuf,
    disk: Arc<Mutex<DiskMonitor>>,
    trace: Arc<TraceMonitor>,
    /// Plugin data was already there: its undo does not cover a replay.
    resumed: bool,
    /// The first Bitview or build seen beside the run: it is not comparable.
    overlap: Arc<Mutex<Option<String>>>,
    stop: Arc<AtomicBool>,
    state: Mutex<State>,
}

enum State {
    Ready,
    Running {
        started_at: Instant,
        monitors: [JoinHandle<Result<()>>; 2],
    },
    Finished,
}

impl Benchmark {
    pub fn new(
        data_path: &Path,
        blocks_path: &Path,
        chain_height: Height,
        cache_budget: usize,
        busiest: &str,
    ) -> Result<Self> {
        // The recordings are log events: a filter dropping them would leave the report empty.
        if !tracing::enabled!(target: "bitview::plugin_timing", Level::INFO) {
            return Err(Error::Internal(
                "LOG/RUST_LOG filters out the INFO events the benchmark records",
            ));
        }
        let reports = data_path.join("benches");
        fs::create_dir_all(&reports)?;
        // Every run keeps its own directory, named by its UTC start (`20261014T153012Z`, then
        // `-1`, `-2`... for restarts within the same second), so repetitions of one revision compare.
        let started_at = Timestamp::now();
        let revision = reports
            .join(metadata::machine(data_path))
            .join(env!("BENCH_REVISION"));
        fs::create_dir_all(&revision)?;
        let run_id = started_at.to_iso8601().replace(['-', ':'], "");
        let mut path = revision.join(&run_id);
        let mut suffix = 0;
        while let Err(error) = fs::create_dir(&path) {
            if error.kind() != ErrorKind::AlreadyExists {
                return Err(error.into());
            }
            suffix += 1;
            path = revision.join(format!("{run_id}-{suffix}"));
        }

        // Resumed when plugin data was already there: its time isn't a full sync's.
        let resumed = fs::read_dir(data_path.join("plugins"))
            .is_ok_and(|mut entries| entries.next().is_some());
        metadata::write(
            &path,
            data_path,
            blocks_path,
            *started_at,
            chain_height,
            cache_budget,
            resumed,
        )?;
        metadata::start_state(&path, busiest)?;

        let trace = Arc::new(TraceMonitor::new(&path)?);
        let trace_hook = Arc::clone(&trace);
        register_hook(move |event| trace_hook.record(event))
            .map_err(|error| IoError::new(ErrorKind::AlreadyExists, error))?;

        Ok(Self(Arc::new(Inner {
            disk: Arc::new(Mutex::new(DiskMonitor::new(
                data_path,
                &reports,
                &path.join("disk.csv"),
            )?)),
            trace,
            resumed,
            overlap: Arc::default(),
            path,
            stop: Arc::new(AtomicBool::new(false)),
            state: Mutex::new(State::Ready),
        })))
    }

    pub fn path(&self) -> &Path {
        &self.0.path
    }

    /// After the timed run: replays its last blocks serially, one plugin at a time, and records
    /// each plugin's CPU per block (see the README). A resumed run has no replay.
    pub fn replay(&self, plugins: &mut DefaultPlugins, context: UpdateContext<'_>) -> Result<()> {
        if self.0.resumed {
            return Ok(());
        }
        replay::run(self.path(), plugins, context)
    }

    pub fn measure<T>(&self, measure: impl FnOnce() -> Result<T>) -> Result<T> {
        self.start()?;
        let started_at = Instant::now();
        self.0.trace.start(started_at);
        let result = measure();
        let elapsed = started_at.elapsed();
        self.finish(
            if result.is_ok() {
                Outcome::Complete
            } else {
                Outcome::Failed
            },
            Some(elapsed),
        )?;
        result
    }

    pub fn abort(&self) -> Result<()> {
        self.finish(Outcome::Aborted, None)
    }

    fn start(&self) -> Result<()> {
        let mut state = self.0.state.lock();
        if !matches!(*state, State::Ready) {
            return Err(Error::Internal("Benchmark already started"));
        }

        self.0.disk.lock().record(0)?;
        let mut process = ProcessMonitor::new(process_id(), &self.0.path)?;
        process.record(0)?;
        let started_at = Instant::now();

        let disk = Arc::clone(&self.0.disk);
        let overlap = Arc::clone(&self.0.overlap);
        let mut watch = quiet::Watch::new()?;
        let monitors = [
            sampler(
                &self.0.stop,
                started_at,
                PROCESS_INTERVAL,
                move |elapsed, _| process.record(elapsed),
            ),
            // The final scan belongs to `finish`, with the breakdown.
            sampler(
                &self.0.stop,
                started_at,
                DISK_INTERVAL,
                move |elapsed, last| {
                    if last {
                        return Ok(());
                    }
                    // A list that can't be read can't vouch for the minute.
                    let seen = watch
                        .check()
                        .map(|names| names.join(", "))
                        .unwrap_or_else(|e| format!("a process list that failed ({e})"));
                    if !seen.is_empty() {
                        overlap.lock().get_or_insert(seen);
                    }
                    disk.lock().record(elapsed)
                },
            ),
        ];

        *state = State::Running {
            started_at,
            monitors,
        };
        Ok(())
    }

    fn finish(&self, outcome: Outcome, measured: Option<Duration>) -> Result<()> {
        let ended_at = Instant::now();
        // Held until the report is written: a shutdown's `abort` waits for it instead of exiting
        // mid-finalization.
        let mut state = self.0.state.lock();
        let State::Running {
            started_at,
            monitors,
        } = mem::replace(&mut *state, State::Finished)
        else {
            return Ok(());
        };

        self.0.stop.store(true, Ordering::Relaxed);
        for monitor in &monitors {
            monitor.thread().unpark();
        }
        let mut monitor_result = Ok(());
        for monitor in monitors {
            monitor_result = monitor_result.and(
                monitor
                    .join()
                    .unwrap_or(Err(Error::Internal("Benchmark monitor panicked"))),
            );
        }

        let elapsed = measured.unwrap_or_else(|| ended_at.duration_since(started_at));
        let trace_result = self.0.trace.finish();
        let disk_result = self
            .0
            .disk
            .lock()
            .finish(elapsed.as_millis(), &self.0.path.join("disk_breakdown.csv"));
        let recorded = monitor_result
            .and(trace_result.map_err(Error::from))
            .and_then(|timings| match (outcome, timings) {
                // A finished bootstrap logs plugin timings: none means they were lost.
                (Outcome::Complete, 0) => {
                    Err(Error::Internal("Benchmark recorded no plugin timings"))
                }
                _ => Ok(()),
            })
            .and(disk_result.map_err(Error::from));
        // The status comes last: a run that lost recordings is not complete.
        let outcome = match outcome {
            Outcome::Complete if recorded.is_err() => Outcome::Failed,
            outcome => outcome,
        };
        // Both files are written whatever the other's result.
        let comparable = metadata::comparable(&self.0.path, self.0.overlap.lock().as_deref());
        let run = fs::write(
            self.0.path.join("run.csv"),
            format!(
                "phase,start_ms,duration_ms,status\nbootstrap,0,{:.3},{}\n",
                elapsed.as_secs_f64() * 1_000.0,
                outcome.as_str()
            ),
        );
        run?;
        comparable?;
        recorded
    }
}

/// Records every `interval` until stopped, then once more with `last` set. A failed sample
/// keeps the first error for the end and sampling goes on, so one bad read loses one row.
fn sampler(
    stop: &Arc<AtomicBool>,
    started_at: Instant,
    interval: Duration,
    mut record: impl FnMut(u128, bool) -> std::io::Result<()> + Send + 'static,
) -> JoinHandle<Result<()>> {
    let stop = Arc::clone(stop);
    thread::spawn(move || {
        let mut first_error = None;
        let mut next_sample = started_at + interval;
        loop {
            while !stop.load(Ordering::Relaxed) {
                let now = Instant::now();
                if now >= next_sample {
                    break;
                }
                thread::park_timeout(next_sample - now);
            }
            let last = stop.load(Ordering::Relaxed);
            if let Err(error) = record(started_at.elapsed().as_millis(), last) {
                first_error.get_or_insert(error);
            }
            if last {
                break;
            }
            next_sample += interval;
        }
        first_error.map_or(Ok(()), |error| Err(error.into()))
    })
}
