use std::sync::Mutex;

use brk_error::{Error, Result};
use rayon::Scope;

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

/// The first error of the plugins nothing waits for, reported once the scope ends.
pub type Failed = Mutex<Option<Error>>;

/// `scope.spawn` of a plugin nothing waits for, keeping its error in `failed` unless an earlier
/// one is there; in the serial CPU replay it runs now, to completion.
pub fn spawn<'s>(
    scope: &Scope<'s>,
    failed: &'s Failed,
    run: impl FnOnce() -> Result<()> + Send + 's,
) {
    let run = move || {
        if let Err(error) = run() {
            failed.lock().unwrap().get_or_insert(error);
        }
    };
    if serial::enabled() {
        run();
    } else {
        scope.spawn(move |_| run());
    }
}
