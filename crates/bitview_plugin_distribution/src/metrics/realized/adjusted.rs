use brk_error::Result;

use bitview_cohort::{
    CohortContext, Filter, TERM_NAMES, Term, UTXO_ALL_NAME, UTXOAllAndSth, UTXOAllAndSthId,
};
use bitview_traversable::Traversable;
use brk_types::{Cents, Height, StoredF32, Version};
use vecdb::{
    AnyStoredVec, BinaryTransform, ColumnId, Database, Exit, ReadableVec, Rw, StorageMode,
};

use bitview_compute::{
    CachedWindowStartVec, ColumnarPerBlockCumulativeRolling, ColumnarRollingWindows,
    LazyColumnPerBlockCumulativeRolling, SoprRatio, Windows,
};

use super::AdjustedSoprComputeSource;

const SOURCE_VERSION: Version = Version::ONE;
const RATIO_VERSION: Version = Version::ONE;

#[derive(Traversable)]
pub struct AdjustedSoprVecs<M: StorageMode = Rw> {
    /// Adjusted spent output profit ratio over the named trailing window:
    /// spending-date value divided by creation-date value for outputs at least
    /// one hour old. Returns one when creation-date value is zero.
    pub ratio: UTXOAllAndSth<ColumnarRollingWindows<StoredF32, M>>,
    /// Spending-date USD value of outputs at least one hour old spent from the
    /// selected all-chain or short-term-holder cohort.
    pub transfer_volume: ColumnarPerBlockCumulativeRolling<
        Cents,
        UTXOAllAndSthId,
        UTXOAllAndSth<LazyColumnPerBlockCumulativeRolling<Cents, UTXOAllAndSthId>>,
        M,
    >,
    /// Creation-date USD value of outputs at least one hour old spent from the
    /// selected all-chain or short-term-holder cohort.
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
        mappings: &bitview_plugin_mappings::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Result<Self> {
        let source_version = version + SOURCE_VERSION;
        let ratio_version = source_version + RATIO_VERSION;
        let transfer_volume = Self::import_cumulative(
            db,
            "adjusted_sopr_transfer_volume_cumulative_by_cohort",
            "adj_value_created",
            source_version,
            mappings,
            cached_starts,
        )?;
        let value_destroyed = Self::import_cumulative(
            db,
            "adjusted_sopr_value_destroyed_cumulative_by_cohort",
            "adj_value_destroyed",
            source_version,
            mappings,
            cached_starts,
        )?;
        let ratio = UTXOAllAndSth {
            all: ColumnarRollingWindows::forced_import(
                db,
                "asopr",
                Self::cohort_version(ratio_version, UTXOAllAndSthId::All),
                mappings,
            )?,
            sth: ColumnarRollingWindows::forced_import(
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
        matrix_name: &str,
        metric: &str,
        version: Version,
        mappings: &bitview_plugin_mappings::Vecs,
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
                    mappings,
                    cached_starts,
                ),
                sth: LazyColumnPerBlockCumulativeRolling::new(
                    &Self::cohort_metric_name(UTXOAllAndSthId::Sth, metric),
                    Self::cohort_version(version, UTXOAllAndSthId::Sth),
                    source,
                    UTXOAllAndSthId::Sth,
                    mappings,
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
        under_1h_transfer_volume_cumulative: &V1,
        under_1h_value_destroyed_cumulative: &V2,
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
            |_| under_1h_transfer_volume_cumulative,
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
            |_| under_1h_value_destroyed_cumulative,
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
                    SoprRatio::apply(transfer_volume, value_destroyed)
                },
                exit,
            )?;
        }

        Ok(())
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
