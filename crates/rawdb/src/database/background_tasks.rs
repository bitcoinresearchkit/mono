use std::{
    mem,
    thread::{self, JoinHandle},
    time::Duration,
};

use parking_lot::{Condvar, Mutex};

use crate::{Error, Result};

/// Owns worker handles and coordinates deferred waits with task joining.
#[derive(Default)]
pub(crate) struct BackgroundTasks {
    handles: Mutex<Vec<JoinHandle<Result<()>>>>,
    join: Mutex<()>,
    joining: Mutex<bool>,
    wake: Condvar,
}

impl BackgroundTasks {
    pub(crate) fn spawn(&self, task: impl FnOnce() -> Result<()> + Send + 'static) {
        self.handles.lock().push(thread::spawn(task));
    }

    pub(crate) fn sleep(&self, duration: Duration) {
        self.wake
            .wait_while_for(&mut self.joining.lock(), |joining| !*joining, duration);
    }

    pub(crate) fn join(&self) -> Result<()> {
        // An empty queue is enough only when no other join owns drained tasks.
        if self.handles.lock().is_empty() && self.join.try_lock().is_some() {
            return Ok(());
        }
        let _join = self.join.lock();
        *self.joining.lock() = true;
        self.wake.notify_all();

        let mut result = Ok(());
        loop {
            let handles = mem::take(&mut *self.handles.lock());
            if handles.is_empty() {
                break;
            }
            for handle in handles {
                // An owner acquired inside a callback can become the last owner
                // on that worker. Its storage remains alive until it returns.
                let joined = if handle.thread().id() == thread::current().id() {
                    Err(Error::BackgroundTaskSelfJoin)
                } else {
                    handle.join().unwrap_or(Err(Error::BackgroundTaskPanicked))
                };
                if result.is_ok() {
                    result = joined;
                }
            }
        }
        *self.joining.lock() = false;
        result
    }
}
