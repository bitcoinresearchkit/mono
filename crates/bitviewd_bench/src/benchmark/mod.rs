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
use brk_types::Height;
use parking_lot::Mutex;

mod disk;
mod metadata;
mod process;
mod run;
mod trace;

use disk::DiskMonitor;
use process::ProcessMonitor;
use run::RunMonitor;
use trace::TraceMonitor;

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

struct Inner {
    path: PathBuf,
    disk: Mutex<DiskMonitor>,
    run: Mutex<RunMonitor>,
    trace: Arc<TraceMonitor>,
    stop: Arc<AtomicBool>,
    state: Mutex<State>,
}

enum State {
    Ready,
    Running {
        started_at: Instant,
        monitor: JoinHandle<Result<()>>,
    },
    Finished,
}

impl Benchmark {
    pub fn new(data_path: &Path, blocks_path: &Path, chain_height: Height) -> Result<Self> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .ok_or(Error::Internal("Invalid benchmark crate path"))?;
        let runs = workspace.join("benches").join("bitviewd");
        fs::create_dir_all(&runs)?;
        let path = runs.join(format!("run-{timestamp}"));
        fs::create_dir(&path)?;

        metadata::write(&path, workspace, data_path, blocks_path, chain_height)?;

        let trace = Arc::new(TraceMonitor::new(&path)?);
        let trace_hook = Arc::clone(&trace);
        brk_logger::register_hook(move |event| trace_hook.record(event))
            .map_err(|error| IoError::new(ErrorKind::AlreadyExists, error))?;

        Ok(Self(Arc::new(Inner {
            disk: Mutex::new(DiskMonitor::new(data_path, &path.join("disk.csv"))?),
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

        let stop = Arc::clone(&self.0.stop);
        let monitor = thread::spawn(move || -> Result<()> {
            let mut next_sample = started_at + Duration::from_secs(5);

            loop {
                while !stop.load(Ordering::Relaxed) {
                    let now = Instant::now();
                    if now >= next_sample {
                        break;
                    }
                    thread::park_timeout(next_sample - now);
                }

                if stop.load(Ordering::Relaxed) {
                    break;
                }

                process.record(started_at.elapsed().as_millis())?;
                next_sample += Duration::from_secs(5);
            }

            process.record(started_at.elapsed().as_millis())?;
            process.flush()?;
            Ok(())
        });

        *state = State::Running {
            started_at,
            monitor,
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
                    monitor,
                } => Some((started_at, monitor)),
                State::Ready | State::Finished => None,
            }
        };
        let Some((started_at, monitor)) = running else {
            return Ok(());
        };

        self.0.stop.store(true, Ordering::Relaxed);
        monitor.thread().unpark();
        let process_result = match monitor.join() {
            Ok(result) => result,
            Err(_) => Err(Error::Internal("Process monitor panicked")),
        };

        let elapsed = measured.unwrap_or_else(|| ended_at.duration_since(started_at));
        let run_result = self.0.run.lock().record(elapsed, outcome.as_str());
        let trace_result = self.0.trace.finish();
        let disk_result = self.0.disk.lock().record(elapsed.as_millis());

        process_result?;
        run_result?;
        trace_result?;
        disk_result?;
        Ok(())
    }
}
