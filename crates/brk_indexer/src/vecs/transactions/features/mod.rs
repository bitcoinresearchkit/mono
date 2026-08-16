mod counts;
mod flags;
mod schema;
mod transaction_counts;

pub use counts::TransactionCountVecs;
pub(crate) use flags::TxFeatureFlags;
pub(crate) use transaction_counts::TransactionCounts;

use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{Height, StoredBool, TxIndex, Version};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, Database, ImportableVec, PcoVec, Rw, Stamp, StorageMode, WritableVec};

use self::schema::with_transaction_features;
use crate::parallel_import;

macro_rules! define_vecs {
    ($($(#[$attribute:meta])* $vector:ident: $flag:ident = $bit:literal $(, count: $count:ident $(, count_attr: $count_attr:meta)?)?;)+) => {
        #[derive(Traversable)]
        pub struct TransactionFeaturesVecs<M: StorageMode = Rw> {
            pub count: TransactionCountVecs<M>,
            $($(#[$attribute])* pub $vector: M::Stored<PcoVec<TxIndex, StoredBool>>,) +
        }

        impl TransactionFeaturesVecs {
            pub fn forced_import(db: &Database, version: Version) -> Result<Self> {
                let (count, $($vector,) +) = parallel_import! {
                    count = TransactionCountVecs::forced_import(db, version),
                    $($vector = PcoVec::forced_import(db, stringify!($vector), version),) +
                };
                Ok(Self { count, $($vector,) + })
            }

            pub(crate) fn push_and_count(
                &mut self,
                flags: TxFeatureFlags,
                counts: &mut TransactionCounts,
            ) {
                $(
                    let is_set = flags.is_set(TxFeatureFlags::$flag);
                    self.$vector.push(StoredBool::from(is_set));
                    $(counts.$count += is_set as u64;)?
                ) +
            }

            pub fn truncate(
                &mut self,
                height: Height,
                tx_index: TxIndex,
                stamp: Stamp,
            ) -> Result<()> {
                self.count.truncate(height, stamp)?;
                $(self.$vector.truncate_if_needed_with_stamp(tx_index, stamp)?;) +
                Ok(())
            }

            pub fn par_iter_mut_any(
                &mut self,
            ) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
                [$( &mut self.$vector as &mut dyn AnyStoredVec, ) +]
                    .into_par_iter()
                    .chain(self.count.par_iter_mut_any())
            }

            pub fn iter_any(&self) -> impl Iterator<Item = &dyn AnyStoredVec> {
                [$( &self.$vector as &dyn AnyStoredVec, ) +]
                    .into_iter()
                    .chain(self.count.iter_any())
            }
        }
    };
}

with_transaction_features!(define_vecs);
