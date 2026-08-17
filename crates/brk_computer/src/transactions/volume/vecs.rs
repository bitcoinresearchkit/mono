use brk_traversable::Traversable;
use vecdb::{Rw, StorageMode};

use crate::internal::{LazyPerSecondWindows, ValuePerBlockCumulativeRolling};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Sum of the input values of non-coinbase transactions. This equals their
    /// total output value plus transaction fees and is not adjusted to estimate
    /// economic payment volume.
    pub transfer_volume: ValuePerBlockCumulativeRolling<M>,
    /// Transaction rate, including coinbase transactions.
    pub tx_per_sec: LazyPerSecondWindows,
}

#[cfg(test)]
mod tests {
    use brk_types::StoredU64;
    use vecdb::UnaryTransform;

    use crate::internal::PerSecond;

    #[test]
    fn transactions_per_second_uses_the_full_fixed_window() {
        assert_eq!(
            f32::from(PerSecond::<86_400>::apply(StoredU64::from(86_400_u64))),
            1.0
        );
        assert_eq!(
            f32::from(PerSecond::<2_592_000>::apply(StoredU64::from(
                1_296_000_u64
            ))),
            0.5
        );
    }
}
