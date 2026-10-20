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
    pub supply: CohortSupply<M>,
    pub capital: CohortCapital<M>,
    pub utxos: StoredUtxos<M>,
    pub activity: ActivityVecs<M>,
    pub realized: RealizedVecs<M>,
    pub cost_basis: CohortCostBasis<M>,
}

/// The zero-value outputs' cohort: its counts alone, its amounts and values all zero.
#[derive(Traversable)]
pub struct ZeroVecs<M: StorageMode = Rw> {
    pub utxos: StoredUtxos<M>,
}

impl ZeroVecs {
    pub fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        windows: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let name = |metric: &str| {
            CohortContext::Utxo.metric_name(CohortId::Amount(AmountRangeId::Zero), metric)
        };
        Ok(Self {
            utxos: StoredUtxos::import(db, name, version, mappings, windows)?,
        })
    }

    #[inline(always)]
    pub fn push(&mut self, state: &State) {
        debug_assert_eq!(
            state.supply.value,
            Sats::ZERO,
            "zero-value outputs hold value"
        );
        self.utxos.push(state.output_counts());
    }

    /// The band's state at the end of `height`, `None` without that block: no value.
    pub fn restore(&self, state: &mut State, height: Height) -> Option<()> {
        state.supply.value = Sats::ZERO;
        state.supply.utxo_count = u64::from(self.utxos.count.stored.collect_one(height)?);
        Some(())
    }
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
            utxos: StoredUtxos::import(db, name, version, mappings, windows)?,
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
        self.supply.total.push(state.supply.value);
        self.utxos.push(state.output_counts());
        let sent = state.sent;
        self.activity
            .transfer_volume
            .push_block(sent, sent.value_at(price));
        let realized = state.realized_block_data();
        self.capital.push(realized.cap);
        self.realized.profit.push_block(realized.profit);
        self.realized.loss.push_block(realized.loss);
        self.cost_basis.push(realized.price());
    }
}
