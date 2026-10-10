use bitview_cohort::{ProfitabilityRange, ProfitabilityRangeId};
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_primitives::PartsPerMillion32;
use bitview_transforms::Quotient;
use bitview_traversable::Traversable;
use bitview_vecs::{
    CachedSeries, LazyFiatPerBlock, LazyPercentPerBlock, LazySpotValuePerBlock, import_cached,
};
use brk_error::Result;
use brk_types::{Cents, CentsSats, CentsSigned, Height, Sats, Version};
use vecdb::{AnyStoredVec, Database, PcoVecValue, ReadableBoxedVec, Rw, StorageMode, WritableVec};

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
    #[traversable(hidden)]
    net_pnl_stored: Bands<CentsSigned, M>,
    /// Each filter's supply and capital, the denominators of its bands' shares.
    #[traversable(hidden)]
    supply_total: Totals<Sats, M>,
    #[traversable(hidden)]
    capital_total: Totals<Cents, M>,
}

/// One band of one age filter: the subset of the cohort node that a band carries.
#[derive(Clone, Traversable)]
struct Band {
    supply: BandSupply,
    capital: BandCapital,
    unrealized: BandUnrealized,
}

#[derive(Clone, Traversable)]
struct BandSupply {
    /// Supply: amount of bitcoin held in the band's unspent transaction outputs.
    total: LazySpotValuePerBlock,
    /// The band's share of its age filter's supply; a filter's bands add up to 100%.
    share: LazyPercentPerBlock<PartsPerMillion32>,
}

#[derive(Clone, Traversable)]
struct BandCapital {
    /// Capital: the band's unspent outputs valued at Bitcoin's spot price when each was created.
    total: LazyFiatPerBlock<Cents>,
    /// Realized cap: the capital, under its jargon name.
    realized_cap: LazyFiatPerBlock<Cents>,
    /// The band's share of its age filter's capital; a filter's bands add up to 100%.
    share: LazyPercentPerBlock<PartsPerMillion32>,
}

#[derive(Clone, Traversable)]
struct BandUnrealized {
    /// Net unrealized profit and loss: the supply's market value minus its capital, positive in
    /// profit and negative in loss.
    net_pnl: LazyFiatPerBlock<CentsSigned>,
}

/// A band's stored series and its filter's totals.
struct BandSources<'a> {
    supply: &'a CachedSeries<Height, Sats>,
    capital: &'a CachedSeries<Height, Cents>,
    net_pnl: &'a CachedSeries<Height, CentsSigned>,
    supply_total: &'a CachedSeries<Height, Sats>,
    capital_total: &'a CachedSeries<Height, Cents>,
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
                share: LazyPercentPerBlock::from_ratio::<Cents, Cents, Quotient<PartsPerMillion32>>(
                    &name("capital_share"),
                    version,
                    sources.capital,
                    sources.capital_total,
                    mappings,
                ),
            },
            unrealized: BandUnrealized {
                net_pnl: LazyFiatPerBlock::from_cents_source(
                    &name("net_unrealized_pnl"),
                    version,
                    sources.net_pnl,
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
        let net_pnl_stored = Self::import_bands(db, "net_unrealized_pnl_cents", version)?;
        let supply_total = Self::import_totals(db, "profitability_supply_sats", version)?;
        let capital_total = Self::import_totals(db, "profitability_capital_cents", version)?;
        let terms = Terms::from_fn(|term| {
            ProfitabilityRange::from_fn(|band| {
                let band_name = band.select(ProfitabilityRange::names()).id;
                Band::new(
                    |metric| term.metric_name(&format!("{band_name}_{metric}")),
                    version,
                    BandSources {
                        supply: band.select(supply_stored.select(term)),
                        capital: band.select(capital_stored.select(term)),
                        net_pnl: band.select(net_pnl_stored.select(term)),
                        supply_total: supply_total.select(term),
                        capital_total: capital_total.select(term),
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
            net_pnl_stored,
            supply_total,
            capital_total,
        }))
    }

    fn import_bands<T: PcoVecValue>(
        db: &Database,
        metric: &str,
        version: Version,
    ) -> Result<Bands<T, Rw>> {
        Terms::try_from_fn(|term| {
            ProfitabilityRange::try_from_fn(|band| {
                let band_name = band.select(ProfitabilityRange::names()).id;
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

    pub(crate) fn push(
        &mut self,
        spot: Cents,
        bands: &ProfitabilityRange<Bucket>,
        totals: &Bucket,
    ) {
        for term in TERMS {
            for &band in ProfitabilityRangeId::ALL {
                let bucket = band.select(bands);
                let supply = *term.select(&bucket.supply);
                let capital = *term.select(&bucket.cap);
                band.select_mut(self.supply_stored.select_mut(term))
                    .push(supply);
                band.select_mut(self.capital_stored.select_mut(term))
                    .push(capital);
                band.select_mut(self.net_pnl_stored.select_mut(term))
                    .push(Self::net_pnl(spot, capital, supply));
            }
            self.supply_total
                .select_mut(term)
                .push(*term.select(&totals.supply));
            self.capital_total
                .select_mut(term)
                .push(*term.select(&totals.cap));
        }
    }

    pub(crate) fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        fn bands<T: PcoVecValue>(
            bands: &mut Bands<T, Rw>,
        ) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
            bands
                .iter_mut()
                .flat_map(|term| term.iter_mut())
                .map(|v| v as &mut dyn AnyStoredVec)
        }
        fn totals<T: PcoVecValue>(
            totals: &mut Totals<T, Rw>,
        ) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
            totals.iter_mut().map(|v| v as &mut dyn AnyStoredVec)
        }
        bands(&mut self.supply_stored)
            .chain(bands(&mut self.capital_stored))
            .chain(bands(&mut self.net_pnl_stored))
            .chain(totals(&mut self.supply_total))
            .chain(totals(&mut self.capital_total))
    }

    /// The supply's market value at `spot` minus its capital.
    fn net_pnl(spot: Cents, capital: Cents, supply: Sats) -> CentsSigned {
        let market = CentsSats::from_price_sats(spot, supply).to_cents_rounded();
        CentsSigned::new(market.inner() as i64 - capital.inner() as i64)
    }
}
