use bitview_cohort::AgeAggregateId;
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_primitives::{PartsPerMillion32, PartsPerMillionSigned64};
use bitview_transforms::Quotient;
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyFiatPerBlock, LazyRollingDeltasFiatFromHeight, LazyWindowStartVec, PercentPerBlock,
};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, CentsSigned, Height, Version};
use vecdb::{AnyStoredVec, Database, ReadableBoxedVec, Rw, StorageMode};

use crate::{columns::Columns, part::Part};

/// The mirror of the supply: its outputs valued at their creation price.
#[derive(Traversable)]
pub struct Capital<M: StorageMode = Rw> {
    /// Total capital: the cohort's unspent outputs valued at Bitcoin's spot price when each was
    /// created.
    pub total: LazyFiatPerBlock<Cents>,
    /// Realized cap: the total capital, under its jargon name.
    pub realized_cap: LazyFiatPerBlock<Cents>,
    /// Share of all capital.
    pub share: PercentPerBlock<PartsPerMillion32, M>,
    /// Capital in profit: creation price at or below the current spot price.
    pub in_profit: Part<LazyFiatPerBlock<Cents>, M>,
    /// Capital in loss: creation price above the current spot price.
    pub in_loss: Part<LazyFiatPerBlock<Cents>, M>,
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
            .compute_binary::<_, _, Quotient<PartsPerMillion32>>(from, &c.cap, all_capital, exit)?;
        self.in_profit
            .share
            .compute_binary::<_, _, Quotient<PartsPerMillion32>>(
                from,
                &c.cap_profit,
                &c.cap,
                exit,
            )?;
        self.in_loss
            .share
            .compute_binary::<_, _, Quotient<PartsPerMillion32>>(from, &c.cap_loss, &c.cap, exit)?;
        Ok(())
    }
    pub(crate) fn stored_vecs_mut(&mut self) -> [&mut dyn AnyStoredVec; 3] {
        [
            &mut self.share.fixed.height,
            &mut self.in_profit.share.fixed.height,
            &mut self.in_loss.share.fixed.height,
        ]
    }
}
