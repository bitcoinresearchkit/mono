use bitview_cohort::{CohortContext, CohortId};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{CentsSquaredSats, CoinDays};
use bitview_transforms::SatsToCents;
use bitview_traversable::Traversable;
use bitview_vecs::{
    CachedSeries, LazyPerBlock, LazyValuePerBlockCumulativeRolling, LazyWindowStartVec,
    PerBlockCumulativeRolling, Price, SatsCents, import_cached,
};
use brk_error::Result;
use brk_types::{Cents, CentsSats, Height, Sats, Version};
use vecdb::{
    AnyStoredVec, BinaryTransform, Database, ImportableVec, ReadableBoxedVec, Rw, StorageMode,
    WritableVec, ZstdVec, ZstdVecValue,
};

use derive_more::{Deref, DerefMut};

use super::CohortVecs;
use crate::state::{RealizedState, UTXOCohortState, WithCapital};

const MATURED_VERSION: Version = Version::new(5);

/// One UTXO age range: a creation cohort plus what only disjoint age bands have.
#[derive(Deref, DerefMut, Traversable)]
pub struct RangeVecs<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    cohort: CohortVecs<M>,
    /// Amount of unspent bitcoin that ages out of the range during the
    /// represented block interval.
    #[traversable(wrap = "supply", rename = "matured")]
    matured: LazyValuePerBlockCumulativeRolling,
    #[traversable(hidden)]
    matured_sources:
        SatsCents<PerBlockCumulativeRolling<Sats, M>, PerBlockCumulativeRolling<Cents, M>>,
    /// Coin days created in the range: its supply held for the block's
    /// duration, one coin day per BTC per day.
    #[traversable(wrap = "activity", rename = "coindays_created")]
    pub coindays_created: PerBlockCumulativeRolling<CoinDays, M>,
    /// Realized price of the range: realized cap divided by supply, the
    /// satoshi-weighted mean Bitcoin spot price at which its unspent outputs
    /// were created. Zero while the range holds no supply.
    #[traversable(wrap = "realized", rename = "price")]
    price: Price<LazyPerBlock<Cents>>,
    /// Reported in cents per BTC.
    #[traversable(hidden)]
    price_cents: CachedSeries<Height, Cents, M>,
    #[traversable(hidden)]
    pub raw: RawSources<M>,
}

/// Exact raw accounting inputs of an age range, for the aggregated cohorts and the weighted
/// models. Zstd: u128 values that pco cannot encode, read only sequentially.
#[derive(Traversable)]
pub struct RawSources<M: StorageMode = Rw> {
    /// Raw sum of creation price in cents per BTC multiplied by unspent
    /// satoshis. Dividing by 100,000,000 converts it to realized
    /// capitalization in cents; dividing by unspent satoshis gives realized
    /// price in cents per BTC.
    pub cap: M::Stored<ZstdVec<Height, CentsSats>>,
    /// Raw sum of squared creation price in cents per BTC multiplied by
    /// unspent satoshis. Dividing it by the raw creation-price-times-satoshis
    /// sum gives capitalized price in cents per BTC.
    pub capitalized_cap: M::Stored<ZstdVec<Height, CentsSquaredSats>>,
    /// Exact peak-regret product in this block.
    pub peak_regret: M::Stored<ZstdVec<Height, CentsSats>>,
    /// The capitalized-cap product of the supply in profit.
    pub capitalized_cap_in_profit: M::Stored<ZstdVec<Height, CentsSquaredSats>>,
    /// The capitalized-cap product of the supply in loss.
    pub capitalized_cap_in_loss: M::Stored<ZstdVec<Height, CentsSquaredSats>>,
}

