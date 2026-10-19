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
use vecdb::{AnyStoredVec, Database, ImportableVec, PcoVec, Rw, StorageMode, WritableVec};

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
