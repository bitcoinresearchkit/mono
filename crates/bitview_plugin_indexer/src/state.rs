use std::sync::Arc;

use brk_types::Lengths;
use parking_lot::RwLock;

#[derive(Clone)]
pub struct State(Arc<RwLock<Lengths>>);

impl State {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(Lengths::default())))
    }

    pub fn lengths(&self) -> Lengths {
        *self.0.read()
    }

    pub fn finish_update(&self, next: Lengths) {
        let mut lengths = self.0.write();
        debug_assert!(
            {
                let mut clamped = next;
                clamped.clamp_to(&lengths);
                clamped == *lengths
            },
            "length regression"
        );
        *lengths = next;
    }

    pub fn lower_before(&self, starting: &Lengths) {
        self.0.write().clamp_to(starting);
    }
}

#[cfg(test)]
mod tests {
    use brk_types::Height;

    use super::*;

    #[test]
    fn lifecycle_publishes_lengths() {
        let state = State::new();
        state.finish_update(Lengths::default());

        let lengths = Lengths {
            height: Height::new(1),
            ..Default::default()
        };
        state.finish_update(lengths);
        assert_eq!(state.lengths(), lengths);
    }

    #[test]
    fn lower_before_clamps_published_lengths() {
        let state = State::new();
        state.finish_update(Lengths {
            height: Height::new(1),
            ..Default::default()
        });
        state.lower_before(&Lengths::default());

        assert_eq!(state.lengths(), Lengths::default());
    }
}
