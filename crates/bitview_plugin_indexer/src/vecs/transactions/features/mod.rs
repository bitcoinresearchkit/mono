include!("schema.rs");

pub mod feature;
pub mod flags;
pub mod transaction_counts;

pub(crate) use feature::block_count;
pub use feature::{BlockCount, CountVecs, FeatureVecs, FlagView};
pub use flags::TxFeatureFlags;
pub use transaction_counts::TransactionCounts;

use bitview_primitives::Boolean;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Height, TxIndex, Version};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, Database, ImportableVec, PcoVec, Rw, Stamp, StorageMode, WritableVec};

macro_rules! define_vecs {
    (
        features { $($(#[$doc:meta])* $feature:ident: $vector:ident, $flag:ident = $bit:literal;)+ }
        flags { $($(#[$flag_attribute:meta])* $flag_vector:ident: $flag_only:ident = $flag_bit:literal;)+ }
    ) => {
        #[derive(Traversable)]
        pub struct TransactionFeaturesVecs<M: StorageMode = Rw> {
            $($(#[$doc])* pub $feature: FeatureVecs<M>,)+
            $($(#[$flag_attribute])* pub $flag_vector: M::Stored<PcoVec<TxIndex, Boolean>>,)+
            /// Transactions with exactly one input, including the coinbase transaction.
            pub one_input: CountVecs<M>,
            /// Transactions with exactly one output, including the coinbase transaction.
            pub one_output: CountVecs<M>,
        }

        impl TransactionFeaturesVecs {
            pub fn import(db: &Database, version: Version) -> Result<Self> {
                let ($($feature,)+ $($flag_vector,)+ one_input, one_output) = parallel_import! {
                    $($feature = FeatureVecs::import(
                        db,
                        stringify!($vector),
                        concat!(stringify!($feature), "_tx_count"),
                        version,
                    ),)+
                    $($flag_vector = PcoVec::import(db, stringify!($flag_vector), version),)+
                    one_input = CountVecs::import(db, "one_input_tx_count", version),
                    one_output = CountVecs::import(db, "one_output_tx_count", version),
                };
                Ok(Self { $($feature,)+ $($flag_vector,)+ one_input, one_output })
            }

            pub fn push_and_count(
                &mut self,
                tx_index: TxIndex,
                flags: TxFeatureFlags,
                counts: &mut TransactionCounts,
            ) {
                $(
                    let is_set = flags.is_set(TxFeatureFlags::$flag);
                    self.$feature.flag.debug_checked_push(tx_index, Boolean::from(is_set));
                    counts.$feature += u16::from(is_set);
                )+
                $(self.$flag_vector.debug_checked_push(
                    tx_index,
                    Boolean::from(flags.is_set(TxFeatureFlags::$flag_only)),
                );)+
            }

            pub fn push_counts(&mut self, height: Height, counts: &TransactionCounts) {
                $(self.$feature.block.debug_checked_push(height, counts.$feature.into());)+
                self.one_input.block.debug_checked_push(height, counts.one_input.into());
                self.one_output.block.debug_checked_push(height, counts.one_output.into());
            }

            pub fn truncate(
                &mut self,
                height: Height,
                tx_index: TxIndex,
                stamp: Stamp,
            ) -> Result<()> {
                $(
                    self.$feature.flag.truncate_if_needed_with_stamp(tx_index, stamp)?;
                    self.$feature.block.truncate_if_needed_with_stamp(height, stamp)?;
                )+
                $(self.$flag_vector.truncate_if_needed_with_stamp(tx_index, stamp)?;)+
                self.one_input.block.truncate_if_needed_with_stamp(height, stamp)?;
                self.one_output.block.truncate_if_needed_with_stamp(height, stamp)?;
                Ok(())
            }

            pub fn par_iter_mut_any(
                &mut self,
            ) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
                let features = [$(&mut self.$feature,)+].into_par_iter().flat_map_iter(|feature| {
                    [&mut feature.flag as &mut dyn AnyStoredVec, &mut *feature.block]
                });
                [
                    $(&mut self.$flag_vector as &mut dyn AnyStoredVec,)+
                    &mut *self.one_input.block,
                    &mut *self.one_output.block,
                ]
                .into_par_iter()
                .chain(features)
            }

            pub fn iter_any(&self) -> impl Iterator<Item = &dyn AnyStoredVec> {
                [$(&self.$feature,)+]
                    .into_iter()
                    .flat_map(|feature| [&feature.flag as &dyn AnyStoredVec, &*feature.block])
                    .chain([
                        $(&self.$flag_vector as &dyn AnyStoredVec,)+
                        &*self.one_input.block,
                        &*self.one_output.block,
                    ])
            }
        }
    };
}

with_transaction_features!(define_vecs);
