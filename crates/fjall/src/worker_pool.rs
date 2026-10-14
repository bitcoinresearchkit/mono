use std::thread::{self, Builder, JoinHandle};

use flume::{Receiver, Sender, unbounded};
use log::error as LogError;

use crate::{Keyspace, Result};

const MAX_WORKERS: usize = 4;

/// Work accepted by the background compaction pool.
pub enum WorkerMessage {
    /// Compact a keyspace until no eligible work remains.
    Compact(Keyspace),
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
