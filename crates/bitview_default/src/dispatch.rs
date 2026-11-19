use std::thread::{self, Scope, ScopedJoinHandle};

use crate::serial;

/// `rayon::join`, except in the serial CPU replay (see [`serial`]): there the branches run one
/// after the other, so a step's process CPU is its own. The parallel work inside each plugin is
/// unchanged.
pub fn join<A: Send, B: Send>(
    left: impl FnOnce() -> A + Send,
    right: impl FnOnce() -> B + Send,
) -> (A, B) {
    if serial::enabled() {
        (left(), right())
    } else {
        rayon::join(left, right)
    }
}

/// A branch of the composition: running on its own thread, or already complete.
pub enum Branch<'a, T> {
    Complete(T),
    Running(ScopedJoinHandle<'a, T>),
}

impl<T> Branch<'_, T> {
    pub fn join(self) -> thread::Result<T> {
        match self {
            Self::Complete(value) => Ok(value),
            Self::Running(handle) => handle.join(),
        }
    }
}

/// `scope.spawn`, except in the serial CPU replay: there the branch runs now, to completion.
pub fn spawn<'a, T: Send + 'a>(
    scope: &'a Scope<'a, '_>,
    run: impl FnOnce() -> T + Send + 'a,
) -> Branch<'a, T> {
    if serial::enabled() {
        Branch::Complete(run())
    } else {
        Branch::Running(scope.spawn(run))
    }
}
