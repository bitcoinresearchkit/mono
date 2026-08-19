use brk_error::Result;

use bitview_traversable::Traversable;
use brk_types::{Height, StoredU64, Version};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, Database, ImportableVec, PcoVec, Rw, Stamp, StorageMode, WritableVec};

use super::TransactionCounts;

macro_rules! define_counts {
    ($($(#[$attribute:meta])* $vector:ident: $flag:ident = $bit:literal $(, count: $count:ident $(, count_attr: $count_attr:meta)?)?;)+) => {
        #[derive(Traversable)]
        pub struct TransactionCountVecs<M: StorageMode = Rw> {
            /// Number of transactions in the block whose signed 32-bit Bitcoin
            /// transaction version is exactly 1, including coinbase.
            pub v1: M::Stored<PcoVec<Height, StoredU64>>,
            /// Number of transactions in the block whose signed 32-bit Bitcoin
            /// transaction version is exactly 2, including coinbase.
            pub v2: M::Stored<PcoVec<Height, StoredU64>>,
            /// Number of transactions in the block whose signed 32-bit Bitcoin
            /// transaction version is exactly 3, including coinbase.
            pub v3: M::Stored<PcoVec<Height, StoredU64>>,
            /// Number of transactions in the block whose signed 32-bit Bitcoin
            /// transaction version is not 1, 2, or 3, including coinbase. This
            /// category combines every other value; use individual raw
            /// transaction data to inspect the original version.
            pub other_version: M::Stored<PcoVec<Height, StoredU64>>,
            /// Number of transactions in the block with at least one input
            /// sequence number below `0xfffffffe`, the explicit opt-in RBF
            /// signal defined by BIP 125. This counts the mechanical sequence
            /// signal, not whether a transaction was replaceable or replaced,
            /// inherited signaling, or full-RBF policy. Coinbase transactions
            /// are evaluated by the same sequence rule.
            pub explicitly_rbf: M::Stored<PcoVec<Height, StoredU64>>,
            /// Number of transactions in the block with exactly one input,
            /// including the coinbase transaction.
            pub one_input: M::Stored<PcoVec<Height, StoredU64>>,
            /// Number of transactions in the block with exactly one output,
            /// including the coinbase transaction.
            pub one_output: M::Stored<PcoVec<Height, StoredU64>>,
            $($($(#[$count_attr])* pub $count: M::Stored<PcoVec<Height, StoredU64>>,)?) +
        }

        impl TransactionCountVecs {
            pub fn forced_import(db: &Database, version: Version) -> Result<Self> {
                let (
                    v1,
                    v2,
                    v3,
                    other_version,
                    explicitly_rbf,
                    one_input,
                    one_output,
                    $($($count,)?) +
                ) = parallel_import! {
                    v1 = PcoVec::forced_import(db, "tx_count_v1", version),
                    v2 = PcoVec::forced_import(db, "tx_count_v2", version),
                    v3 = PcoVec::forced_import(db, "tx_count_v3", version),
                    other_version = PcoVec::forced_import(db, "tx_count_other_version", version),
                    explicitly_rbf = PcoVec::forced_import(db, "tx_count_explicitly_rbf", version),
                    one_input = PcoVec::forced_import(db, "tx_count_one_input", version),
                    one_output = PcoVec::forced_import(db, "tx_count_one_output", version),
                    $($($count = PcoVec::forced_import(
                        db,
                        concat!("tx_count_", stringify!($count)),
                        version,
                    ),)?) +
                };
                Ok(Self {
                    v1,
                    v2,
                    v3,
                    other_version,
                    explicitly_rbf,
                    one_input,
                    one_output,
                    $($($count,)?) +
                })
            }

            pub fn push(&mut self, height: Height, counts: TransactionCounts) {
                self.v1.debug_checked_push(height, counts.v1.into());
                self.v2.debug_checked_push(height, counts.v2.into());
                self.v3.debug_checked_push(height, counts.v3.into());
                self.other_version
                    .debug_checked_push(height, counts.other_version.into());
                self.explicitly_rbf
                    .debug_checked_push(height, counts.explicitly_rbf.into());
                self.one_input.debug_checked_push(height, counts.one_input.into());
                self.one_output.debug_checked_push(height, counts.one_output.into());
                $($(self.$count.debug_checked_push(height, counts.$count.into());)?) +
            }

            pub fn truncate(&mut self, height: Height, stamp: Stamp) -> Result<()> {
                self.v1.truncate_if_needed_with_stamp(height, stamp)?;
                self.v2.truncate_if_needed_with_stamp(height, stamp)?;
                self.v3.truncate_if_needed_with_stamp(height, stamp)?;
                self.other_version
                    .truncate_if_needed_with_stamp(height, stamp)?;
                self.explicitly_rbf
                    .truncate_if_needed_with_stamp(height, stamp)?;
                self.one_input.truncate_if_needed_with_stamp(height, stamp)?;
                self.one_output.truncate_if_needed_with_stamp(height, stamp)?;
                $($(self.$count.truncate_if_needed_with_stamp(height, stamp)?;)?) +
                Ok(())
            }

            pub fn par_iter_mut_any(
                &mut self,
            ) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
                [
                    &mut self.v1 as &mut dyn AnyStoredVec,
                    &mut self.v2,
                    &mut self.v3,
                    &mut self.other_version,
                    &mut self.explicitly_rbf,
                    &mut self.one_input,
                    &mut self.one_output,
                    $($(&mut self.$count,)?) +
                ]
                .into_par_iter()
            }

            pub fn iter_any(&self) -> impl Iterator<Item = &dyn AnyStoredVec> {
                [
                    &self.v1 as &dyn AnyStoredVec,
                    &self.v2,
                    &self.v3,
                    &self.other_version,
                    &self.explicitly_rbf,
                    &self.one_input,
                    &self.one_output,
                    $($(&self.$count,)?) +
                ]
                .into_iter()
            }
        }
    };
}

with_transaction_features!(define_counts);
