use bitview_cohort::{PROFITABILITY_RANGE_NAMES, ProfitabilityRange, ProfitabilityRangeId};
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_primitives::PartsPerMillion32;
use bitview_traversable::Traversable;
use bitview_vecs::{
    CachedSeries, LazyFiatPerBlock, LazyPercentPerBlock, LazySpotValuePerBlock, import_cached,
    ops::Quotient,
};
use brk_error::Result;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{Database, PcoVecValue, ReadableBoxedVec, Rw, StorageMode, WritableVec};

use crate::{
    bucket::Bucket,
    terms::{TERMS, Terms},
};

type Bands<T, M> = Terms<ProfitabilityRange<CachedSeries<Height, T, M>>>;
type Totals<T, M> = Terms<CachedSeries<Height, T, M>>;

/// The 25 profitability bands for all, short-term and long-term holders.
#[derive(Traversable)]
pub struct Metrics<M: StorageMode = Rw> {
    #[traversable(flatten)]
    terms: Terms<ProfitabilityRange<Band>>,
    #[traversable(hidden)]
    supply_stored: Bands<Sats, M>,
    #[traversable(hidden)]
    capital_stored: Bands<Cents, M>,
    /// Each band's share of its filter's capital, from the exact capitals: whole cents would
    /// distort the proportions of capitals worth a few cents.
    #[traversable(hidden)]
    capital_share_stored: Bands<PartsPerMillion32, M>,
    /// Each filter's supply, the denominator of its bands' supply shares.
    #[traversable(hidden)]
    supply_total: Totals<Sats, M>,
}

#[derive(Clone, Traversable)]
struct Band {
    /// Outputs are placed by their creation price rounded to four significant digits of cents
    /// (exact under $100, then steps of at most 0.1%); a band's edge rounds up to that grid, and
    /// the spot's own grid price counts as profit.
    supply: BandSupply,
    /// Outputs are placed by their creation price rounded to four significant digits of cents
    /// (exact under $100, then steps of at most 0.1%); a band's edge rounds up to that grid, and
    /// the spot's own grid price counts as profit.
    capital: BandCapital,
}

#[derive(Clone, Traversable)]
struct BandSupply {
    /// Supply: amount of bitcoin held in the band's unspent transaction outputs.
    #[traversable(lead = "Supply: the bitcoin held.")]
    total: LazySpotValuePerBlock,
    /// The band's share of its age filter's supply; undefined (null) when that supply is zero.
    #[traversable(
        lead = "Supply share: their part of their group's supply (all coins, or short- or long-term holders')."
    )]
    share: LazyPercentPerBlock<PartsPerMillion32>,
}

#[derive(Clone, Traversable)]
struct BandCapital {
    /// Capital: the band's unspent outputs valued at Bitcoin's spot price when each was created.
    #[traversable(
        lead = "Capital: what holders paid for these coins, in all, each at the price it last moved at."
    )]
    total: LazyFiatPerBlock<Cents>,
    /// Realized cap: the sum of each unspent output's BTC value times the spot price when it was
    /// created.
    #[traversable(
        lead = "Realized cap: what holders paid for their bitcoin, in all, each coin at the price it last moved at."
    )]
    realized_cap: LazyFiatPerBlock<Cents>,
    /// The band's share of its age filter's capital; undefined (null) when that capital is zero.
    #[traversable(
        lead = "Capital share: their part of their group's capital (all coins, or short- or long-term holders')."
    )]
    share: LazyPercentPerBlock<PartsPerMillion32>,
}

/// A band's stored series and its filter's totals.
struct BandSources<'a> {
    supply: &'a CachedSeries<Height, Sats>,
    capital: &'a CachedSeries<Height, Cents>,
    supply_total: &'a CachedSeries<Height, Sats>,
    capital_share: &'a CachedSeries<Height, PartsPerMillion32>,
}

impl Band {
    fn new(
        name: impl Fn(&str) -> String,
        version: Version,
        sources: BandSources<'_>,
        mappings: &Mappings,
        spot: &ReadableBoxedVec<Height, Cents>,
    ) -> Self {
        let fiat = |metric: &str| {
            LazyFiatPerBlock::from_cents_source(&name(metric), version, sources.capital, mappings)
        };
        Self {
            supply: BandSupply {
                total: LazySpotValuePerBlock::from_sats_source(
                    &name("supply"),
                    version,
                    sources.supply,
                    mappings,
                    spot,
                ),
                share: LazyPercentPerBlock::from_ratio::<Sats, Sats, Quotient<PartsPerMillion32>>(
                    &name("supply_share"),
                    version,
                    sources.supply,
                    sources.supply_total,
                    mappings,
                ),
            },
            capital: BandCapital {
                total: fiat("capital"),
                realized_cap: fiat("realized_cap"),
                share: LazyPercentPerBlock::from_height_source(
                    &name("capital_share"),
                    version,
                    sources.capital_share,
                    mappings,
                ),
            },
        }
    }
}

impl Metrics {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &Mappings,
        spot: &ReadableBoxedVec<Height, Cents>,
    ) -> Result<Box<Self>> {
        let supply_stored = Self::import_bands(db, "supply_sats", version)?;
        let capital_stored = Self::import_bands(db, "realized_cap_cents", version)?;
        let supply_total = Self::import_totals(db, "profitability_supply_sats", version)?;
        let capital_share_stored = Self::import_bands(db, "capital_share_ppm", version)?;
        let terms = Terms::from_fn(|term| {
            ProfitabilityRange::from_fn(|band| {
                let band_name = band.select(&PROFITABILITY_RANGE_NAMES).id;
                Band::new(
                    |metric| term.metric_name(&format!("{band_name}_{metric}")),
                    version,
                    BandSources {
                        supply: band.select(supply_stored.select(term)),
                        capital: band.select(capital_stored.select(term)),
                        supply_total: supply_total.select(term),
                        capital_share: band.select(capital_share_stored.select(term)),
                    },
                    mappings,
                    spot,
                )
            })
        });
        Ok(Box::new(Self {
            terms,
            supply_stored,
            capital_stored,
            capital_share_stored,
            supply_total,
        }))
    }

    fn import_bands<T: PcoVecValue>(
        db: &Database,
        metric: &str,
        version: Version,
    ) -> Result<Bands<T, Rw>> {
        Terms::try_from_fn(|term| {
            ProfitabilityRange::try_from_fn(|band| {
                let band_name = band.select(&PROFITABILITY_RANGE_NAMES).id;
                import_cached(
                    db,
                    &term.metric_name(&format!("{band_name}_{metric}")),
                    version,
                )
            })
        })
    }

    fn import_totals<T: PcoVecValue>(
        db: &Database,
        metric: &str,
        version: Version,
    ) -> Result<Totals<T, Rw>> {
        Terms::try_from_fn(|term| import_cached(db, &term.metric_name(metric), version))
    }

    pub(crate) fn push(&mut self, bands: &ProfitabilityRange<Bucket>, totals: &Bucket) {
        for term in TERMS {
            for &band in ProfitabilityRangeId::ALL {
                let bucket = band.select(bands);
                band.select_mut(self.supply_stored.select_mut(term))
                    .push(*bucket.supply.select(term));
                band.select_mut(self.capital_stored.select_mut(term))
                    .push(*bucket.cap.select(term));
                band.select_mut(self.capital_share_stored.select_mut(term))
                    .push(PartsPerMillion32::from(
                        bucket.cap_raw.select(term).inner() as f64
                            / totals.cap_raw.select(term).inner() as f64,
                    ));
            }
            self.supply_total
                .select_mut(term)
                .push(*totals.supply.select(term));
        }
    }
}
