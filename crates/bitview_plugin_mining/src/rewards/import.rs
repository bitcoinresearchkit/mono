use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{Float, Multiple, PartsPerMillion32};
use bitview_vecs::{
    LazyIndexedVec, LazyPerBlock, LazyPercentCumulativeRolling, LazyWindowStartVec, Price,
    ValuePerBlockCumulativeRolling, ValuePerBlockFull,
    ops::{OneMinusPpm, Quotient},
    shapes::Windows,
};
use brk_error::Result;
use brk_types::{Cents, Sats, Version};
use vecdb::{Database, Ident, ReverseOperands};

use super::Vecs;

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let coinbase = ValuePerBlockCumulativeRolling::import(
            db,
            "coinbase",
            version,
            mappings,
            window_starts,
        )?;
        let subsidy = ValuePerBlockCumulativeRolling::import(
            db,
            "subsidy",
            version,
            mappings,
            window_starts,
        )?;
        let fees = ValuePerBlockFull::import(db, "fees", version, mappings, window_starts)?;
        let puell_source = LazyIndexedVec::new(
            "puell_multiple_source",
            version,
            &subsidy.sum._24h.cents.height,
            &subsidy.sum._1y.cents.height,
            |_, day: Cents, year: Cents| Multiple::from(365.0 * f64::from(day) / f64::from(year)),
        );
        let puell_multiple = LazyPerBlock::from_height_source::<Ident>(
            "puell_multiple",
            version,
            &puell_source,
            mappings,
        );

        // PlanB's stock and flow: every coin mined so far, and the past year's.
        let stock_to_flow_source = LazyIndexedVec::new(
            "stock_to_flow_source",
            version,
            subsidy.cumulative.sats.resolutions.height_source(),
            &subsidy.sum._1y.sats.height,
            |_, stock: Sats, flow: Sats| Float::from(stock_to_flow(stock, flow)),
        );
        let stock_to_flow = LazyPerBlock::from_height_source::<Ident>(
            "stock_to_flow",
            version,
            &stock_to_flow_source,
            mappings,
        );
        let stock_to_flow_price = Price::from_height_source(
            "stock_to_flow_price",
            version,
            &LazyIndexedVec::new(
                "stock_to_flow_price_cents_source",
                version,
                subsidy.cumulative.sats.resolutions.height_source(),
                &subsidy.sum._1y.sats.height,
                |_, stock: Sats, flow: Sats| Cents::from(planb_price_cents(stock, flow)),
            ),
            mappings,
        );

        // Claimed subsidy over coinbase, so an underclaimed coinbase stays within 0..=100%;
        // the coinbase is read first.
        let subsidy_share = LazyPercentCumulativeRolling::from_cumulative_ratio::<
            Sats,
            Sats,
            ReverseOperands<Quotient<PartsPerMillion32>>,
        >(
            "subsidy_share",
            version,
            coinbase.cumulative.sats.resolutions.height_source(),
            subsidy.cumulative.sats.resolutions.height_source(),
            window_starts,
            mappings,
        );
        let fee_share = LazyPercentCumulativeRolling::from_lazy_source::<OneMinusPpm>(
            "fee_share",
            version,
            &subsidy_share,
        );
        Ok(Vecs {
            coinbase,
            subsidy,
            puell_multiple,
            stock_to_flow,
            stock_to_flow_price,
            fees,
            unclaimed: ValuePerBlockCumulativeRolling::import(
                db,
                "unclaimed_rewards",
                version,
                mappings,
                window_starts,
            )?,
            fee_share,
            subsidy_share,
        })
    }
}

/// The coins mined so far over the past year's; NaN while that year reaches back to the first
/// block (2009): its flow is then every coin mined, not a year's.
fn stock_to_flow(stock: Sats, flow: Sats) -> f64 {
    if flow >= stock {
        return f64::NAN;
    }
    f64::from(stock) / f64::from(flow)
}

/// PlanB's model price in cents (PlanB, 2019, "Modeling Bitcoin Value with Scarcity"): market value
/// `exp(14.6) · SF^3.3` dollars, his published coefficients, over the stock.
fn planb_price_cents(stock: Sats, flow: Sats) -> f64 {
    let market_value = 14.6f64.exp() * stock_to_flow(stock, flow).powf(3.3);
    market_value / (f64::from(stock) / 100_000_000.0) * 100.0
}
