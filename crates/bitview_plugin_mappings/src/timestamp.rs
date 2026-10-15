use bitview_plugin_indexer::Indexer;
use bitview_primitives::{
    Day1, Day3, Epoch, Halving, Hour1, Hour4, Hour12, Minute10, Minute30, Month1, Month3, Month6,
    Week1, Year1, Year10,
};
use bitview_traversable::Traversable;
use bitview_vecs::{IndexSources, RangeMapVec, shapes::PerResolution, with_resolution_fields};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{BLOCKS_PER_DIFF_EPOCHS, BLOCKS_PER_HALVING, Height, Timestamp};
use derive_more::{Deref, DerefMut};
use vecdb::{
    Budgeted, ComputeVec, Database, ImportableVec, IndexVec, PcoVec, ReadableBoxedVec, ReadableVec,
    Rw, StorageMode, Version,
};

mod boundary;

pub use boundary::BoundaryTimestampVec;

macro_rules! define_timestamps {
    (
        periods { $($field:ident: $index:ident => $param:ident,)* }
        epochs { $($epoch:ident: $epoch_index:ident => $epoch_param:ident,)* }
    ) => {
        /// Timestamps: monotonic height→timestamp + per-period timestamp lookups.
        ///
        /// Time-based periods (minute10–year10) are lazy: `idx.to_timestamp()` is a pure
        /// function of the index, so no storage or decompression is needed.
        /// Block-based periods (halving, difficulty) are storage-free views of the raw
        /// timestamp at each period's first block.
        #[derive(Deref, DerefMut, Traversable)]
        pub struct Timestamps<M: StorageMode = Rw> {
            /// Nondecreasing Unix timestamp in seconds at each block height, computed as
            /// the maximum of the represented raw block-header timestamp and the preceding
            /// monotonic timestamp.
            pub monotonic: M::Stored<PcoVec<Height, Timestamp, Budgeted>>,
            /// Unix timestamp in seconds: at a height, the block's header time (not guaranteed to
            /// increase between heights); at a calendar period, its UTC start; at a halving or
            /// difficulty epoch, its first block's header time.
            #[deref]
            #[deref_mut]
            #[traversable(flatten)]
            pub period_start: PerResolution<
                $(IndexVec<$index, Timestamp, RangeMapVec<$index, Height>>,)*
                $(BoundaryTimestampVec<$epoch_index>,)*
            >,
        }

        impl Timestamps {
            pub fn new(
                version: Version,
                monotonic: PcoVec<Height, Timestamp, Budgeted>,
                raw_timestamps: ReadableBoxedVec<Height, Timestamp>,
                sources: &IndexSources,
            ) -> Self {
                Self {
                    monotonic,
                    period_start: PerResolution {
                        $($field: IndexVec::new(
                            "timestamp",
                            version,
                            sources.first_height.$field.clone(),
                            |idx| idx.to_timestamp(),
                        ),)*
                        halving: BoundaryTimestampVec::new(
                            raw_timestamps.clone(),
                            BLOCKS_PER_HALVING as usize,
                        ),
                        epoch: BoundaryTimestampVec::new(
                            raw_timestamps,
                            BLOCKS_PER_DIFF_EPOCHS as usize,
                        ),
                    },
                }
            }
        }
    };
}

with_resolution_fields!(define_timestamps);

impl Timestamps {
    pub fn import_monotonic(
        db: &Database,
        version: Version,
    ) -> Result<PcoVec<Height, Timestamp, Budgeted>> {
        Ok(PcoVec::import(db, "timestamp_monotonic", version)?)
    }

    pub fn compute_monotonic(
        &mut self,
        indexer: &Indexer,
        starting_height: Height,
        exit: &Exit,
    ) -> Result<()> {
        let mut prev = None;
        self.monotonic.compute_transform(
            starting_height,
            &indexer.vecs().blocks.timestamp,
            |(h, timestamp, this)| {
                if prev.is_none()
                    && let Some(prev_h) = h.decremented()
                {
                    prev.replace(this.collect_one(prev_h).unwrap());
                }
                let monotonic = prev.map_or(timestamp, |p| p.max(timestamp));
                prev.replace(monotonic);
                (h, monotonic)
            },
            exit,
        )?;
        Ok(())
    }
}
