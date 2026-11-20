#[cfg(feature = "bench-cpu")]
use std::{io, sync::Mutex, time::Duration};
use std::thread::{self, Builder, JoinHandle};

#[cfg(feature = "bench-cpu")]
use crossbeam_channel::{RecvTimeoutError, bounded};
use crossbeam_channel::{Receiver, Sender, unbounded};
use log::error as LogError;

use crate::{Keyspace, Result};

const MAX_WORKERS: usize = 4;

/// Work accepted by the background compaction pool.
pub enum WorkerMessage {
    /// Compact a keyspace until no eligible work remains.
    Compact(Keyspace),
    /// Parks a worker after the preceding jobs, until every worker reaches the fence.
    #[cfg(feature = "bench-cpu")]
    Fence(Sender<()>, Receiver<()>),
    /// Stop one worker.
    Close,
}

/// Background compaction workers shared by all keyspaces.
///
/// The queue is unbounded, but each keyspace queues at most one request at a time.
pub struct WorkerPool {
    sender: Sender<WorkerMessage>,
    receiver: Receiver<WorkerMessage>,
    handles: Vec<JoinHandle<()>>,
    #[cfg(feature = "bench-cpu")]
    fence: Mutex<()>,
}

impl WorkerPool {
    /// Creates and starts a worker pool.
    pub fn start() -> Result<Self> {
        let worker_count = thread::available_parallelism()
            .map_or(1, usize::from)
            .min(MAX_WORKERS);
        let (sender, receiver) = unbounded();
        let mut pool = Self {
            sender,
            receiver,
            handles: Vec::with_capacity(worker_count),
            #[cfg(feature = "bench-cpu")]
            fence: Mutex::new(()),
        };

        // On a failed spawn, dropping the pool stops and joins the started workers.
        for worker_id in 0..worker_count {
            let receiver = pool.receiver.clone();
            pool.handles.push(
                Builder::new()
                    .name("fjall:compact".to_owned())
                    .spawn(move || Self::run(worker_id, &receiver))?,
            );
        }

        Ok(pool)
    }

    /// Clones the work sender for a keyspace.
    pub fn sender(&self) -> Sender<WorkerMessage> {
        self.sender.clone()
    }

    /// Returns once every compaction queued before it has finished: each worker takes one fence
    /// job and parks, so one still compacting an older job reaches it last. Waits as long as a
    /// compaction takes; errs only when a worker has died.
    #[cfg(feature = "bench-cpu")]
    pub(crate) fn wait_idle(&self) -> Result<()> {
        let _fence = self.fence.lock().expect("compaction fence lock is poisoned");
        let (arrived, arrivals) = bounded(self.handles.len());
        let (release, parked) = bounded(0);
        for _ in &self.handles {
            self.sender
                .send(WorkerMessage::Fence(arrived.clone(), parked.clone()))
                .map_err(|error| io::Error::other(format!("Queueing compaction fence: {error}")))?;
        }
        for _ in &self.handles {
            loop {
                match arrivals.recv_timeout(Duration::from_secs(1)) {
                    Ok(()) => break,
                    Err(RecvTimeoutError::Timeout) if self.handles.iter().all(|worker| !worker.is_finished()) => {}
                    Err(error) => {
                        return Err(io::Error::other(format!("Waiting for compaction workers: {error}")).into());
                    }
                }
            }
        }
        // Dropping `release` unparks every worker.
        drop(release);
        Ok(())
    }

    fn run(worker_id: usize, receiver: &Receiver<WorkerMessage>) {
        while let Ok(message) = receiver.recv() {
            match message {
                WorkerMessage::Compact(keyspace) => {
                    if let Err(error) = keyspace.compact_queued() {
                        LogError!(
                            "Compaction worker {worker_id} failed for {}: {error}",
                            keyspace.name(),
                        );
                    }
                }
                #[cfg(feature = "bench-cpu")]
                WorkerMessage::Fence(arrived, parked) => {
                    let _ = arrived.send(());
                    let _ = parked.recv();
                }
                WorkerMessage::Close => break,
            }
        }
    }
}

impl Drop for WorkerPool {
    fn drop(&mut self) {
        while self.receiver.try_recv().is_ok() {}

        for _ in 0..self.handles.len() {
            let _ = self.sender.send(WorkerMessage::Close);
        }
        for handle in self.handles.drain(..) {
            let _ = handle.join();
        }
    }
}
