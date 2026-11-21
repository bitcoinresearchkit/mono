use std::sync::{Arc, Mutex};

use bitview_plugin::UpdateContext;
use brk_error::{Error, Result};
use rayon::Scope;

use crate::serial;

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
/// the serial CPU replay `own` runs, then the roots in order. Once a shutdown is pending, no root
/// starts: the drainers park, and the process exits when the running ones' sections end.
///
/// Inside a root, the same holds for every join: its pushed side can be taken by a worker waiting
/// in any other join and is run to its end first. So a root never pushes a stage that walks every
/// transaction, input or output, or a whole plugin: those run in place, one after another, and
/// only per-block or per-batch work is pushed.
pub fn spread<'s>(
    scope: &Scope<'s>,
    failed: &'s Failed,
    context: UpdateContext<'s>,
    roots: Vec<Root<'s>>,
    own: impl FnOnce() -> Result<()>,
) -> Result<()> {
    let queue = Arc::new(Mutex::new(roots.into_iter().rev().collect::<Vec<_>>()));
    if !serial::enabled() {
        let caller = rayon::current_thread_index();
        debug_assert!(caller.is_some(), "spread runs on a pool worker, or every copy drains");
        let shared = queue.clone();
        scope.spawn_broadcast(move |_, broadcast| {
            if Some(broadcast.index()) != caller {
                drain(&shared, failed, context);
            }
        });
    }
    let own = own();
    drain(&queue, failed, context);
    own
}

fn drain(queue: &Mutex<Vec<Root<'_>>>, failed: &Failed, context: UpdateContext<'_>) {
    loop {
        context.exit().park_if_shutting_down();
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
        self.arrive(|slots| slots.left = Some(value))
    }

    pub fn right(&self, value: B) -> Result<()> {
        self.arrive(|slots| slots.right = Some(value))
    }

    fn arrive(&self, hand_over: impl FnOnce(&mut Slots<A, B, F>)) -> Result<()> {
        let met = {
            let mut slots = self.slots.lock().unwrap();
            hand_over(&mut slots);
            slots.met()
        };
        met.map_or(Ok(()), |(left, right, then)| then(left, right))
    }
}
