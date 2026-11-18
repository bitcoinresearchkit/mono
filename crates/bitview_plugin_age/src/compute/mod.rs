mod price_range_max;
pub use price_range_max::PriceRangeMax;

use bitview_distribution::{
    state::RealizedOps,
    walk::{Block, Consumer, Origins, Sources},
};
use bitview_plugin::UpdateContext;
use bitview_primitives::CoinBlocks;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{CentsSats, Height, Sats, Timestamp, Version};
use rayon::prelude::*;
use vecdb::WritableVec;

use crate::{
    Vecs,
    live::LiveState,
    state::{UTXOStates, supply, tick_tock},
};
use bitview_cohort::creation_cohorts_ahead;
use tracing::warn;

/// The plugin as a consumer of the walk: every age range's, halving epoch's and creation year's
/// series.
pub struct AgeConsumer<'a> {
    vecs: &'a mut Vecs,
    context: UpdateContext<'a>,
    start: usize,
    live: LiveState,
    kept: bool,
    /// The last block applied.
    tip: Option<(Height, Timestamp)>,
}

impl Vecs {
    /// Readies the plugin for a walk over `sources`, from `from` at most: its outputs validated
    /// and cut to where they resume, the cohort states the last walk left taken.
    pub fn consumer<'a>(
        &'a mut self,
        sources: Sources<'_>,
        from: Height,
        context: UpdateContext<'a>,
    ) -> Result<AgeConsumer<'a>> {
        self.db.sync_bg_tasks()?;
        // A version reset must invalidate the resident state even if validation fails.
        let live = self.live.take();
        let inputs = sources.inputs(Version::ONE);
        let start = inputs
            .prepare(
                self.iter_any_stored_mut().collect::<Vec<_>>(),
                usize::from(from),
                context.exit(),
            )?
            .start;
        let live = live.filter(|live| live.version == inputs.version);
        Ok(AgeConsumer {
            context,
            start,
            kept: live.is_some(),
            live: live.unwrap_or_else(|| LiveState {
                states: UTXOStates::new(),
                max: PriceRangeMax::default(),
                version: inputs.version,
            }),
            tip: None,
            vecs: self,
        })
    }
}

impl Consumer for AgeConsumer<'_> {
    fn start(&self) -> usize {
        self.start
    }

    fn kept(&self) -> bool {
        self.kept
    }

    fn restore(&mut self, origins: &Origins<'_>) -> Result<()> {
        // The price maxima too: a kept table can hold prices of blocks since replaced.
        self.live.max = PriceRangeMax::default();
        self.live.states = UTXOStates::new();
        self.live.states.restore_origins(origins);
        Ok(())
    }

    fn apply(&mut self, block: &Block<'_>) -> Result<()> {
        let LiveState { states, max, .. } = &mut self.live;
        // Every price of the walk at once: a no-op from its second block on.
        max.extend(block.prices);
        let height = Height::from(block.height);
        let (timestamp, price) = (block.timestamp(), block.price());
        let vecs = &mut *self.vecs;

        let tick = tick_tock(states, block);
        for ((range, &aged_out), &coindays_created) in vecs
            .ranges
            .iter_mut()
            .zip(tick.aged_out.iter())
            .zip(tick.coindays_created.iter())
        {
            range.push_tick(aged_out, coindays_created, price);
        }
        states.receive_origins(supply(block.diff.created), height, timestamp, price);
        let satblocks = states.send_origins(block, max);
        vecs.coinblocks_destroyed.push_block(CoinBlocks::from(
            satblocks as f64 / Sats::ONE_BTC_U128 as f64,
        ));
        if let Some((origin, amount)) = block.diff.correction() {
            states.lose_origin(Height::new(origin), supply(amount), block);
        }

        states.cross(block);

        // Every cohort's values once the block's spends and receives are applied, and the
        // capital of all ranges together, summed exactly and converted to cents once.
        let mut all_capital = CentsSats::ZERO;
        for (range, state) in vecs.ranges.iter_mut().zip(states.age_range.iter_mut()) {
            range.push(state, price);
            all_capital += state.realized.cap_raw();
        }
        vecs.all_capital.push(all_capital.to_cents());
        for (epoch, state) in (vecs.halving_epochs.iter_mut()).zip(states.halving_epoch.iter_mut())
        {
            epoch.push(state, price);
        }
        for (vintage, state) in vecs.vintages.iter_mut().zip(states.vintage.iter_mut()) {
            vintage.push(state, price);
        }

        states.reset_block();
        self.tip = Some((height, timestamp));
        Ok(())
    }

    /// In parallel, which `write_locked` is not: the plugin has over a thousand outputs.
    fn save(&mut self) -> Result<()> {
        let _lock = self.context.exit().lock();
        self.vecs
            .iter_any_stored_mut()
            .collect::<Vec<_>>()
            .into_par_iter()
            .try_for_each(|v| v.write().map(drop))?;
        self.vecs.db.flush();
        Ok(())
    }
}

impl AgeConsumer<'_> {
    /// The walk completed: the plugin keeps the cohort states for the next one.
    pub fn keep(self) {
        if let Some((tip, timestamp)) = self.tip
            && let Some(warning) = creation_cohorts_ahead(tip, timestamp)
        {
            warn!("{warning}");
        }
        self.vecs.live = Some(self.live);
        self.context.compact_database(&self.vecs.db);
    }
}
