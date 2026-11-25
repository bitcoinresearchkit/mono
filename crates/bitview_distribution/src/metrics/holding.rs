//! A holder group's layout, shared by the groups (holders) and their weighted coins (awake,
//! mobile): one tree, one set of names and descriptions. `S` is a part's share, stored or read.

use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{PartsPerMillionSigned32, PartsPerMillionSigned64};
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyFiatPerBlock, LazyIndexedVec, LazyRatioPerBlock, LazyRollingDeltasAmountFromHeight,
    LazyRollingDeltasFiatFromHeight, LazySpotValuePerBlock,
};
use brk_types::{Cents, CentsSigned, Height, Sats, SatsSigned, Version};
use vecdb::ReadableCloneableVec;

/// A part of a cohort quantity and its share of the cohort's whole.
#[derive(Clone, Traversable)]
pub struct Part<T, S> {
    #[traversable(flatten)]
    pub value: T,
    /// Share of the cohort's total (for capital, of the in-profit and in-loss sides together,
    /// which keeps each within 100%); undefined (null) when that total is zero.
    pub share: S,
}

#[derive(Clone, Traversable)]
pub struct Supply<S> {
    /// Total unspent supply.
    #[traversable(lead = "Supply: the bitcoin held.")]
    pub total: LazySpotValuePerBlock,
    /// Supply in profit: creation price at or below the spot price. Its client type and its side's
    /// keep the names they were first published under (named here, not by the shapes around them).
    #[traversable(priced)]
    #[traversable(
        client = "SupplyInProfit",
        lead = "Supply in profit: coins whose price now is at or above the one they last moved at."
    )]
    pub in_profit: Part<LazySpotValuePerBlock, S>,
    /// Supply in loss: creation price above the spot price.
    #[traversable(priced)]
    #[traversable(
        client = "AllSupplyInLoss",
        lead = "Supply in loss: coins whose price now is below the one they last moved at."
    )]
    pub in_loss: Part<LazySpotValuePerBlock, S>,
    /// Change in the cohort's supply over a trailing window.
    #[traversable(lead = "Supply change: how much the bitcoin held changed.")]
    pub delta: LazyRollingDeltasAmountFromHeight<Sats, SatsSigned, PartsPerMillionSigned64>,
}

/// The mirror of the supply: its outputs valued at their creation price.
#[derive(Clone, Traversable)]
pub struct Capital<S> {
    /// Total capital: the cohort's unspent outputs valued at Bitcoin's spot price when each was
    /// created.
    #[traversable(
        lead = "Capital: what these coins cost, in all, each at the price it last moved at."
    )]
    pub total: LazyFiatPerBlock<Cents>,
    /// Realized cap: the sum of each unspent output's BTC value times the spot price when it was
    /// created.
    #[traversable(
        lead = "Realized cap: what the bitcoin held cost, in all, each coin at the price it last moved at."
    )]
    pub realized_cap: LazyFiatPerBlock<Cents>,
    /// Capital in profit: creation-date USD value of the outputs whose creation price is at or
    /// below the current spot price.
    #[traversable(lead = "Capital in profit: what the coins now in profit cost, in all.")]
    pub in_profit: Part<LazyFiatPerBlock<Cents>, S>,
    /// Capital in loss: creation-date USD value of the outputs whose creation price is above the
    /// current spot price.
    #[traversable(lead = "Capital in loss: what the coins now at a loss cost, in all.")]
    pub in_loss: Part<LazyFiatPerBlock<Cents>, S>,
    /// Change in the cohort's capital over a trailing window.
    #[traversable(lead = "Capital change: the change in what the bitcoin held cost, in all.")]
    pub delta: LazyRollingDeltasFiatFromHeight<Cents, CentsSigned, PartsPerMillionSigned64>,
}

