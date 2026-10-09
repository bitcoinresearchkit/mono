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
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use brk_error::{Error, Result};
use brk_logger::register_hook;
use brk_types::Height;
use disk::DiskMonitor;
use parking_lot::Mutex;
use process::ProcessMonitor;
use run::RunMonitor;
use trace::TraceMonitor;

mod disk;
mod metadata;
mod process;
mod run;
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
    run: Mutex<RunMonitor>,
    trace: Arc<TraceMonitor>,
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
    ) -> Result<Self> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .ok_or(Error::Internal("Invalid benchmark crate path"))?;
        let runs = data_path.join("benches").join("bitviewd");
        fs::create_dir_all(&runs)?;
        let path = runs.join(format!("run-{timestamp}"));
        fs::create_dir(&path)?;

        metadata::write(
            &path,
            workspace,
            data_path,
            blocks_path,
            chain_height,
            cache_budget,
        )?;

        let trace = Arc::new(TraceMonitor::new(&path)?);
        let trace_hook = Arc::clone(&trace);
        register_hook(move |event| trace_hook.record(event))
            .map_err(|error| IoError::new(ErrorKind::AlreadyExists, error))?;

        Ok(Self(Arc::new(Inner {
            disk: Arc::new(Mutex::new(DiskMonitor::new(
                data_path,
                &runs,
                &path.join("disk.csv"),
            )?)),
            run: Mutex::new(RunMonitor::new(&path.join("run.csv"))?),
            trace,
            path,
            stop: Arc::new(AtomicBool::new(false)),
            state: Mutex::new(State::Ready),
        })))
    }

    pub fn path(&self) -> &Path {
        &self.0.path
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
        self.0.stop.store(false, Ordering::Relaxed);

        let disk = Arc::clone(&self.0.disk);
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
                        Ok(())
                    } else {
                        disk.lock().record(elapsed)
                    }
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
        let running = {
            let mut state = self.0.state.lock();
            match mem::replace(&mut *state, State::Finished) {
                State::Running {
                    started_at,
                    monitors,
                } => Some((started_at, monitors)),
                State::Ready | State::Finished => None,
            }
        };
        let Some((started_at, monitors)) = running else {
            return Ok(());
        };

        self.0.stop.store(true, Ordering::Relaxed);
        for monitor in &monitors {
            monitor.thread().unpark();
        }
        let mut monitor_result = Ok(());
        for monitor in monitors {
            let result = monitor
                .join()
                .unwrap_or(Err(Error::Internal("Benchmark monitor panicked")));
            if monitor_result.is_ok() {
                monitor_result = result;
            }
        }

        let elapsed = measured.unwrap_or_else(|| ended_at.duration_since(started_at));
        let run_result = self.0.run.lock().record(elapsed, outcome.as_str());
        let trace_result = self.0.trace.finish();
        let disk_result = self
            .0
            .disk
            .lock()
            .finish(elapsed.as_millis(), &self.0.path.join("disk_breakdown.csv"));

        monitor_result?;
        run_result?;
        trace_result?;
        disk_result?;
        Ok(())
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
