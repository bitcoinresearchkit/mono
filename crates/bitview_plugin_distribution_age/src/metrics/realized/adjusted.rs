use bitview_cohort::{CohortContext, CohortId, Term, UTXOAllAndSth, UTXOAllAndSthId};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_transforms::SoprRatio;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyWindowStartVec, PerBlockCumulativeRolling, RollingWindows};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, StoredF32, Version};
use vecdb::{AnyStoredVec, Database, ReadableVec, Rw, StorageMode};

use super::RealizedAggregateSources;

const SOURCE_VERSION: Version = Version::ONE;
const RATIO_VERSION: Version = Version::ONE;

#[derive(Traversable)]
pub struct AdjustedSoprVecs<M: StorageMode = Rw> {
    /// For each supported trailing window, adjusted spent output profit ratio:
    /// spending-date value divided by creation-date value for outputs at least
    /// one hour old. Values above one mean aggregate profit and values below
    /// one mean aggregate loss. Returns one when creation-date value is zero.
    pub ratio: UTXOAllAndSth<RollingWindows<StoredF32, M>>,
    /// Spending-date USD value of outputs at least one hour old spent from an
    /// all-chain or short-term-holder cohort.
    pub transfer_volume: UTXOAllAndSth<PerBlockCumulativeRolling<Cents, M>>,
    /// Creation-date USD value of outputs at least one hour old spent from an
    /// all-chain or short-term-holder cohort.
    pub value_destroyed: UTXOAllAndSth<PerBlockCumulativeRolling<Cents, M>>,
}

impl AdjustedSoprVecs {
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let source_version = version + SOURCE_VERSION;
        let ratio_version = source_version + RATIO_VERSION;
        let transfer_volume = Self::import_cumulative(
            db,
            "adj_value_created",
            source_version,
            mappings,
            window_starts,
        )?;
        let value_destroyed = Self::import_cumulative(
            db,
            "adj_value_destroyed",
            source_version,
            mappings,
            window_starts,
        )?;
        let ratio = UTXOAllAndSth {
            all: RollingWindows::forced_import(
                db,
                "asopr",
                Self::cohort_version(ratio_version, UTXOAllAndSthId::All),
                mappings,
            )?,
            sth: RollingWindows::forced_import(
                db,
                &Self::cohort_metric_name(UTXOAllAndSthId::Sth, "asopr"),
                Self::cohort_version(ratio_version, UTXOAllAndSthId::Sth),
                mappings,
            )?,
        };

        Ok(Self {
            ratio,
            transfer_volume,
            value_destroyed,
        })
    }

    fn import_cumulative(
        db: &Database,
        metric: &str,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<UTXOAllAndSth<PerBlockCumulativeRolling<Cents>>> {
        UTXOAllAndSth::try_from_fn(|id| {
            PerBlockCumulativeRolling::forced_import(
                db,
                &Self::cohort_metric_name(id, metric),
                Self::cohort_version(version, id) + Version::ONE,
                mappings,
                window_starts,
            )
        })
    }

    fn cohort_metric_name(id: UTXOAllAndSthId, metric: &str) -> String {
        match id {
            UTXOAllAndSthId::All => CohortContext::Utxo.metric_name(CohortId::All, metric),
            UTXOAllAndSthId::Sth => {
                CohortContext::Utxo.metric_name(CohortId::Term(Term::Sth), metric)
            }
        }
    }

    fn cohort_version(base: Version, id: UTXOAllAndSthId) -> Version {
        base + Version::ONE
            + if matches!(id, UTXOAllAndSthId::All) {
                Version::ONE
            } else {
                Version::ZERO
            }
    }

    pub fn compute<V1, V2>(
        &mut self,
        max_from: Height,
        sources: &UTXOAllAndSth<&RealizedAggregateSources>,
        under_1h_transfer_volume_cumulative: &V1,
        under_1h_value_destroyed_cumulative: &V2,
        exit: &Exit,
    ) -> Result<()>
    where
        V1: ReadableVec<Height, Cents>,
        V2: ReadableVec<Height, Cents>,
    {
        for id in UTXOAllAndSthId::ALL {
            let source = id.select(sources);
            id.select_mut(&mut self.transfer_volume)
                .cumulative
                .height
                .compute_transform2(
                    max_from,
                    &source.activity.transfer_volume.cumulative.cents.height,
                    under_1h_transfer_volume_cumulative,
                    |(height, base, under_1h, _)| (height, base - under_1h),
                    exit,
                )?;
            id.select_mut(&mut self.value_destroyed)
                .cumulative
                .height
                .compute_transform2(
                    max_from,
                    &source.realized.value_destroyed.cumulative.cents.height,
                    under_1h_value_destroyed_cumulative,
                    |(height, base, under_1h, _)| (height, base - under_1h),
                    exit,
                )?;
        }

        let Self {
            ratio,
            transfer_volume,
            value_destroyed,
        } = self;
        for id in UTXOAllAndSthId::ALL {
            for ((target, transferred), destroyed) in id
                .select_mut(ratio)
                .as_mut_array()
                .into_iter()
                .zip(id.select(transfer_volume).sum.as_array())
                .zip(id.select(value_destroyed).sum.as_array())
            {
                target.compute_binary::<_, _, SoprRatio>(
                    max_from,
                    &transferred.height,
                    &destroyed.height,
                    exit,
                )?;
            }
        }

        Ok(())
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut vecs: Vec<&mut dyn AnyStoredVec> = self
            .transfer_volume
            .iter_mut()
            .chain(self.value_destroyed.iter_mut())
            .map(|v| &mut v.cumulative.height as &mut dyn AnyStoredVec)
            .collect();
        vecs.extend(
            self.ratio
                .iter_mut()
                .flat_map(|value| value.as_mut_array())
                .map(|value| &mut value.height as &mut dyn AnyStoredVec),
        );
        vecs
    }
}
