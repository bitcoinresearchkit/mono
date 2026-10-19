use bitview_cohort::AgeAggregateId;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_primitives::{PartsPerMillion32, PartsPerMillionSigned64};
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyFiatPerBlock, LazyRollingDeltasFiatFromHeight, LazyWindowStartVec, PercentPerBlock,
    ops::Quotient, shapes::Windows,
};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, CentsSigned, Height, Version};
use vecdb::{BinaryTransform, ComputeVec, Database, ReadableBoxedVec, Rw, StorageMode};

use crate::{columns::Columns, part::Part};

/// The mirror of the supply: its outputs valued at their creation price.
#[derive(Traversable)]
pub struct Capital<M: StorageMode = Rw> {
    /// Total capital: the cohort's unspent outputs valued at Bitcoin's spot price when each was
    /// created.
    pub total: LazyFiatPerBlock<Cents>,
    /// Realized cap: the sum of each unspent output's BTC value times the spot price when it was
    /// created.
    pub realized_cap: LazyFiatPerBlock<Cents>,
    /// Share of all capital.
    pub share: PercentPerBlock<PartsPerMillion32, M>,
    /// Capital in profit: creation-date USD value of the outputs whose creation price is at or
    /// below the current spot price.
    pub in_profit: Part<LazyFiatPerBlock<Cents>, M>,
    /// Capital in loss: creation-date USD value of the outputs whose creation price is above the
    /// current spot price.
    pub in_loss: Part<LazyFiatPerBlock<Cents>, M>,
    /// Change in the cohort's capital over a trailing window.
    pub delta: LazyRollingDeltasFiatFromHeight<Cents, CentsSigned, PartsPerMillionSigned64>,
}
impl Capital {
    pub(crate) fn import(
        db: &Database,
        id: AgeAggregateId,
        v: Version,
        c: &Columns,
        mappings: &Mappings,
        windows: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
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
            share: PercentPerBlock::import(db, &id.metric_name("capital_share"), v, mappings)?,
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
    pub(crate) fn compute(
        &mut self,
        from: Height,
        c: &Columns,
        all_capital: &ReadableBoxedVec<Height, Cents>,
        exit: &Exit,
    ) -> Result<()> {
        self.share
            .fixed
            .height
            .compute_binary::<_, _, Quotient<PartsPerMillion32>>(from, &c.cap, all_capital, exit)?;
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
