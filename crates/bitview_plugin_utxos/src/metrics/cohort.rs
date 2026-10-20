use bitview_cohort::{AmountRangeId, CohortContext, CohortId};
use bitview_distribution::{
    families::{CumulativeFiat, CumulativeValue},
    metrics::{CohortCapital, CohortCostBasis, CohortSupply, ShareTotals, StoredUtxos},
    state::{CohortState, MinimalRealizedState},
};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyWindowStartVec, shapes::Windows};
use brk_error::Result;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{Database, ReadableBoxedVec, ReadableVec, Rw, StorageMode};

type State = CohortState<MinimalRealizedState, ()>;

/// One UTXO cohort: an amount range or a spendable output type.
#[derive(Traversable)]
pub struct CohortVecs<M: StorageMode = Rw> {
    // Its amounts and values: none for zero-value outputs, whose amounts are all zero.
    #[traversable(flatten)]
    pub values: Option<ValueVecs<M>>,
    pub utxos: StoredUtxos<M>,
}

/// A cohort's amounts and what they're worth.
#[derive(Traversable)]
pub struct ValueVecs<M: StorageMode = Rw> {
    pub supply: CohortSupply<M>,
    pub capital: CohortCapital<M>,
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
        let values = (cohort != CohortId::Amount(AmountRangeId::Zero))
            .then(|| -> Result<_> {
                Ok(ValueVecs {
                    supply: CohortSupply::import(
                        db,
                        &name("supply"),
                        version,
                        mappings,
                        windows,
                        spot,
                        totals.supply,
                    )?,
                    capital: CohortCapital::import(
                        db,
                        name,
                        version,
                        mappings,
                        windows,
                        totals.capital,
                    )?,
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
                    cost_basis: CohortCostBasis::import(
                        db,
                        name,
                        version + Version::ONE,
                        mappings,
                    )?,
                })
            })
            .transpose()?;
        Ok(Self {
            values,
            utxos: StoredUtxos::import(db, name, version, mappings, windows)?,
        })
    }

    #[inline(always)]
    pub fn push(&mut self, state: &State, price: Cents) {
        self.utxos.push(state.output_counts());
        let Some(values) = &mut self.values else {
            return;
        };
        values.supply.total.push(state.supply.value);
        let sent = state.sent;
        values
            .activity
            .transfer_volume
            .push_block(sent, sent.value_at(price));
        let realized = state.realized_block_data();
        values.capital.push(realized.cap);
        values.realized.profit.push_block(realized.profit);
        values.realized.loss.push_block(realized.loss);
        values.cost_basis.push(realized.price());
    }

    /// The cohort's supply at `height`: zero without values.
    pub fn supply_at(&self, height: Height) -> Option<Sats> {
        self.values.as_ref().map_or(Some(Sats::ZERO), |values| {
            values.supply.total.stored.collect_one(height)
        })
    }
}
