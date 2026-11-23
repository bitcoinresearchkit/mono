use std::{
    fs::{self, OpenOptions},
    io::{Error as IoError, ErrorKind, Write},
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

use bitview::{ImportContext, UpdateContext, bootstrap};
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
mod serial_cpu;
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
    /// The first Bitview or build seen beside the run: it is not comparable.
    overlap: Arc<Mutex<Option<String>>>,
    /// The data directory already held plugin data: not a full sync, and no recompute.
    resumed: bool,
    stop: Arc<AtomicBool>,
    state: Mutex<State>,
    /// What `close` ends, once.
    tail: Mutex<Option<Tail>>,
}

/// The quiet watch, which outlives the timed run's monitors to cover the serial recompute, and
/// when the recompute began, if it did.
struct Tail {
    stop: Arc<AtomicBool>,
    thread: JoinHandle<Result<()>>,
    started_at: Instant,
    recompute: Option<Instant>,
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
            overlap: Arc::default(),
            path,
            resumed,
            stop: Arc::new(AtomicBool::new(false)),
            state: Mutex::new(State::Ready),
            tail: Mutex::new(None),
        })))
    }

    pub fn path(&self) -> &Path {
        &self.0.path
    }

    /// Times the bootstrap of the plugins `import` makes, then, from an empty data directory,
    /// bootstraps them again with every plugin but the indexer from zero, one at a time,
    /// recording each one's CPU (see the README). The quiet watch covers both.
    pub fn measure(
        &self,
        import_context: ImportContext<'_>,
        update_context: UpdateContext<'_>,
        import: impl Fn(ImportContext<'_>) -> Result<DefaultPlugins>,
    ) -> Result<()> {
        let sync = || bootstrap(import_context, &import, update_context);
        self.start()?;
        let started_at = Instant::now();
        self.0.trace.start(started_at);
        let result = sync();
        let elapsed = started_at.elapsed();
        let outcome = if result.is_ok() { Outcome::Complete } else { Outcome::Failed };
        let finished = self.finish(outcome, Some(elapsed));
        let measured = match (result, finished) {
            (Ok(plugins), Ok(())) if !self.0.resumed => {
                if let Some(tail) = self.0.tail.lock().as_mut() {
                    tail.recompute = Some(Instant::now());
                }
                serial_cpu::recompute(self.path(), plugins, import_context, sync)
            }
            (result, finished) => result.map(drop).and(finished),
        };
        let closed = self.close(if measured.is_ok() {
            Outcome::Complete
        } else {
            Outcome::Failed
        });
        measured.and(closed)
    }

    pub fn abort(&self) -> Result<()> {
        let finished = self.finish(Outcome::Aborted, None);
        let closed = self.close(Outcome::Aborted);
        finished.and(closed)
    }

    /// Stops the quiet watch, adds the recompute's row to `run.csv` with `recompute` as its status
    /// if it began, and writes whether anything overlapped the run; once. The lock is held to the
    /// end, so a shutdown's `abort` waits for these lines instead of exiting before them.
    fn close(&self, recompute: Outcome) -> Result<()> {
        let mut tail = self.0.tail.lock();
        let Some(Tail { stop, thread, started_at, recompute: began }) = tail.take() else {
            return Ok(());
        };
        stop.store(true, Ordering::Relaxed);
        thread.thread().unpark();
        let watched = thread
            .join()
            .unwrap_or(Err(Error::Internal("Benchmark watch panicked")));
        let row = began.map_or(Ok(()), |began| {
            let mut run = OpenOptions::new().append(true).open(self.0.path.join("run.csv"))?;
            writeln!(
                run,
                "serial_compute,{:.3},{:.3},{}",
                began.duration_since(started_at).as_secs_f64() * 1_000.0,
                began.elapsed().as_secs_f64() * 1_000.0,
                recompute.as_str()
            )
        });
        let comparable = metadata::comparable(&self.0.path, self.0.overlap.lock().as_deref());
        drop(tail);
        row?;
        comparable?;
        watched
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
        let stop = Arc::new(AtomicBool::new(false));
        let thread = sampler(&stop, started_at, DISK_INTERVAL, move |_, last| {
            if !last {
                // A list that can't be read can't vouch for the minute.
                let seen = watch
                    .check()
                    .map(|names| names.join(", "))
                    .unwrap_or_else(|e| format!("a process list that failed ({e})"));
                if !seen.is_empty() {
                    overlap.lock().get_or_insert(seen);
                }
            }
            Ok(())
        });
        *self.0.tail.lock() = Some(Tail {
            stop,
            thread,
            started_at,
            recompute: None,
        });
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
        let run = fs::write(
            self.0.path.join("run.csv"),
            format!(
                "phase,start_ms,duration_ms,status\nbootstrap,0,{:.3},{}\n",
                elapsed.as_secs_f64() * 1_000.0,
                outcome.as_str()
            ),
        );
        run?;
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