#[derive(Clone, Traversable)]
pub struct Unrealized {
    /// Unrealized profit of the cohort's unspent outputs: market value at the
    /// represented block minus creation-date value, summed where spot is above
    /// creation price.
    #[traversable(
        lead = "Unrealized profit: holders' paper gains, on coins worth more than they cost."
    )]
    pub profit: LazyFiatPerBlock<Cents>,
    /// Unrealized loss of the cohort's unspent outputs: creation-date value
    /// minus market value at the represented block, summed where spot is below
    /// creation price.
    #[traversable(
        lead = "Unrealized loss: holders' paper losses, on coins worth less than they cost."
    )]
    pub loss: LazyFiatPerBlock<Cents>,
    /// Net unrealized profit and loss of the cohort: unrealized profit minus
    /// unrealized loss.
    #[traversable(lead = "Net unrealized profit and loss: paper gains minus paper losses.")]
    pub net_pnl: LazyFiatPerBlock<CentsSigned>,
    /// Gross unrealized profit and loss: unrealized profit plus unrealized loss.
    #[traversable(lead = "Gross unrealized profit and loss: paper gains plus paper losses.")]
    pub gross_pnl: LazyFiatPerBlock<Cents>,
    /// Net unrealized profit/loss (NUPL): net unrealized profit and loss divided by the
    /// cohort's market cap: at most 1, with no lower bound (1 - 1/MVRV in exact arithmetic,
    /// though the two round and are undefined apart); undefined (null) when that market cap is
    /// zero.
    #[traversable(
        lead = "NUPL (net unrealized profit and loss): paper gains minus paper losses, as a share of the market cap. Near 1, almost all of it is profit; below 0, losses outweigh gains."
    )]
    pub nupl: LazyRatioPerBlock<PartsPerMillionSigned32>,
}

impl Unrealized {
    /// Views over stored `profit` and `loss` and a `nupl` source; net and gross are profit and
    /// loss's difference and sum, read, not stored. `name` gives each id from its metric.
    pub fn new(
        name: impl Fn(&str) -> String,
        version: Version,
        profit: &impl ReadableCloneableVec<Height, Cents>,
        loss: &impl ReadableCloneableVec<Height, Cents>,
        nupl: &impl ReadableCloneableVec<Height, PartsPerMillionSigned32>,
        mappings: &MappingsVecs,
    ) -> Self {
        let fiat = |metric: &str, source: &dyn ReadableCloneableVec<Height, Cents>| {
            LazyFiatPerBlock::from_cents_source(&name(metric), version, source, mappings)
        };
        let net_pnl =
            net_pnl_source(&name("net_unrealized_pnl_cents_source"), version, profit, loss);
        let gross_pnl = LazyIndexedVec::new(
            &name("gross_unrealized_pnl_cents_source"),
            version,
            profit,
            loss,
            |_, profit: Cents, loss: Cents| profit + loss,
        );
        Self {
            profit: fiat("unrealized_profit", profit),
            loss: fiat("unrealized_loss", loss),
            net_pnl: LazyFiatPerBlock::from_cents_source(
                &name("net_unrealized_pnl"),
                version,
                &net_pnl,
                mappings,
            ),
            gross_pnl: LazyFiatPerBlock::from_cents_source(
                &name("gross_unrealized_pnl"),
                version,
                &gross_pnl,
                mappings,
            ),
            nupl: LazyRatioPerBlock::from_height_source(&name("nupl"), version, nupl, mappings),
        }
    }
}

/// Unrealized profit minus unrealized loss, read from both.
pub fn net_pnl_source<P, L>(
    name: &str,
    version: Version,
    profit: &P,
    loss: &L,
) -> impl ReadableCloneableVec<Height, CentsSigned> + use<P, L>
where
    P: ReadableCloneableVec<Height, Cents>,
    L: ReadableCloneableVec<Height, Cents>,
{
    LazyIndexedVec::new(name, version, profit, loss, |_, profit: Cents, loss: Cents| {
        CentsSigned::from_difference(profit, loss)
    })
}
