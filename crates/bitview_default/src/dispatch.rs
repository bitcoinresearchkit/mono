use std::{mem, sync::Mutex};

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

/// The first error of the pinned roots, reported once the scope ends.
pub type Failed = Mutex<Option<Error>>;

fn record(failed: &Failed, result: Result<()>) {
    if let Err(error) = result {
        failed.lock().unwrap().get_or_insert(error);
    }
}

/// A branch that holds its worker for long: a plugin's loop over the blocks, or a chain of them.
/// Boxed, as the roots are different closures dealt to the workers at run time.
pub type Root<'s> = Box<dyn FnOnce() -> Result<()> + Send + 's>;

/// Runs each root on a worker of its own, other than the caller's, round robin over them when the
/// roots outnumber them, keeping errors in `failed`. A broadcast job runs only on its own worker,
/// so no worker waiting in a join can take a root and hold its own work until the root ends, as
/// it can a whole plugin pushed by a join or a spawn. In the serial CPU replay the roots run now,
/// in order.
pub fn pin<'s>(scope: &Scope<'s>, failed: &'s Failed, roots: Vec<Root<'s>>) {
    if serial::enabled() {
        roots.into_iter().for_each(|root| record(failed, root()));
        return;
    }
    let threads = rayon::current_num_threads();
    let caller = rayon::current_thread_index();
    let others = (0..threads)
        .filter(|&index| Some(index) != caller)
        .collect::<Vec<_>>();
    let mut queues = (0..threads).map(|_| Vec::new()).collect::<Vec<_>>();
    for (index, root) in roots.into_iter().enumerate() {
        // A pool of one has only the caller, which runs its roots once its own work waits.
        let worker = others.get(index % others.len().max(1)).copied().unwrap_or(0);
        queues[worker].push(root);
    }
    let queues = queues.into_iter().map(Mutex::new).collect::<Vec<_>>();
    scope.spawn_broadcast(move |_, context| {
        let roots = mem::take(&mut *queues[context.index()].lock().unwrap());
        roots.into_iter().for_each(|root| record(failed, root()));
    });
}

/// Where two branches meet: each hands its value over, and the second to arrive runs `then` with
/// both, in place. Neither waits for the other; one that fails never arrives, so `then` never runs.
pub struct Meet<A, B, F> {
    state: Mutex<(Option<A>, Option<B>, Option<F>)>,
}

impl<A, B, F: FnOnce(A, B) -> Result<()>> Meet<A, B, F> {
    pub fn new(then: F) -> Self {
        Self {
            state: Mutex::new((None, None, Some(then))),
        }
    }

    pub fn left(&self, value: A) -> Result<()> {
        self.arrive(|state| state.0 = Some(value))
    }

    pub fn right(&self, value: B) -> Result<()> {
        self.arrive(|state| state.1 = Some(value))
    }

    fn arrive(&self, hand_over: impl FnOnce(&mut (Option<A>, Option<B>, Option<F>))) -> Result<()> {
        let met = {
            let mut state = self.state.lock().unwrap();
            hand_over(&mut state);
            match &mut *state {
                (left @ Some(_), right @ Some(_), then) => {
                    Some((left.take(), right.take(), then.take()))
                }
                _ => None,
            }
        };
        match met {
            Some((Some(left), Some(right), Some(then))) => then(left, right),
            _ => Ok(()),
        }
    }
}
