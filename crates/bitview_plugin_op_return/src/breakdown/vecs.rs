use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{Bytes, Count, OpReturnKind, OpReturnPolicyId};
use bitview_traversable::Traversable;
use bitview_vecs::{LazyWindowStartVec, PerBlockCumulativeRolling};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Sats, VSize, Version};
use vecdb::{AnyStoredVec, AnyVec, Database, ReadableCloneableVec, ReadableVec, Rw, StorageMode};

use super::{BlockMetrics, DataBytesSeries, FeesSeries};
use crate::{by_kind::ByKind, policy::Policy};

/// Metrics of one protocol or policy: its matching `OP_RETURN` outputs and
/// transactions.
#[derive(Traversable)]
pub struct BucketVecs<M: StorageMode = Rw> {
    /// Number of matching `OP_RETURN` outputs.
    pub output_count: PerBlockCumulativeRolling<Count, M>,
    /// Script bytes following `OP_RETURN` in matching outputs.
    pub data_bytes: DataBytesSeries<M>,
    /// Number of matching transactions; each transaction counts once.
    pub tx_count: PerBlockCumulativeRolling<Count, M>,
    /// Full virtual sizes of matching transactions.
    pub tx_vsize: PerBlockCumulativeRolling<VSize, M>,
    /// Full fees of matching transactions.
    pub fees: FeesSeries<M>,
}

impl BucketVecs {
    #[allow(clippy::too_many_arguments)]
    fn import(
        db: &Database,
        prefix: &str,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        total_data: &impl ReadableCloneableVec<Height, Bytes>,
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
                total_data,
                block_size,
                mappings,
            ),
            tx_count: import!("tx_count")?,
            tx_vsize: import!("tx_vsize")?,
            fees: FeesSeries::import(db, prefix, version, chain_fees, window_starts, mappings)?,
        })
    }

    fn len(&self) -> usize {
        self.output_count
            .cumulative
            .height
            .len()
            .min(self.data_bytes.cumulative.height.len())
            .min(self.tx_count.cumulative.height.len())
            .min(self.tx_vsize.cumulative.height.len())
            .min(self.fees.len())
    }

    fn push(&mut self, block: BlockMetrics) {
        self.output_count
            .push_block(Count::from(block.output_count));
        self.data_bytes.data_bytes.push_block(block.data_bytes);
        self.tx_count.push_block(Count::from(block.tx_count));
        self.tx_vsize.push_block(block.tx_vsize);
        self.fees.push_block(block.fees);
    }

    fn stored_mut(&mut self) -> [&mut dyn AnyStoredVec; 5] {
        [
            self.output_count.stored_mut(),
            self.data_bytes.data_bytes.stored_mut(),
            self.tx_count.stored_mut(),
            self.tx_vsize.stored_mut(),
            self.fees.stored_mut(),
        ]
    }
}

macro_rules! impl_breakdown {
    ($name:ident, $group:ident, $id:ident) => {
        pub type $name<M = Rw> = $group<BucketVecs<M>>;

        impl $name {
            #[allow(clippy::too_many_arguments)]
            pub fn import(
                db: &Database,
                version: Version,
                mappings: &MappingsVecs,
                window_starts: &Windows<&LazyWindowStartVec>,
                total_data: &impl ReadableCloneableVec<Height, Bytes>,
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
                        total_data,
                        block_size,
                        chain_fees,
                    )
                })
            }

            pub fn len(&self) -> usize {
                self.iter().map(BucketVecs::len).min().unwrap_or_default()
            }

            pub fn push(&mut self, values: [BlockMetrics; $id::ALL.len()]) {
                for (id, bucket) in $id::ALL.iter().zip(self.iter_mut()) {
                    bucket.push(*id.get(&values));
                }
            }

            pub fn validate_and_truncate(
                &mut self,
                version: Version,
                height: Height,
            ) -> Result<()> {
                for target in self.iter_mut().flat_map(BucketVecs::stored_mut) {
                    target.any_validate_computed_version_or_reset(version)?;
                    target.any_truncate_if_needed_at(usize::from(height))?;
                }
                Ok(())
            }

            pub fn truncate_if_needed_at(&mut self, len: usize) -> Result<()> {
                for target in self.iter_mut().flat_map(BucketVecs::stored_mut) {
                    target.any_truncate_if_needed_at(len)?;
                }
                Ok(())
            }

            pub fn write(&mut self) -> Result<()> {
                for target in self.iter_mut().flat_map(BucketVecs::stored_mut) {
                    target.write()?;
                }
                Ok(())
            }

            pub fn compute_cents(
                &mut self,
                max_from: Height,
                price_cents: &impl ReadableVec<Height, Cents>,
                exit: &Exit,
            ) -> Result<()> {
                for bucket in self.iter_mut() {
                    bucket.fees.compute_cents(max_from, price_cents, exit)?;
                }
                Ok(())
            }
        }
    };
}
impl_breakdown!(KindBreakdownVecs, ByKind, OpReturnKind);
impl_breakdown!(PolicyBreakdownVecs, Policy, OpReturnPolicyId);
