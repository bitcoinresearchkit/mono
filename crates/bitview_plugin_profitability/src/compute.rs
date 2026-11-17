use bitview_compute::save_stamped;
use bitview_distribution::{
    state::cost_basis::{PRICE_INDEX_VERSION, age_index::AgeIndex},
    walk::{Block, Consumer, Origins, Sources},
};
use bitview_plugin::UpdateContext;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Height, Version};

use crate::{
    Vecs,
    live::{LiveState, cost_basis, ranges},
};

/// The plugin as a consumer of the walk: every series it reads from the one price index.
pub struct ProfitabilityConsumer<'a> {
    vecs: &'a mut Vecs,
    context: UpdateContext<'a>,
    start: usize,
    live: LiveState,
    kept: bool,
}

impl Vecs {
    /// Readies the plugin for a walk over `sources`, from `from` at most: its outputs validated
    /// and cut to where they resume, the index the last walk left taken.
    pub fn consumer<'a>(
        &'a mut self,
        sources: Sources<'_>,
        from: Height,
        context: UpdateContext<'a>,
    ) -> Result<ProfitabilityConsumer<'a>> {
        self.db.sync_bg_tasks()?;
        // A version reset must invalidate the resident state even if validation fails.
        let live = self.live.take();
        let inputs = sources.inputs(Version::TWO + PRICE_INDEX_VERSION);
        let start = inputs
            .prepare(
                self.iter_any_stored_mut().collect::<Vec<_>>(),
                usize::from(from),
                context.exit(),
            )?
            .start;
        let live = live.filter(|live| live.version == inputs.version);
        Ok(ProfitabilityConsumer {
            context,
            start,
            kept: live.is_some(),
            live: live.unwrap_or_else(|| LiveState {
                index: AgeIndex::default(),
                version: inputs.version,
            }),
            vecs: self,
        })
    }
}

impl Consumer for ProfitabilityConsumer<'_> {
    fn start(&self) -> usize {
        self.start
    }

    fn kept(&self) -> bool {
        self.kept
    }

    fn restore(&mut self, origins: &Origins<'_>) -> Result<()> {
        self.live.index.restore(origins)
    }

    fn apply(&mut self, block: &Block<'_>) -> Result<()> {
        let index = &mut self.live.index;
        index.apply(block)?;
        let (bands, totals) = ranges(&index.index, block.price());
        self.vecs.metrics.push(&bands, &totals);
        self.vecs.cost_basis.push(cost_basis(index, block));
        Ok(())
    }

    fn save(&mut self, end: usize) -> Result<()> {
        let Vecs {
            db,
            metrics,
            cost_basis,
            ..
        } = &mut *self.vecs;
        save_stamped(
            db,
            metrics
                .iter_any_stored_mut()
                .chain(cost_basis.iter_any_stored_mut()),
            end,
            true,
            self.context.exit(),
        )
    }
}

impl ProfitabilityConsumer<'_> {
    /// The walk completed: the plugin keeps the index for the next one.
    pub fn keep(self) {
        self.vecs.live = Some(self.live);
        self.context.compact_database(&self.vecs.db);
    }
}
