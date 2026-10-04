use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{Bytes, Count, OpReturnKind, OpReturnPolicyId};
use bitview_traversable::Traversable;
use bitview_vecs::{LazyWindowStartVec, PerBlockCumulativeRolling};
use brk_error::{Error, Result};
use brk_types::{Height, Sats, VSize, Version};
use vecdb::{AnyStoredVec, AnyVec, Database, ReadableCloneableVec, Rw, VecIndex};

use super::{BlockMetrics, DataBytesSeries, FeesSeries};
use crate::{by_kind::ByKind, policy::Policy};

#[derive(Traversable)]
pub struct BreakdownVecs<C, D, V, F> {
    /// Number of OP_RETURN outputs assigned to each bucket.
    pub output_count: C,
    /// Script bytes following OP_RETURN assigned to each bucket.
    pub data_bytes: D,
    /// Transactions assigned to each bucket, counted at most once per bucket.
    pub tx_count: C,
    /// Full virtual sizes of transactions assigned to each bucket.
    pub tx_vsize: V,
    /// Full fees of transactions assigned to each bucket.
    pub fees: F,
}

macro_rules! impl_breakdown {
    ($name:ident, $group:ident, $id:ident) => {
        pub type $name<M = Rw> = BreakdownVecs<
            $group<PerBlockCumulativeRolling<Count, M>>,
            $group<DataBytesSeries<M>>,
            $group<PerBlockCumulativeRolling<VSize, M>>,
            $group<FeesSeries<M>>,
        >;
        impl $name {
            #[allow(clippy::too_many_arguments)]
            pub fn import(
                db: &Database,
                series_prefix: &str,
                version: Version,
                mappings: &MappingsVecs,
                window_starts: &Windows<&LazyWindowStartVec>,
                total_data: &impl ReadableCloneableVec<Height, Bytes>,
                block_size: &impl ReadableCloneableVec<Height, Bytes>,
                chain_fees: &impl ReadableCloneableVec<Height, Sats>,
            ) -> Result<Self> {
                let version = version + Version::ONE;
                let output_count = $group::try_new(|_, name| {
                    PerBlockCumulativeRolling::import(
                        db,
                        &format!("{series_prefix}_{name}_output_count"),
                        version,
                        mappings,
                        window_starts,
                    )
                })?;
                let data_bytes = $group::try_new(|_, name| {
                    let prefix = format!("{series_prefix}_{name}");
                    let source = PerBlockCumulativeRolling::import(
                        db,
                        &format!("{prefix}_data_bytes"),
                        version,
                        mappings,
                        window_starts,
                    )?;
                    Ok::<_, Error>(DataBytesSeries::new(
                        &prefix, version, source, total_data, block_size, mappings,
                    ))
                })?;
                let tx_count = $group::try_new(|_, name| {
                    PerBlockCumulativeRolling::import(
                        db,
                        &format!("{series_prefix}_{name}_tx_count"),
                        version,
                        mappings,
                        window_starts,
                    )
                })?;
                let tx_vsize = $group::try_new(|_, name| {
                    PerBlockCumulativeRolling::import(
                        db,
                        &format!("{series_prefix}_{name}_tx_vsize"),
                        version,
                        mappings,
                        window_starts,
                    )
                })?;
                let fees = $group::try_new(|_, name| {
                    let prefix = format!("{series_prefix}_{name}");
                    let source = PerBlockCumulativeRolling::import(
                        db,
                        &format!("{prefix}_fees"),
                        version,
                        mappings,
                        window_starts,
                    )?;
                    Ok::<_, Error>(FeesSeries::new(
                        &prefix,
                        version,
                        source,
                        chain_fees,
                        window_starts,
                        mappings,
                    ))
                })?;
                Ok(Self {
                    output_count,
                    data_bytes,
                    tx_count,
                    tx_vsize,
                    fees,
                })
            }

            pub fn len(&self) -> usize {
                self.output_count
                    .iter()
                    .map(|v| v.cumulative.height.len())
                    .chain(self.data_bytes.iter().map(|v| v.cumulative.height.len()))
                    .chain(self.tx_count.iter().map(|v| v.cumulative.height.len()))
                    .chain(self.tx_vsize.iter().map(|v| v.cumulative.height.len()))
                    .chain(self.fees.iter().map(|v| v.cumulative.height.len()))
                    .min()
                    .unwrap_or_default()
            }

            pub fn push(&mut self, values: [BlockMetrics; $id::ALL.len()]) {
                for (id, target) in $id::ALL.iter().zip(self.output_count.iter_mut()) {
                    target.push_block(Count::from(id.get(&values).output_count));
                }
                for (id, target) in $id::ALL.iter().zip(self.data_bytes.iter_mut()) {
                    target.data_bytes.push_block(id.get(&values).data_bytes);
                }
                for (id, target) in $id::ALL.iter().zip(self.tx_count.iter_mut()) {
                    target.push_block(Count::from(id.get(&values).tx_count));
                }
                for (id, target) in $id::ALL.iter().zip(self.tx_vsize.iter_mut()) {
                    target.push_block(id.get(&values).tx_vsize);
                }
                for (id, target) in $id::ALL.iter().zip(self.fees.iter_mut()) {
                    target.fees.push_block(id.get(&values).fees);
                }
            }

            fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
                self.output_count
                    .iter_mut()
                    .map(|v| &mut v.cumulative.height as &mut dyn AnyStoredVec)
                    .chain(
                        self.data_bytes
                            .iter_mut()
                            .map(|v| &mut v.data_bytes.cumulative.height as &mut dyn AnyStoredVec),
                    )
                    .chain(
                        self.tx_count
                            .iter_mut()
                            .map(|v| &mut v.cumulative.height as &mut dyn AnyStoredVec),
                    )
                    .chain(
                        self.tx_vsize
                            .iter_mut()
                            .map(|v| &mut v.cumulative.height as &mut dyn AnyStoredVec),
                    )
                    .chain(
                        self.fees
                            .iter_mut()
                            .map(|v| &mut v.fees.cumulative.height as &mut dyn AnyStoredVec),
                    )
            }

            pub fn validate_and_truncate(
                &mut self,
                version: Version,
                height: Height,
            ) -> Result<()> {
                for target in self.stored_vecs_mut() {
                    target.any_validate_computed_version_or_reset(version)?;
                    target.any_truncate_if_needed_at(height.to_usize())?;
                }
                Ok(())
            }

            pub fn truncate_if_needed_at(&mut self, len: usize) -> Result<()> {
                for target in self.stored_vecs_mut() {
                    target.any_truncate_if_needed_at(len)?;
                }
                Ok(())
            }

            pub fn write(&mut self) -> Result<()> {
                for target in self.stored_vecs_mut() {
                    target.write()?;
                }
                Ok(())
            }
        }
    };
}
impl_breakdown!(KindBreakdownVecs, ByKind, OpReturnKind);
impl_breakdown!(PolicyBreakdownVecs, Policy, OpReturnPolicyId);
