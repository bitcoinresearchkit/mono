use std::sync::{Arc, Mutex};

use brk_error::{Error, Result};
use rayon::Scope;

use crate::serial;

/// `rayon::join`, except in the serial CPU replay (see [`serial`]): there the branches run one
/// after the other, so a step's process CPU is its own. The parallel work inside each plugin is
/// unchanged. The caller runs `left` and waits for `right`, which any worker may take, the
/// caller's own wait included: `right` is never the longer side.
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

/// The first error of the roots, reported once the scope ends.
pub type Failed = Mutex<Option<Error>>;

fn record(failed: &Failed, result: Result<()>) {
    if let Err(error) = result {
        failed.lock().unwrap().get_or_insert(error);
    }
}

/// A branch that holds its worker for long: a plugin's loop over the blocks, or a chain of them.
/// Boxed, as the roots are different closures queued together.
pub type Root<'s> = Box<dyn FnOnce() -> Result<()> + Send + 's>;

/// Runs `own` on the caller and the roots on the other workers, each root taken by the first
/// worker to come free, the caller too once `own` is done; errors from the roots go to `failed`.
/// The roots are broadcast jobs, which only their own worker runs, so no worker waiting in a join
/// can take one and hold its own work until the root ends, as it can a whole plugin pushed by a
/// join or a spawn; the caller's copy does nothing, or `own`'s waits would take roots. Call it with
/// the pool otherwise idle: a worker already waiting inside another job would run roots there. In
/// the serial CPU replay `own` runs, then the roots in order.
pub fn spread<'s>(
    scope: &Scope<'s>,
    failed: &'s Failed,
    roots: Vec<Root<'s>>,
    own: impl FnOnce() -> Result<()>,
) -> Result<()> {
    let queue = Arc::new(Mutex::new(roots.into_iter().rev().collect::<Vec<_>>()));
    if !serial::enabled() {
        let caller = rayon::current_thread_index();
        let shared = queue.clone();
        scope.spawn_broadcast(move |_, context| {
            if Some(context.index()) != caller {
                drain(&shared, failed);
            }
        });
    }
    let own = own();
    drain(&queue, failed);
    own
}

fn drain(queue: &Mutex<Vec<Root<'_>>>, failed: &Failed) {
    loop {
        let Some(root) = queue.lock().unwrap().pop() else {
            return;
        };
        record(failed, root());
    }
}

/// Where two branches meet: each hands its value over, and the second to arrive runs `then` with
/// both, in place. Neither waits for the other; one that fails never arrives, so `then` never runs.
pub struct Meet<A, B, F> {
    slots: Mutex<Slots<A, B, F>>,
}

struct Slots<A, B, F> {
    left: Option<A>,
    right: Option<B>,
    then: Option<F>,
}

impl<A, B, F> Slots<A, B, F> {
    fn met(&mut self) -> Option<(A, B, F)> {
        match (self.left.take(), self.right.take()) {
            (Some(left), Some(right)) => Some((left, right, self.then.take().expect("met twice"))),
            (left, right) => {
                (self.left, self.right) = (left, right);
                None
            }
        }
    }
}

impl<A, B, F: FnOnce(A, B) -> Result<()>> Meet<A, B, F> {
    pub fn new(then: F) -> Self {
        Self {
            slots: Mutex::new(Slots {
                left: None,
                right: None,
                then: Some(then),
            }),
        }
    }

    pub fn left(&self, value: A) -> Result<()> {
        let met = {
            let mut slots = self.slots.lock().unwrap();
            slots.left = Some(value);
            slots.met()
        };
        met.map_or(Ok(()), |(left, right, then)| then(left, right))
    }

    pub fn right(&self, value: B) -> Result<()> {
        let met = {
            let mut slots = self.slots.lock().unwrap();
            slots.right = Some(value);
            slots.met()
        };
        met.map_or(Ok(()), |(left, right, then)| then(left, right))
    }
}
