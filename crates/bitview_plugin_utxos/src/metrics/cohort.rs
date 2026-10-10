use bitview_cohort::{CohortContext, CohortId};
use bitview_collections::Windows;
use bitview_distribution::{
    families::{CumulativeCount, CumulativeFiat, CumulativeValue, Fiat, UnspentOutputCount},
    metrics::CohortSupply,
    state::{MinimalRealizedState, UTXOCohortState},
};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::Count;
use bitview_transforms::SatsToCents;
use bitview_traversable::Traversable;
use bitview_vecs::{
    CachedSeries, LazyPerBlock, LazySpotValuePerBlock, LazyWindowStartVec, Price, import_cached,
};
use brk_error::Result;
use brk_types::{Cents, Height, Sats, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{
    AnyStoredVec, BinaryTransform, Database, ReadableBoxedVec, Rw, StorageMode, WritableVec,
};

type State = UTXOCohortState<MinimalRealizedState, ()>;

/// One UTXO cohort: an amount range or a spendable output type.
#[derive(Traversable)]
pub struct CohortVecs<M: StorageMode = Rw> {
    pub supply: CohortSupply<M>,
    pub outputs: OutputsVecs<M>,
    pub activity: ActivityVecs<M>,
    pub realized: RealizedVecs<M>,
}

#[derive(Traversable)]
pub struct OutputsVecs<M: StorageMode = Rw> {
    /// Number of transaction outputs that are unspent at the represented block.
    pub unspent_count: UnspentOutputCount<M>,
    /// Number of the cohort's outputs spent in each block.
    pub spent_count: CumulativeCount<Count, M>,
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
    /// Creation-date value of the cohort's unspent outputs: the sum of each
    /// output's BTC value multiplied by Bitcoin's spot price when that output
    /// was created.
    pub cap: Fiat<Cents, M>,
    /// Realized price of the cohort: realized cap divided by supply. Zero while
    /// the cohort holds no supply.
    pub price: Price<LazyPerBlock<Cents>>,
    /// Reported in cents per BTC.
    #[traversable(hidden)]
    price_cents: CachedSeries<Height, Cents, M>,
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
        all_supply: &ReadableBoxedVec<Height, Sats>,
    ) -> Result<Self> {
        let flow_version = version + Version::ONE;
        let price_version = version + Version::ONE;
        let price_cents = import_cached(
            db,
            &CohortContext::Utxo.metric_name(cohort, "realized_price_cents"),
            price_version + Version::TWO,
        )?;
        let price = Price::from_height_source(
            &CohortContext::Utxo.metric_name(cohort, "realized_price"),
            price_version,
            &price_cents,
            mappings,
        );
        Ok(Self {
            supply: CohortSupply::import(db, cohort, version, mappings, windows, spot, all_supply)?,
            outputs: OutputsVecs {
                unspent_count: UnspentOutputCount::import(db, cohort, version, mappings, windows)?,
                spent_count: CumulativeCount::import(
                    db,
                    cohort,
                    "spent_utxo_count",
                    version + Version::ONE,
                    mappings,
                    windows,
                )?,
            },
            activity: ActivityVecs {
                transfer_volume: CumulativeValue::import(
                    db,
                    cohort,
                    "transfer_volume",
                    flow_version,
                    mappings,
                    windows,
                )?,
            },
            realized: RealizedVecs {
                cap: Fiat::import(db, cohort, "realized_cap", version, mappings)?,
                price,
                price_cents,
                profit: CumulativeFiat::import(
                    db,
                    cohort,
                    "realized_profit",
                    flow_version,
                    mappings,
                    windows,
                )?,
                loss: CumulativeFiat::import(
                    db,
                    cohort,
                    "realized_loss",
                    flow_version,
                    mappings,
                    windows,
                )?,
            },
        })
    }

    #[inline(always)]
    pub fn push(&mut self, state: &State, price: Cents) {
        self.supply.total.push(state.supply_value());
        let (unspent, spent) = state.output_counts();
        self.outputs.unspent_count.push(unspent);
        self.outputs.spent_count.push_block(spent);
        let sent = state.transfer_volume();
        self.activity
            .transfer_volume
            .push_block(sent, SatsToCents::apply(sent, price));
        let realized = state.realized_block_data();
        self.realized.cap.push(realized.cap);
        self.realized.price_cents.push(realized.price());
        self.realized.profit.push_block(realized.profit);
        self.realized.loss.push_block(realized.loss);
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        let [sats, cents] = self.activity.transfer_volume.stored_vecs_mut();
        [
            self.supply.total.stored_mut(),
            self.outputs.unspent_count.stored_mut(),
            self.outputs.spent_count.stored_mut(),
            sats,
            cents,
            self.realized.cap.stored_mut(),
            &mut self.realized.price_cents,
            self.realized.profit.stored_mut(),
            self.realized.loss.stored_mut(),
        ]
        .into_iter()
    }
}

/// One spendable output type: a UTXO cohort plus its mean unspent output value.
#[derive(Deref, DerefMut, Traversable)]
pub struct TypeVecs<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub cohort: CohortVecs<M>,
    /// Mean unspent output value, calculated from the same block's supply and
    /// count.
    #[traversable(wrap = "outputs", rename = "avg_amount")]
    pub avg_amount: LazySpotValuePerBlock,
    #[traversable(hidden)]
    avg_amount_sats: CachedSeries<Height, Sats, M>,
}

impl TypeVecs {
    pub fn import(
        db: &Database,
        cohort: CohortId,
        version: Version,
        mappings: &MappingsVecs,
        windows: &Windows<&LazyWindowStartVec>,
        spot: &ReadableBoxedVec<Height, Cents>,
        all_supply: &ReadableBoxedVec<Height, Sats>,
    ) -> Result<Self> {
        let name = cohort.name();
        let avg_amount_sats = import_cached(db, &format!("{name}_avg_utxo_amount_sats"), version)?;
        let avg_amount = LazySpotValuePerBlock::from_sats_source(
            &format!("{name}_avg_utxo_amount"),
            version,
            &avg_amount_sats,
            mappings,
            spot,
        );
        Ok(Self {
            cohort: CohortVecs::import(db, cohort, version, mappings, windows, spot, all_supply)?,
            avg_amount,
            avg_amount_sats,
        })
    }

    #[inline(always)]
    pub fn push(&mut self, state: &State, price: Cents) {
        self.cohort.push(state, price);
        let (unspent, _) = state.output_counts();
        self.avg_amount_sats.push(state.supply_value() / unspent);
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.cohort
            .stored_vecs_mut()
            .chain([&mut self.avg_amount_sats as &mut dyn AnyStoredVec])
    }
}
