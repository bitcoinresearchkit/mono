use bitview_distribution::AllChainSources;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{Date, Float64, PriceRatio};
use bitview_vecs::{
    LazyIndexedVec, LazyPerBlock, LazyRatioPerBlock, Price, PriceWithMvrv, PriceWithRatio,
    import_cached,
};
use brk_error::Result;
use brk_types::{Bitcoin, Cents, CentsSigned, Dollars, Height, Sats, Timestamp, Version};
use vecdb::{
    Database, Ident, ImportableVec, PcoVec, ReadableBoxedVec, ReadableCloneableVec,
};

use super::{Vecs, aviv_nupl::AvivNupl};

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        all_chain: &AllChainSources,
        cointime_cap: &impl ReadableCloneableVec<Height, Cents>,
        active_cap: &impl ReadableCloneableVec<Height, Dollars>,
        investor_cap: &impl ReadableCloneableVec<Height, CentsSigned>,
    ) -> Result<Self> {
        let cointime_source = LazyIndexedVec::new(
            "cointime_price_cents_source",
            version,
            cointime_cap,
            &all_chain.supply,
            |_, cap, supply| Cents::from(f64::from(cap) / f64::from(Bitcoin::from(supply))),
        );
        let vaulted_cents = import_cached(db, "vaulted_price_cents", version, mappings)?;
        let active_cents = import_cached(db, "active_price_cents", version, mappings)?;
        let true_market_mean_cents =
            import_cached(db, "true_market_mean_cents", version + Version::ONE, mappings)?;
        let value_days_destroyed_sum = PcoVec::import(db, "value_days_destroyed_sum", version)?;
        let average_cap = PcoVec::import(db, "average_cap", version)?;
        let timestamps = &mappings.timestamp.monotonic;
        // The market's age from the genesis block's day, as RHODL weighs it.
        let genesis = Timestamp::from(Date::from(brk_types::chain().genesis_time()));
        let transferred_cap = LazyIndexedVec::new(
            "transferred_cap",
            version,
            &value_days_destroyed_sum,
            timestamps,
            move |_, sum: Float64, at: Timestamp| over_age(sum, at, genesis),
        );
        let cvdd_source = LazyIndexedVec::new(
            "cvdd_cents_source",
            version,
            &value_days_destroyed_sum,
            timestamps,
            move |_, sum: Float64, at: Timestamp| {
                Cents::from(f64::from(over_age(sum, at, genesis)) * 100.0 / 6_000_000.0)
            },
        );
        let supply = &all_chain.supply;
        let realized_cap = &all_chain.realized_cap;
        let true_market_mean = PriceWithRatio::import(
            db,
            "true_market_mean",
            version,
            &true_market_mean_cents,
            mappings,
        )?;
        Ok(Vecs {
            vaulted: PriceWithMvrv::import(
                db,
                "vaulted_price",
                "vaulted_mvrv",
                version,
                &vaulted_cents,
                mappings,
            )?,
            active: PriceWithMvrv::import(
                db,
                "active_price",
                "active_mvrv",
                version,
                &active_cents,
                mappings,
            )?,
            aviv: LazyPerBlock::from_lazy::<Ident, PriceRatio>(
                "aviv",
                version,
                &true_market_mean.relative.ratio,
            ),
            aviv_nupl: LazyRatioPerBlock::from_ratio::<Dollars, CentsSigned, AvivNupl>(
                "aviv_nupl",
                version,
                active_cap,
                investor_cap,
                mappings,
            ),
            true_market_mean,
            cointime: PriceWithMvrv::import(
                db,
                "cointime_price",
                "cointime_mvrv",
                version,
                &cointime_source,
                mappings,
            )?,
            transferred: per_coin(
                "transferred_price",
                version,
                &transferred_cap,
                1.0,
                supply,
                mappings,
            ),
            balanced: per_coin(
                "balanced_price",
                version,
                &realized_less("balanced_cap", version, realized_cap, &transferred_cap),
                1.0,
                supply,
                mappings,
            ),
            terminal: per_coin("terminal_price", version, &transferred_cap, 21.0, supply, mappings),
            cvdd: Price::from_height_source("cvdd", version, &cvdd_source, mappings),
            delta: per_coin(
                "delta_price",
                version,
                &realized_less("delta_cap", version, realized_cap, &average_cap),
                1.0,
                supply,
                mappings,
            ),
            value_days_destroyed_sum,
            average_cap,
            vaulted_cents,
            active_cents,
            true_market_mean_cents,
        })
    }
}

/// A running sum in USD-days over the market's age in days: a capitalization in USD, undefined
/// until the market has an age.
fn over_age(sum: Float64, at: Timestamp, genesis: Timestamp) -> Dollars {
    if at > genesis {
        Dollars::from(f64::from(sum) / at.days_since(genesis))
    } else {
        Dollars::NAN
    }
}

/// A capitalization in USD, `times` over, per BTC of the supply: a price.
fn per_coin(
    name: &str,
    version: Version,
    cap: &impl ReadableCloneableVec<Height, Dollars>,
    times: f64,
    supply: &ReadableBoxedVec<Height, Sats>,
    mappings: &MappingsVecs,
) -> Price<LazyPerBlock<Cents>> {
    let source = LazyIndexedVec::new(
        &format!("{name}_cents_source"),
        version,
        cap,
        supply,
        move |_, cap: Dollars, supply: Sats| {
            Cents::from(f64::from(cap) * times * 100.0 / f64::from(Bitcoin::from(supply)))
        },
    );
    Price::from_height_source(name, version, &source, mappings)
}

/// Realized capitalization less another, in USD; undefined (NaN) where the difference is negative,
/// as a price below zero is none.
fn realized_less<C: ReadableCloneableVec<Height, Dollars>>(
    name: &str,
    version: Version,
    realized_cap: &ReadableBoxedVec<Height, Cents>,
    cap: &C,
) -> impl ReadableCloneableVec<Height, Dollars> + use<C> {
    LazyIndexedVec::new(name, version, realized_cap, cap, |_, realized: Cents, cap: Dollars| {
        let difference = Dollars::from(realized) - cap;
        if difference < Dollars::ZERO {
            Dollars::NAN
        } else {
            difference
        }
    })
}
