use bitview_primitives::{Difficulty, Hashrate};
use vecdb::UnaryTransform;

pub struct DifficultyToHashrate;

impl UnaryTransform<Difficulty, Hashrate> for DifficultyToHashrate {
    #[inline(always)]
    fn apply(difficulty: Difficulty) -> Hashrate {
        const MULTIPLIER: f64 = 4_294_967_296.0 / 600.0; // 2^32 / 600
        Hashrate::from(*difficulty * MULTIPLIER)
    }
}
