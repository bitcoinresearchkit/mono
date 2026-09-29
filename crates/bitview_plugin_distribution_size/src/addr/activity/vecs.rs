use bitview_cohort::{AddrTypeId, ByAddrType, WithAddrTypes};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_transforms::StoredU64ToStoredU32;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerBlockCumulativeAverage, LazyWindowStartVec, PerBlockCumulativeRolling};
use brk_error::Result;
use brk_types::{StoredU32, StoredU64, Version};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, AnyVec, Database, Rw, StorageMode, WritableVec};

use super::{AddrTypeToActivityCounts, BlockActivityCounts};

#[derive(Traversable)]
pub struct AddrActivityVecs<M: StorageMode = Rw> {
    /// Distinct previously seen addresses that received bitcoin in the
    /// represented block while holding no unspent balance immediately before
    /// that receive was applied.
    pub reactivated:
        WithAddrTypes<LazyPerBlockCumulativeAverage<StoredU32, StoredU64, StoredU64ToStoredU32>>,
    /// Distinct addresses that sent bitcoin in the represented block.
    pub sending:
        WithAddrTypes<LazyPerBlockCumulativeAverage<StoredU32, StoredU64, StoredU64ToStoredU32>>,
    /// Distinct addresses that received bitcoin in the represented block.
    pub receiving:
        WithAddrTypes<LazyPerBlockCumulativeAverage<StoredU32, StoredU64, StoredU64ToStoredU32>>,
    /// Distinct addresses that both sent and received bitcoin in the
    /// represented block.
    pub bidirectional:
        WithAddrTypes<LazyPerBlockCumulativeAverage<StoredU32, StoredU64, StoredU64ToStoredU32>>,
    /// Distinct addresses active in the represented block: sending plus
    /// receiving minus bidirectional addresses.
    pub active:
        WithAddrTypes<LazyPerBlockCumulativeAverage<StoredU32, StoredU64, StoredU64ToStoredU32>>,

    #[traversable(hidden)]
    cumulative_reactivated: WithAddrTypes<PerBlockCumulativeRolling<StoredU64, M>>,
    #[traversable(hidden)]
    cumulative_sending: WithAddrTypes<PerBlockCumulativeRolling<StoredU64, M>>,
    #[traversable(hidden)]
    cumulative_receiving: WithAddrTypes<PerBlockCumulativeRolling<StoredU64, M>>,
    #[traversable(hidden)]
    cumulative_bidirectional: WithAddrTypes<PerBlockCumulativeRolling<StoredU64, M>>,
    #[traversable(hidden)]
    cumulative_active: WithAddrTypes<PerBlockCumulativeRolling<StoredU64, M>>,
}

impl AddrActivityVecs {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let cumulative_version = version + Version::TWO;
        let import = |name: &str| -> Result<_> {
            let source = |name: &str| {
                PerBlockCumulativeRolling::forced_import(
                    db,
                    name,
                    cumulative_version + Version::ONE,
                    mappings,
                    window_starts,
                )
            };
            Ok(WithAddrTypes {
                all: source(name)?,
                by_addr_type: ByAddrType::try_from_fn(|id| {
                    source(&format!("{}_{name}", id.name()))
                })?,
            })
        };
        let views = |name: &str, source: &WithAddrTypes<PerBlockCumulativeRolling<StoredU64>>| {
            WithAddrTypes {
                all: LazyPerBlockCumulativeAverage::new(
                    name,
                    version,
                    &source.all.cumulative.height,
                    mappings,
                    window_starts,
                ),
                by_addr_type: AddrTypeId::series(|id, type_name| {
                    LazyPerBlockCumulativeAverage::new(
                        &format!("{type_name}_{name}"),
                        version,
                        &id.select(&source.by_addr_type).cumulative.height,
                        mappings,
                        window_starts,
                    )
                }),
            }
        };
        let cumulative_reactivated = import("reactivated_addrs")?;
        let reactivated = views("reactivated_addrs", &cumulative_reactivated);
        let cumulative_sending = import("sending_addrs")?;
        let sending = views("sending_addrs", &cumulative_sending);
        let cumulative_receiving = import("receiving_addrs")?;
        let receiving = views("receiving_addrs", &cumulative_receiving);
        let cumulative_bidirectional = import("bidirectional_addrs")?;
        let bidirectional = views("bidirectional_addrs", &cumulative_bidirectional);
        let cumulative_active = import("active_addrs")?;
        let active = views("active_addrs", &cumulative_active);

        Ok(Self {
            reactivated,
            sending,
            receiving,
            bidirectional,
            active,
            cumulative_reactivated,
            cumulative_sending,
            cumulative_receiving,
            cumulative_bidirectional,
            cumulative_active,
        })
    }

    pub fn min_resume_len(&self) -> usize {
        [
            &self.cumulative_reactivated,
            &self.cumulative_sending,
            &self.cumulative_receiving,
            &self.cumulative_bidirectional,
            &self.cumulative_active,
        ]
        .into_iter()
        .flat_map(|family| family.iter())
        .map(|v| v.cumulative.height.len())
        .min()
        .unwrap_or_default()
    }

    pub fn par_iter_height_mut(&mut self) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        self.cumulative_sources_mut()
            .map(|v| &mut v.cumulative.height as &mut dyn AnyStoredVec)
            .collect::<Vec<_>>()
            .into_par_iter()
    }

    fn cumulative_sources_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut PerBlockCumulativeRolling<StoredU64>> {
        [
            &mut self.cumulative_reactivated,
            &mut self.cumulative_sending,
            &mut self.cumulative_receiving,
            &mut self.cumulative_bidirectional,
            &mut self.cumulative_active,
        ]
        .into_iter()
        .flat_map(|family| family.iter_mut())
    }

    pub fn reset_height(&mut self) -> Result<()> {
        for source in self.cumulative_sources_mut() {
            source.cumulative.height.reset()?;
        }
        Ok(())
    }

    #[inline(always)]
    pub fn push_height(&mut self, counts: &AddrTypeToActivityCounts) {
        let push = |targets: &mut WithAddrTypes<PerBlockCumulativeRolling<StoredU64>>,
                    value: fn(&BlockActivityCounts) -> u32| {
            let mut total = StoredU64::default();
            for (target, counts) in targets.by_addr_type.values_mut().zip(counts.values()) {
                let value = StoredU64::from(u64::from(value(counts)));
                total += value;
                target.push_block(value);
            }
            targets.all.push_block(total);
        };
        push(&mut self.cumulative_reactivated, |counts| {
            counts.reactivated
        });
        push(&mut self.cumulative_sending, |counts| counts.sending);
        push(&mut self.cumulative_receiving, |counts| counts.receiving);
        push(&mut self.cumulative_bidirectional, |counts| {
            counts.bidirectional
        });
        push(&mut self.cumulative_active, BlockActivityCounts::active);
    }
}
