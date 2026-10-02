use std::{sync::Arc, time::Duration};

use arc_swap::ArcSwap;
use bitview_plugin::Publication;
use brk_types::Lengths;
use parking_lot::RwLock;

use crate::SafeLengths;

/// One writer publishes complete bounds; only rollback drains pinned readers.
pub struct State {
    pub publication: Publication,
    lengths: ArcSwap<Lengths>,
    reorg: Arc<RwLock<()>>,
}

impl State {
    pub fn new() -> Self {
        Self {
            publication: Default::default(),
            lengths: ArcSwap::from_pointee(Lengths::default()),
            reorg: Arc::new(RwLock::new(())),
        }
    }

    pub fn lengths(&self) -> Lengths {
        **self.lengths.load()
    }

    pub fn pin_for(&self, timeout: Duration) -> Option<SafeLengths> {
        self.reorg
            .try_read_arc_for(timeout)
            .map(|guard| SafeLengths::new(guard, self.lengths()))
    }

    pub fn try_pin(&self) -> Option<SafeLengths> {
        self.reorg
            .try_read_arc()
            .map(|guard| SafeLengths::new(guard, self.lengths()))
    }

    pub fn pin_recursive(&self) -> SafeLengths {
        SafeLengths::new(self.reorg.read_arc_recursive(), self.lengths())
    }

    pub fn finish_update(&self, next: Lengths) {
        let current = self.lengths();
        if current == next {
            return;
        }
        debug_assert!(
            {
                let mut clamped = next;
                clamped.clamp_to(&current);
                clamped == current
            },
            "length regression"
        );
        self.lengths.store(Arc::new(next));
    }

    pub fn lower_before(&self, starting: &Lengths) {
        let current = self.lengths();
        let mut lowered = current;
        lowered.clamp_to(starting);
        if lowered == current {
            return;
        }
        // Acquire exclusion before publishing the smaller prefix. Once old
        // readers drain, new readers can use the unaffected prefix during undo.
        let guard = self.reorg.write();
        self.lengths.store(Arc::new(lowered));
        drop(guard);
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::mpsc, thread, time::Duration};

    use brk_types::{Height, TxIndex};

    use super::*;

    #[test]
    fn publication_advances_without_draining_or_changing_old_pins() {
        let state = Arc::new(State::new());
        let old = Lengths {
            height: Height::new(2),
            tx_index: TxIndex::from(3usize),
            ..Default::default()
        };
        let next = Lengths {
            height: Height::new(3),
            tx_index: TxIndex::from(5usize),
            ..old
        };
        state.finish_update(old);
        let pin = state.pin_for(Duration::from_secs(1)).unwrap();
        let writer = state.clone();
        let (finished, done) = mpsc::channel();
        let task = thread::spawn(move || {
            writer.finish_update(next);
            finished.send(()).unwrap();
        });
        let result = done.recv_timeout(Duration::from_secs(2));
        let retained = pin.lengths();
        let published = state.try_pin().map(|pin| pin.lengths());
        drop(pin);
        task.join().unwrap();
        result.expect("publication waited for an old pin");
        assert_eq!(retained, old);
        assert_eq!(published, Some(next));
    }

    #[test]
    fn pinned_prefix_blocks_rollback_and_allows_nested_bound_reads() {
        let state = Arc::new(State::new());
        state.finish_update(Lengths {
            height: Height::new(2),
            ..Default::default()
        });
        let prefix = state.pin_for(Duration::from_secs(1)).unwrap();
        let writer = state.clone();
        let (started, ready) = mpsc::channel();
        let (finished, done) = mpsc::channel();
        let task = thread::spawn(move || {
            started.send(()).unwrap();
            writer.lower_before(&Lengths::default());
            finished.send(()).unwrap();
        });
        ready.recv().unwrap();
        assert!(done.recv_timeout(Duration::from_millis(50)).is_err());
        assert!(
            state.try_pin().is_none(),
            "a queued writer has priority over new readers"
        );
        assert_eq!(prefix.lengths().height, Height::new(2));
        assert_eq!(state.lengths().height, Height::new(2));
        assert!(state.pin_for(Duration::from_millis(10)).is_none());
        drop(prefix);
        done.recv_timeout(Duration::from_secs(2)).unwrap();
        task.join().unwrap();
        assert_eq!(state.lengths().height, Height::ZERO);
    }

    #[test]
    fn pins_wait_for_writer_release_without_notifications() {
        let state = Arc::new(State::new());
        let guard = state.reorg.write();
        assert!(state.pin_for(Duration::from_millis(10)).is_none());
        let reader = state.clone();
        let (finished, done) = mpsc::channel();
        let task = thread::spawn(move || {
            finished
                .send(
                    reader
                        .pin_for(Duration::from_secs(2))
                        .map(|pin| pin.lengths()),
                )
                .unwrap();
        });
        let waiting = done.recv_timeout(Duration::from_millis(50));
        let next = Lengths {
            height: Height::new(2),
            ..Default::default()
        };
        state.lengths.store(Arc::new(next));
        drop(guard);
        let result = done.recv_timeout(Duration::from_secs(2));
        task.join().unwrap();
        assert!(waiting.is_err(), "reader passed a held write lock");
        assert_eq!(result.unwrap(), Some(next));
    }
}
