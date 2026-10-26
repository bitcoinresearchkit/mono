use std::ops::Deref;

use bitview_cohort::AgeAggregateId;
use bitview_primitives::Multiple;
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyFiatPerBlockCumulativeWithSums, LazyIndexedVec, RollingWindows, ops::Quotient,
};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height};
use vecdb::{ComputeVec, Rw, StorageMode};

use crate::{
    realized::Realized,
    section::{CohortCompute, Imports, Section},
};

/// The realized flows of a cohort holding outputs under an hour old, beside the SOPR without
/// them (the others' equals their SOPR).
#[derive(Traversable)]
pub struct AdjustedRealized<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub realized: Realized<M>,
    /// Adjusted SOPR (aSOPR): SOPR without the outputs spent within an hour of their creation.
    #[traversable(lead = "Adjusted SOPR (aSOPR): SOPR without the coins that moved again within an hour.")]
    pub adjusted_sopr: AdjustedSopr<M>,
}

impl<M: StorageMode> Deref for AdjustedRealized<M> {
    type Target = Realized<M>;
    fn deref(&self) -> &Realized<M> {
        &self.realized
    }
}

impl Section for AdjustedRealized {
    fn import(id: AgeAggregateId, cx: &Imports) -> Result<Self> {
        Ok(Self {
            realized: Realized::import(id, cx)?,
            adjusted_sopr: AdjustedSopr::import(id, cx)?,
        })
    }
    fn compute(&mut self, cx: &CohortCompute) -> Result<()> {
        self.realized.compute(cx)?;
        self.adjusted_sopr.compute(cx.from, cx.exit)
    }
}

#[derive(Traversable)]
pub struct AdjustedSopr<M: StorageMode = Rw> {
    /// Spend-time value in US dollars of the outputs it keeps over the window divided by their
    /// creation-time value; undefined (null) when that value is zero.
    #[traversable(flatten)]
    pub ratio: RollingWindows<Multiple, M>,
    /// Spend-time value of the outputs it keeps (spent an hour or more after creation).
    pub value_created: LazyFiatPerBlockCumulativeWithSums<Cents>,
    /// Creation-time value of the outputs it keeps (spent an hour or more after creation).
    pub value_destroyed: LazyFiatPerBlockCumulativeWithSums<Cents>,
}
impl AdjustedSopr {
    fn import(id: AgeAggregateId, cx: &Imports) -> Result<Self> {
        let Imports {
            db,
            v,
            mappings,
            windows,
            ..
        } = *cx;
        let c = id.select(cx.columns);
        // Every group with aSOPR holds the under-an-hour band, and both sum the bands' cumulative
        // cents: the outputs it keeps are exactly the group's less that band's.
        let kept = |metric: &str, total, under_1h| {
            let name = id.metric_name(metric);
            let cumulative = LazyIndexedVec::new(
                &format!("{name}_cumulative_cents_source"),
                v,
                total,
                under_1h,
                |_, total: Cents, under_1h: Cents| total - under_1h,
            );
            LazyFiatPerBlockCumulativeWithSums::from_cumulative_cents_source(
                &name,
                v,
                &cumulative,
                mappings,
                windows,
            )
        };
        Ok(Self {
            ratio: RollingWindows::import(db, &id.metric_name("adjusted_sopr"), v, mappings)?,
            value_created: kept(
                "adjusted_value_created",
                &c.volume_cents,
                cx.under_1h_volume,
            ),
            value_destroyed: kept(
                "adjusted_value_destroyed",
                &c.value_destroyed,
                cx.under_1h_value_destroyed,
            ),
        })
    }
    fn compute(&mut self, from: Height, exit: &Exit) -> Result<()> {
        for ((target, created), destroyed) in self
            .ratio
            .as_mut_array()
            .into_iter()
            .zip(self.value_created.sum.as_array())
            .zip(self.value_destroyed.sum.as_array())
        {
            target.height.compute_binary::<_, _, Quotient<Multiple>>(
                from,
                &created.cents.height,
                &destroyed.cents.height,
                exit,
            )?;
        }
        Ok(())
    }
}
