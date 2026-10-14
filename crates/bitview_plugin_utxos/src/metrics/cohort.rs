use bitview_cohort::{CohortContext, CohortId};
use bitview_distribution::{
    families::{CumulativeFiat, CumulativeValue},
    metrics::{CohortCapital, CohortCostBasis, CohortSupply, ShareTotals, StoredOutputs},
    state::{MinimalRealizedState, UTXOCohortState},
};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyWindowStartVec, ops::SatsToCents, shapes::Windows};
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use vecdb::{AnyStoredVec, BinaryTransform, Database, ReadableBoxedVec, Rw, StorageMode};

type State = UTXOCohortState<MinimalRealizedState, ()>;

/// One UTXO cohort: an amount range or a spendable output type.
#[derive(Traversable)]
pub struct CohortVecs<M: StorageMode = Rw> {
    pub supply: CohortSupply<M>,
    pub capital: CohortCapital<M>,
    pub outputs: StoredOutputs<M>,
    pub activity: ActivityVecs<M>,
    pub realized: RealizedVecs<M>,
    pub cost_basis: CohortCostBasis<M>,
}

#[derive(Traversable)]
pub struct ActivityVecs<M: StorageMode = Rw> {
    /// Value of the cohort's outputs spent in each block. BTC representations use
    /// the spent output value; USD representations value it at the spending
    /// block's spot price.
    pub transfer_volume: CumulativeValue<M>,
}

#[derive(Traversable)]
pub struct RealizedVecs<M: StorageMode = Rw> {
    /// Profit realized by the cohort's outputs: spending value minus
    /// creation-date value, counted only for profitable spends.
    pub profit: CumulativeFiat<Cents, M>,
    /// Loss realized by the cohort's outputs: creation-date value minus
    /// spending value, counted only for losing spends.
    pub loss: CumulativeFiat<Cents, M>,
}

impl CohortVecs {
    pub fn import(
        db: &Database,
        cohort: CohortId,
        version: Version,
        mappings: &MappingsVecs,
        windows: &Windows<&LazyWindowStartVec>,
        spot: &ReadableBoxedVec<Height, Cents>,
        totals: ShareTotals<'_>,
    ) -> Result<Self> {
        let name = |metric: &str| CohortContext::Utxo.metric_name(cohort, metric);
        let flow_version = version + Version::ONE;
        Ok(Self {
            supply: CohortSupply::import(
                db,
                &name("supply"),
                version,
                mappings,
                windows,
                spot,
                totals.supply,
            )?,
            capital: CohortCapital::import(db, name, version, mappings, windows, totals.capital)?,
            outputs: StoredOutputs::import(db, name, version, mappings, windows)?,
            activity: ActivityVecs {
                transfer_volume: CumulativeValue::import(
                    db,
                    &name("transfer_volume"),
                    flow_version,
                    mappings,
                    windows,
                )?,
            },
            realized: RealizedVecs {
                profit: CumulativeFiat::import(
                    db,
                    &name("realized_profit"),
                    flow_version,
                    mappings,
                    windows,
                )?,
                loss: CumulativeFiat::import(
                    db,
                    &name("realized_loss"),
                    flow_version,
                    mappings,
                    windows,
                )?,
            },
            cost_basis: CohortCostBasis::import(db, name, version + Version::ONE, mappings)?,
        })
    }

    #[inline(always)]
    pub fn push(&mut self, state: &State, price: Cents) {
        self.supply.total.push(state.supply_value());
        self.outputs.push(state.output_counts());
        let sent = state.transfer_volume();
        self.activity
            .transfer_volume
            .push_block(sent, SatsToCents::apply(sent, price));
        let realized = state.realized_block_data();
        self.capital.push(realized.cap);
        self.realized.profit.push_block(realized.profit);
        self.realized.loss.push_block(realized.loss);
        self.cost_basis.push(realized.price());
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        let [sats, cents] = self.activity.transfer_volume.stored_vecs_mut();
        let [unspent_count, spent_count] = self.outputs.stored_vecs_mut();
        [
            self.supply.total.stored_mut(),
            unspent_count,
            spent_count,
            sats,
            cents,
            self.capital.stored_mut(),
            self.realized.profit.stored_mut(),
            self.realized.loss.stored_mut(),
            self.cost_basis.stored_mut(),
        ]
        .into_iter()
    }
}
