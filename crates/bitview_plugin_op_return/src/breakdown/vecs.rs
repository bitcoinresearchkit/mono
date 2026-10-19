use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{Bytes, Count};
use bitview_traversable::Traversable;
use bitview_vecs::{LazyWindowStartVec, PerBlockCumulativeRolling, shapes::Windows};
use brk_error::Result;
use brk_types::{Height, Sats, VSize, Version};
use vecdb::{AnyStoredVec, Database, ReadableCloneableVec, Rw, StorageMode};

use super::{BlockMetrics, DataBytesSeries, FeesSeries};
use crate::{by_kind::ByKind, policy::Policy};

/// Metrics of a set of `OP_RETURN` outputs (all of them, or one protocol's or policy's) and
/// of the transactions creating them.
#[derive(Traversable)]
pub struct BucketVecs<M: StorageMode = Rw> {
    pub output_count: PerBlockCumulativeRolling<Count, M>,
    /// Script bytes following the `OP_RETURN` opcode, summed over the outputs.
    pub data_bytes: DataBytesSeries<M>,
    /// Number of transactions creating at least one of the outputs; each
    /// transaction counts once.
    pub tx_count: PerBlockCumulativeRolling<Count, M>,
    /// Full virtual sizes of the transactions creating the outputs, in vbytes;
    /// each transaction is included once.
    pub tx_vsize: PerBlockCumulativeRolling<VSize, M>,
    /// Full fees of the transactions creating the outputs; each transaction is
    /// included once.
    pub fees: FeesSeries<M>,
}

impl BucketVecs {
    pub(crate) fn import(
        db: &Database,
        prefix: &str,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        block_size: &impl ReadableCloneableVec<Height, Bytes>,
        chain_fees: &impl ReadableCloneableVec<Height, Sats>,
    ) -> Result<Self> {
        macro_rules! import {
            ($name:literal) => {
                PerBlockCumulativeRolling::import(
                    db,
                    &format!("{prefix}_{}", $name),
                    version,
                    mappings,
                    window_starts,
                )
            };
        }
        Ok(Self {
            output_count: import!("output_count")?,
            data_bytes: DataBytesSeries::new(
                prefix,
                version,
                import!("data_bytes")?,
                block_size,
                mappings,
            ),
            tx_count: import!("tx_count")?,
            tx_vsize: import!("tx_vsize")?,
            fees: FeesSeries::import(db, prefix, version, chain_fees, window_starts, mappings)?,
        })
    }

    pub(crate) fn push(&mut self, block: BlockMetrics) {
        self.output_count
            .push_block(Count::from(block.output_count));
        self.data_bytes.data_bytes.push_block(block.data_bytes);
        self.tx_count.push_block(Count::from(block.tx_count));
        self.tx_vsize.push_block(block.tx_vsize);
        self.fees.fees.push_sats(block.fees);
    }

    /// The stored columns every block pushes to; fiat columns compute apart.
    pub(crate) fn stored_mut(&mut self) -> [&mut dyn AnyStoredVec; 5] {
        [
            self.output_count.stored_mut(),
            self.data_bytes.data_bytes.stored_mut(),
            self.tx_count.stored_mut(),
            self.tx_vsize.stored_mut(),
            &mut self.fees.fees.cumulative.sats.height,
        ]
    }
}

macro_rules! impl_breakdown {
    ($name:ident, $group:ident) => {
        pub type $name<M = Rw> = $group<BucketVecs<M>>;

        impl $name {
            pub fn import(
                db: &Database,
                version: Version,
                mappings: &MappingsVecs,
                window_starts: &Windows<&LazyWindowStartVec>,
                block_size: &impl ReadableCloneableVec<Height, Bytes>,
                chain_fees: &impl ReadableCloneableVec<Height, Sats>,
            ) -> Result<Self> {
                let version = version + Version::ONE;
                $group::try_new(|name| {
                    BucketVecs::import(
                        db,
                        &format!("op_return_{name}"),
                        version,
                        mappings,
                        window_starts,
                        block_size,
                        chain_fees,
                    )
                })
            }

            /// One block's metrics per bucket, in field order.
            pub fn push(&mut self, values: impl IntoIterator<Item = BlockMetrics>) {
                for (bucket, value) in self.iter_mut().zip(values) {
                    bucket.push(value);
                }
            }
        }
    };
}
impl_breakdown!(KindBreakdownVecs, ByKind);
impl_breakdown!(PolicyBreakdownVecs, Policy);
