use brk_types::{Cents, Timestamp};

use super::{
    compute::PriceRangeMax,
    state::{BlockState, LiveState},
};

/// Transient computation state, absent from the read-only distribution view.
#[derive(Default)]
pub struct Inner {
    pub live: Option<LiveState>,
    pub chain_state: Vec<BlockState>,
    pub prices: Vec<Cents>,
    pub timestamps: Vec<Timestamp>,
    pub price_range_max: PriceRangeMax,
}

impl Inner {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use bitview_cohort::EntryPrice;
    use bitview_traversable::Traversable;
    use brk_types::{Height, Version};
    use tempfile::tempdir;
    use vecdb::{
        AnyStoredVec, Database, EagerVec, ImportableVec, PcoVec, ReadOnlyClone, ReadableVec, Ro,
        Rw, StorageMode, WritableVec,
    };

    use super::*;

    #[derive(Traversable)]
    struct Projection<M: StorageMode = Rw> {
        #[traversable(skip)]
        mode: PhantomData<M>,
        #[traversable(skip)]
        db: Database,
        inner: M::WriteOnly<Inner>,
        values: M::Stored<EagerVec<PcoVec<Height, Cents>>>,
    }

    #[test]
    fn projection_does_not_copy_distribution_history() {
        let directory = tempdir().unwrap();
        let db = Database::open(directory.path()).unwrap();
        let mut inner = Inner {
            live: Some(LiveState::new(directory.path())),
            prices: vec![Cents::new(10), Cents::new(20)],
            timestamps: vec![Timestamp::default(); 2],
            ..Default::default()
        };
        inner.price_range_max.extend(&inner.prices);
        inner.chain_state.push(BlockState {
            supply: Default::default(),
            entry: EntryPrice::Discount,
            price: Cents::new(10),
            timestamp: Timestamp::default(),
        });
        let mut values = EagerVec::forced_import(&db, "projection", Version::ONE).unwrap();
        values.push(Cents::new(7));
        values.write().unwrap();
        let writer = Projection::<Rw> {
            mode: PhantomData,
            db,
            inner,
            values,
        };
        let reader: Projection<Ro> = writer.read_only_clone();

        assert!(writer.inner.live.is_some());
        assert_eq!(writer.inner.prices.len(), 2);
        assert_eq!(writer.inner.timestamps.len(), 2);
        assert_eq!(writer.inner.chain_state.len(), 1);
        assert_eq!(writer.inner.price_range_max.range_max(0, 1), Cents::new(20));
        let (): () = reader.inner;
        assert_eq!(size_of_val(&reader.inner), 0);
        assert_eq!(reader.values.collect(), [Cents::new(7)]);
        assert_eq!(reader.iter_any_visible().count(), 1);
        drop(writer);
        assert_eq!(reader.values.collect(), [Cents::new(7)]);
    }
}
