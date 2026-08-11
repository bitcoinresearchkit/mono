use brk_cohort::{
    CohortContext, Filter, TERM_NAMES, Term, UTXO_ALL_NAME, UTXOAllAndSth, UTXOAllAndSthId,
};
use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{Cents, Height, StoredF64, Version};
use vecdb::{
    AnyStoredVec, AnyVec, BinaryTransform, ColumnId, Database, Exit, ReadableVec, Rw, StorageMode,
};

use crate::{
    indexes,
    internal::{
        CachedWindowStartVec, ColumnarPerBlockCumulativeRolling, ColumnarRollingWindows,
        LazyColumnPerBlockCumulativeRolling, RatioCents64, Windows,
    },
};

use super::AdjustedSoprComputeSource;

#[derive(Traversable)]
pub struct AdjustedSoprVecs<M: StorageMode = Rw> {
    pub ratio: UTXOAllAndSth<ColumnarRollingWindows<StoredF64, M>>,
    pub transfer_volume: ColumnarPerBlockCumulativeRolling<
        Cents,
        UTXOAllAndSthId,
        UTXOAllAndSth<LazyColumnPerBlockCumulativeRolling<Cents, UTXOAllAndSthId>>,
        M,
    >,
    pub value_destroyed: ColumnarPerBlockCumulativeRolling<
        Cents,
        UTXOAllAndSthId,
        UTXOAllAndSth<LazyColumnPerBlockCumulativeRolling<Cents, UTXOAllAndSthId>>,
        M,
    >,
}

impl AdjustedSoprVecs {
    pub fn forced_import(
        db: &Database,
        version: Version,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let transfer_volume = Self::import_cumulative(
            db,
            "adjusted_sopr_transfer_volume_cumulative_by_cohort",
            "adj_value_created",
            version,
            indexes,
            cached_starts,
        )?;
        let value_destroyed = Self::import_cumulative(
            db,
            "adjusted_sopr_value_destroyed_cumulative_by_cohort",
            "adj_value_destroyed",
            version,
            indexes,
            cached_starts,
        )?;
        let ratio = UTXOAllAndSth {
            all: ColumnarRollingWindows::forced_import(
                db,
                "asopr",
                Self::cohort_version(version, UTXOAllAndSthId::All),
                indexes,
            )?,
            sth: ColumnarRollingWindows::forced_import(
                db,
                &Self::cohort_metric_name(UTXOAllAndSthId::Sth, "asopr"),
                Self::cohort_version(version, UTXOAllAndSthId::Sth),
                indexes,
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
        matrix_name: &str,
        metric: &str,
        version: Version,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<
        ColumnarPerBlockCumulativeRolling<
            Cents,
            UTXOAllAndSthId,
            UTXOAllAndSth<LazyColumnPerBlockCumulativeRolling<Cents, UTXOAllAndSthId>>,
        >,
    > {
        ColumnarPerBlockCumulativeRolling::forced_import(
            db,
            matrix_name,
            version + Version::ONE,
            |source| UTXOAllAndSth {
                all: LazyColumnPerBlockCumulativeRolling::new(
                    metric,
                    Self::cohort_version(version, UTXOAllAndSthId::All),
                    source,
                    UTXOAllAndSthId::All,
                    indexes,
                    cached_starts,
                ),
                sth: LazyColumnPerBlockCumulativeRolling::new(
                    &Self::cohort_metric_name(UTXOAllAndSthId::Sth, metric),
                    Self::cohort_version(version, UTXOAllAndSthId::Sth),
                    source,
                    UTXOAllAndSthId::Sth,
                    indexes,
                    cached_starts,
                ),
            },
        )
    }

    fn cohort_metric_name(id: UTXOAllAndSthId, metric: &str) -> String {
        match id {
            UTXOAllAndSthId::All => {
                CohortContext::Utxo.metric_name(&Filter::All, UTXO_ALL_NAME.id, metric)
            }
            UTXOAllAndSthId::Sth => CohortContext::Utxo.metric_name(
                &Filter::Term(Term::Sth),
                TERM_NAMES.short.id,
                metric,
            ),
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
        sources: &UTXOAllAndSth<AdjustedSoprComputeSource>,
        under_1h_transfer_volume: &V1,
        under_1h_value_destroyed: &V2,
        exit: &Exit,
    ) -> Result<()>
    where
        V1: ReadableVec<Height, Cents>,
        V2: ReadableVec<Height, Cents>,
    {
        self.transfer_volume.compute_columns2(
            max_from,
            |id| {
                &id.select(sources)
                    .activity
                    .transfer_volume
                    .cumulative
                    .cents
                    .height
            },
            |_| under_1h_transfer_volume,
            |_, base, under_1h| base - under_1h,
            exit,
        )?;
        self.value_destroyed.compute_columns2(
            max_from,
            |id| {
                &id.select(sources)
                    .realized
                    .value_destroyed
                    .cumulative
                    .cents
                    .height
            },
            |_| under_1h_value_destroyed,
            |_, base, under_1h| base - under_1h,
            exit,
        )?;

        let Self {
            ratio,
            transfer_volume,
            value_destroyed,
        } = self;
        for id in UTXOAllAndSthId::ALL {
            id.select_mut(ratio).compute_columns2(
                max_from,
                |window| {
                    &window
                        .select(&id.select(&transfer_volume.series).sum)
                        .height
                },
                |window| {
                    &window
                        .select(&id.select(&value_destroyed.series).sum)
                        .height
                },
                |_, transfer_volume, value_destroyed| {
                    RatioCents64::apply(transfer_volume, value_destroyed)
                },
                exit,
            )?;
        }

        Ok(())
    }

    pub fn min_len(&self) -> usize {
        self.transfer_volume
            .cumulative
            .len()
            .min(self.value_destroyed.cumulative.len())
            .min(
                self.ratio
                    .iter()
                    .map(|value| value.height.len())
                    .min()
                    .unwrap_or_default(),
            )
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut vecs = vec![
            self.transfer_volume.stored_mut(),
            self.value_destroyed.stored_mut(),
        ];
        vecs.extend(self.ratio.iter_mut().map(|value| value.stored_mut()));
        vecs
    }
}
