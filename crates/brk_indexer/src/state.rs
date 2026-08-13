use std::sync::Arc;

use parking_lot::RwLock;

use crate::lengths::{IndexerLengths as _, Lengths};

#[derive(Clone)]
pub struct State(Arc<RwLock<StateInner>>);

struct StateInner {
    lengths: Lengths,
    generation: Generation,
}

#[derive(Clone, Copy)]
enum Generation {
    Running(usize),
    Done(usize),
}

impl State {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(StateInner {
            lengths: Lengths::default(),
            generation: Generation::Running(0),
        })))
    }

    pub fn lengths(&self) -> Lengths {
        self.0.read().lengths
    }

    pub fn generation(&self) -> Option<usize> {
        match self.0.read().generation {
            Generation::Running(_) => None,
            Generation::Done(generation) => Some(generation),
        }
    }

    pub fn begin_update(&self) {
        let mut inner = self.0.write();
        inner.generation = match inner.generation {
            Generation::Done(generation) => Generation::Running(generation.wrapping_add(1)),
            Generation::Running(generation) => Generation::Running(generation),
        };
    }

    pub fn finish_update(&self, next: Lengths) {
        let mut inner = self.0.write();
        debug_assert!(
            {
                let mut clamped = next;
                clamped.clamp_to(&inner.lengths);
                clamped == inner.lengths
            },
            "length regression"
        );
        let generation = match inner.generation {
            Generation::Running(generation) => generation,
            Generation::Done(_) => panic!("state update is not running"),
        };
        inner.lengths = next;
        inner.generation = Generation::Done(generation);
    }

    pub fn lower_before(&self, starting: &Lengths) {
        self.0.write().lengths.clamp_to(starting);
    }
}

#[cfg(test)]
mod tests {
    use brk_types::Height;

    use super::*;

    #[test]
    fn lifecycle_publishes_lengths_and_generation_together() {
        let state = State::new();
        assert_eq!(state.generation(), None);

        state.finish_update(Lengths::default());
        assert_eq!(state.generation(), Some(0));

        state.begin_update();
        state.begin_update();
        assert_eq!(state.generation(), None);

        let lengths = Lengths {
            height: Height::new(1),
            ..Default::default()
        };
        state.finish_update(lengths);
        assert_eq!(state.lengths(), lengths);
        assert_eq!(state.generation(), Some(1));
    }

    #[test]
    fn lower_before_clamps_published_lengths() {
        let state = State::new();
        state.finish_update(Lengths {
            height: Height::new(1),
            ..Default::default()
        });
        state.begin_update();
        state.lower_before(&Lengths::default());

        assert_eq!(state.lengths(), Lengths::default());
    }
}
