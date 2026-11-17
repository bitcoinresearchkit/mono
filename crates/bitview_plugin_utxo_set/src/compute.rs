use bitview_compute::prepare_computed;
use bitview_plugin::{ComputePlugin, UpdateContext, compute_items};
use bitview_primitives::Count;
use brk_error::Result;
use brk_types::{Sats, Version};
use vecdb::{AnyStoredVec, WritableVec};

use crate::{Dependencies, Vecs};

impl ComputePlugin for Vecs {
    compute_items!(Dependencies, db);

    fn compute_state(
        &mut self,
        dependencies: Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        let Dependencies {
            spends,
            creations,
            price,
            mappings,
            from,
            end,
        } = dependencies;
        let end = end.min(creations.end()).min(spends.end());
        let version = Version::TWO
            + Version::from(creations.version() as u32)
            + Version::from(spends.version() as u32);
        let start = prepare_computed(
            [
                &mut self.supply as &mut dyn AnyStoredVec,
                &mut self.count.height,
            ],
            version,
            usize::from(from).min(end),
            context.exit(),
        )?;
        let _lock = context.exit().lock();
        self.history
            .advance(start, end, spends, creations, |_, amount| {
                self.supply.push(Sats::new(amount.sats));
                self.count.height.push(Count::from(amount.count));
                Ok(())
            })?;
        self.urpd.advance(
            start,
            &self.history.reader(spends, creations)?,
            &price.spot.cents.height,
            &mappings.timestamp.monotonic,
        )?;
        self.supply.write()?;
        self.count.height.write()?;
        self.db.flush();
        Ok(())
    }
}
