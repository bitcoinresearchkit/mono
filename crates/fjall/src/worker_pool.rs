use crate::Keyspace;
use std::{sync::Mutex, thread::JoinHandle};

const MAX_WORKERS: usize = 4;

/// Work accepted by the background compaction pool.
pub enum WorkerMessage {
    /// Compact a keyspace until no eligible work remains.
    Compact(Keyspace),
    /// Stop one worker.
    Close,
}

/// Background compaction workers shared by all keyspaces.
pub struct WorkerPool {
    sender: flume::Sender<WorkerMessage>,
    receiver: flume::Receiver<WorkerMessage>,
    handles: Mutex<Vec<JoinHandle<()>>>,
}

impl WorkerPool {
    /// Creates and starts a worker pool.
    pub fn start() -> crate::Result<Self> {
        let worker_count = std::thread::available_parallelism()
            .map_or(1, usize::from)
            .min(MAX_WORKERS);
        let (sender, receiver) = flume::bounded(1_000);
        let pool = Self {
            sender,
            receiver,
            handles: Mutex::new(Vec::with_capacity(worker_count)),
        };

        let handles = (0..worker_count)
            .map(|worker_id| {
                let receiver = pool.receiver.clone();
                std::thread::Builder::new()
                    .name("fjall:compact".to_owned())
                    .spawn(move || Self::run(worker_id, &receiver))
            })
            .collect::<std::io::Result<Vec<_>>>()?;

        *pool.handles.lock().expect("worker lock is poisoned") = handles;
        Ok(pool)
    }

    /// Clones the work sender for a keyspace.
    pub fn sender(&self) -> flume::Sender<WorkerMessage> {
        self.sender.clone()
    }

    fn run(worker_id: usize, receiver: &flume::Receiver<WorkerMessage>) {
        while let Ok(message) = receiver.recv() {
            match message {
                WorkerMessage::Compact(keyspace) => {
                    if let Err(error) = keyspace.compact() {
                        log::error!(
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

        let mut handles = self.handles.lock().expect("worker lock is poisoned");
        for _ in 0..handles.len() {
            let _ = self.sender.send(WorkerMessage::Close);
        }
        for handle in handles.drain(..) {
            let _ = handle.join();
        }
    }
}
