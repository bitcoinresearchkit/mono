use std::ops::Deref;

use bitview_cohort::AgeAggregateId;
use bitview_primitives::{PartsPerMillion32, PartsPerMillionSigned64};
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyFiatPerBlock, LazyRollingDeltasFiatFromHeight, PercentPerBlock, ops::Quotient,
};
use brk_error::Result;
use brk_types::{Cents, CentsSigned, Version};
use vecdb::{BinaryTransform, ComputeVec, Rw, StorageMode};

use crate::{
    part::Part,
    section::{CohortCompute, Imports, Section},
};

/// The mirror of the supply: its outputs valued at their creation price.
#[derive(Traversable)]
pub struct Capital<M: StorageMode = Rw> {
    /// Total capital: the cohort's unspent outputs valued at Bitcoin's spot price when each was
    /// created.
    #[traversable(
        lead = "Capital: what holders paid for these coins, in all, each at the price it last moved at."
    )]
    pub total: LazyFiatPerBlock<Cents>,
    /// Realized cap: the sum of each unspent output's BTC value times the spot price when it was
    /// created.
    #[traversable(
        lead = "Realized cap: what holders paid for their bitcoin, in all, each coin at the price it last moved at."
    )]
    pub realized_cap: LazyFiatPerBlock<Cents>,
    /// Capital in profit: creation-date USD value of the outputs whose creation price is at or
    /// below the current spot price.
    #[traversable(
        lead = "Capital in profit: what holders paid for the coins now at or above their cost."
    )]
    pub in_profit: Part<LazyFiatPerBlock<Cents>, M>,
    /// Capital in loss: creation-date USD value of the outputs whose creation price is above the
    /// current spot price.
    #[traversable(lead = "Capital in loss: what holders paid for the coins now below their cost.")]
    pub in_loss: Part<LazyFiatPerBlock<Cents>, M>,
    /// Change in the cohort's capital over a trailing window.
    #[traversable(lead = "Capital change: the change in what holders paid, in all.")]
    pub delta: LazyRollingDeltasFiatFromHeight<Cents, CentsSigned, PartsPerMillionSigned64>,
}

/// The capital of a cohort but all, beside its share of all capital (all's is 100%).
#[derive(Traversable)]
pub struct SharedCapital<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub capital: Capital<M>,
    /// Share of all capital.
    #[traversable(lead = "Capital share: their part of all the capital.")]
    pub share: PercentPerBlock<PartsPerMillion32, M>,
}

impl Section for Capital {
    fn import(id: AgeAggregateId, cx: &Imports) -> Result<Self> {
        let Imports {
            db,
            v,
            mappings,
            windows,
            ..
        } = *cx;
        let c = id.select(cx.columns);
        let fiat = |metric: &str, source| {
            LazyFiatPerBlock::from_cents_source(&id.metric_name(metric), v, source, mappings)
        };
        let part = |metric: &str, source| {
            Part::import(
                db,
                &id.metric_name(metric),
                v,
                mappings,
                fiat(metric, source),
            )
        };
        Ok(Self {
            total: fiat("capital", &c.cap),
            realized_cap: fiat("realized_cap", &c.cap),
            in_profit: part("capital_in_profit", &c.cap_profit)?,
            in_loss: part("capital_in_loss", &c.cap_loss)?,
            delta: LazyRollingDeltasFiatFromHeight::new(
                &id.metric_name("capital_delta"),
                v + Version::TWO,
                &c.cap,
                windows,
                mappings,
            ),
        })
    }
    fn compute(&mut self, cx: &CohortCompute) -> Result<()> {
        let CohortCompute { from, c, exit, .. } = *cx;
        self.in_profit
            .share
            .fixed
            .height
            .compute_binary::<_, _, SideShare>(from, &c.cap_profit, &c.cap_loss, exit)?;
        self.in_loss
            .share
            .fixed
            .height
            .compute_binary::<_, _, SideShare>(from, &c.cap_loss, &c.cap_profit, exit)?;
        Ok(())
    }
}

impl<M: StorageMode> Deref for SharedCapital<M> {
    type Target = Capital<M>;
    fn deref(&self) -> &Capital<M> {
        &self.capital
    }
}

impl Section for SharedCapital {
    fn import(id: AgeAggregateId, cx: &Imports) -> Result<Self> {
        Ok(Self {
            capital: Capital::import(id, cx)?,
            share: PercentPerBlock::import(
                cx.db,
                &id.metric_name("capital_share"),
                cx.v,
                cx.mappings,
            )?,
        })
    }
    fn compute(&mut self, cx: &CohortCompute) -> Result<()> {
        self.capital.compute(cx)?;
        self.share
            .fixed
            .height
            .compute_binary::<_, _, Quotient<PartsPerMillion32>>(
                cx.from,
                &cx.c.cap,
                cx.all_capital,
                cx.exit,
            )?;
        Ok(())
    }
}

/// One side's share of both sides together. The sides sum the bands' capitals, each rounded
/// once, so their total can differ from the exact `cap` by a few cents: dividing by it could
/// exceed 100%, dividing by the sides' own total keeps each share within it.
struct SideShare;

impl BinaryTransform<Cents, Cents, PartsPerMillion32> for SideShare {
    #[inline(always)]
    fn apply(side: Cents, other: Cents) -> PartsPerMillion32 {
        Quotient::apply(side, side + other)
    }
}
