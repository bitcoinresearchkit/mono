use bitview_primitives::StoredF64;
use vecdb::UnaryTransform;

pub struct DifficultyToHashF64;

impl UnaryTransform<StoredF64, StoredF64> for DifficultyToHashF64 {
    #[inline(always)]
    fn apply(difficulty: StoredF64) -> StoredF64 {
        const MULTIPLIER: f64 = 4_294_967_296.0 / 600.0; // 2^32 / 600
        StoredF64::from(*difficulty * MULTIPLIER)
    }
}
