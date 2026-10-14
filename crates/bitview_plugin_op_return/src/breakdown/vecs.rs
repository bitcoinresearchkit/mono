use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{Bytes, Count, OpReturnKind, OpReturnPolicyId};
use bitview_traversable::Traversable;
use bitview_vecs::{LazyWindowStartVec, PerBlockCumulativeRolling, shapes::Windows};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Sats, VSize, Version};
use vecdb::{AnyStoredVec, Database, ReadableCloneableVec, ReadableVec, Rw, StorageMode};

use super::{BlockMetrics, DataBytesSeries, FeesSeries};
use crate::{by_kind::ByKind, policy::Policy};

/// Metrics of a set of `OP_RETURN` outputs (all of them, or one protocol's or policy's) and
/// of the transactions creating them.
#[derive(Traversable)]
pub struct BucketVecs<M: StorageMode = Rw> {
    /// Number of `OP_RETURN` outputs.
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
        self.fees.push_block(block.fees);
    }

    /// The stored columns every block pushes to; fiat columns compute apart.
    pub(crate) fn stored_mut(&mut self) -> [&mut dyn AnyStoredVec; 5] {
        [
            self.output_count.stored_mut(),
            self.data_bytes.data_bytes.stored_mut(),
            self.tx_count.stored_mut(),
            self.tx_vsize.stored_mut(),
            self.fees.stored_mut(),
        ]
    }

    pub(crate) fn compute_cents(
        &mut self,
        max_from: Height,
        price_cents: &impl ReadableVec<Height, Cents>,
        exit: &Exit,
    ) -> Result<()> {
        self.fees.compute_cents(max_from, price_cents, exit)
    }
}

macro_rules! impl_breakdown {
    ($name:ident, $group:ident, $id:ident) => {
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
                $group::try_new(|_, name| {
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

            pub fn push(&mut self, values: [BlockMetrics; $id::ALL.len()]) {
                for (id, bucket) in $id::ALL.iter().zip(self.iter_mut()) {
                    bucket.push(*id.get(&values));
                }
            }

            pub fn compute_cents(
                &mut self,
                max_from: Height,
                price_cents: &impl ReadableVec<Height, Cents>,
                exit: &Exit,
            ) -> Result<()> {
                for bucket in self.iter_mut() {
                    bucket.compute_cents(max_from, price_cents, exit)?;
                }
                Ok(())
            }
        }
    };
}
impl_breakdown!(KindBreakdownVecs, ByKind, OpReturnKind);
impl_breakdown!(PolicyBreakdownVecs, Policy, OpReturnPolicyId);