impl RangeVecs {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn import(
        db: &Database,
        cohort: CohortId,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        spot_price: &ReadableBoxedVec<Height, Cents>,
        all_supply: &ReadableBoxedVec<Height, Sats>,
    ) -> Result<Self> {
        let full_name = CohortContext::Utxo.full_name(cohort);
        let matured_version = version + MATURED_VERSION;
        let matured_name = format!("{full_name}_matured_supply");
        let matured_sources = SatsCents {
            sats: PerBlockCumulativeRolling::import(
                db,
                &format!("{matured_name}_raw_sats"),
                matured_version + Version::ONE,
                mappings,
                window_starts,
            )?,
            cents: PerBlockCumulativeRolling::import(
                db,
                &format!("{matured_name}_raw_cents"),
                matured_version + Version::ONE,
                mappings,
                window_starts,
            )?,
        };
        let matured = LazyValuePerBlockCumulativeRolling::from_cumulative_sources(
            &matured_name,
            matured_version,
            &matured_sources.sats.cumulative.height,
            &matured_sources.cents.cumulative.height,
            mappings,
            window_starts,
        );
        let coindays_created = PerBlockCumulativeRolling::import(
            db,
            &format!("{full_name}_coindays_created"),
            version + Version::TWO,
            mappings,
            window_starts,
        )?;
        let price_cents = import_cached(
            db,
            &CohortContext::Utxo.metric_name(cohort, "realized_price_cents"),
            version,
        )?;
        let price = Price::from_height_source(
            &CohortContext::Utxo.metric_name(cohort, "realized_price"),
            version,
            &price_cents,
            mappings,
        );
        let raw = RawSources {
            cap: import_raw(db, cohort, "cap_raw", version)?,
            capitalized_cap: import_raw(db, cohort, "capitalized_cap_raw", version)?,
            peak_regret: import_raw(db, cohort, "peak_regret_raw", version)?,
            capitalized_cap_in_profit: import_raw(
                db,
                cohort,
                "capitalized_cap_in_profit_raw",
                version,
            )?,
            capitalized_cap_in_loss: import_raw(
                db,
                cohort,
                "capitalized_cap_in_loss_raw",
                version,
            )?,
        };
        Ok(Self {
            cohort: CohortVecs::import(
                db,
                cohort,
                version,
                mappings,
                window_starts,
                spot_price,
                all_supply,
            )?,
            matured,
            matured_sources,
            coindays_created,
            price,
            price_cents,
            raw,
        })
    }

    /// The tick before the block's spends: supply aging out of the range and coin days created.
    #[inline(always)]
    pub(crate) fn push_tick(&mut self, matured: Sats, coindays_created: CoinDays, price: Cents) {
        debug_assert!(!price.is_nan(), "NaN spot price");
        self.matured_sources.sats.push_block(matured);
        self.matured_sources
            .cents
            .push_block(SatsToCents::apply(matured, price));
        self.coindays_created.push_block(coindays_created);
    }

    #[inline(always)]
    pub(crate) fn push(
        &mut self,
        state: &mut UTXOCohortState<RealizedState, WithCapital>,
        price: Cents,
    ) {
        let profitability = self.cohort.push(state, price);
        let realized = &state.realized;
        self.price_cents
            .push(realized.cap_raw().realized_price(state.supply_value()));
        self.raw.cap.push(realized.cap_raw());
        self.raw
            .capitalized_cap
            .push(realized.capitalized_cap_raw());
        self.raw
            .peak_regret
            .push(CentsSats::new(realized.peak_regret_raw()));
        self.raw
            .capitalized_cap_in_profit
            .push(CentsSquaredSats::new(
                profitability.capitalized_cap_in_profit_raw,
            ));
        self.raw.capitalized_cap_in_loss.push(CentsSquaredSats::new(
            profitability.capitalized_cap_in_loss_raw,
        ));
    }

    pub(crate) fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        let RawSources {
            cap,
            capitalized_cap,
            peak_regret,
            capitalized_cap_in_profit,
            capitalized_cap_in_loss,
        } = &mut self.raw;
        self.cohort.stored_vecs_mut().chain([
            self.matured_sources.sats.stored_mut(),
            self.matured_sources.cents.stored_mut(),
            self.coindays_created.stored_mut(),
            &mut self.price_cents as &mut dyn AnyStoredVec,
            cap as &mut dyn AnyStoredVec,
            capitalized_cap,
            peak_regret,
            capitalized_cap_in_profit,
            capitalized_cap_in_loss,
        ])
    }
}

fn import_raw<T: ZstdVecValue>(
    db: &Database,
    cohort: CohortId,
    name: &str,
    version: Version,
) -> Result<ZstdVec<Height, T>> {
    Ok(ZstdVec::import(
        db,
        &CohortContext::Utxo.metric_name(cohort, name),
        version + Version::TWO,
    )?)
}
