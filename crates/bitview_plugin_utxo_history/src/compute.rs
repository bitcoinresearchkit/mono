use bitview_compute::prepare_computed;
use bitview_plugin::{ComputePlugin, UpdateContext};
use brk_error::Result;
use brk_types::{Sats, StoredU64, Version};
use vecdb::{AnyStoredVec, WritableVec};

use crate::{Dependencies, Vecs};

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;

    fn compute(
        &mut self,
        dependencies: Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        let Dependencies {
            spends,
            creations,
            from,
            end,
        } = dependencies;
        self.db.sync_bg_tasks()?;
        let end = end.min(creations.len()).min(spends.len());
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
        )?;
        self.history
            .advance(start, end, spends, creations, |_, amount| {
                self.supply.push(Sats::new(amount.sats));
                self.count.height.push(StoredU64::from(amount.count));
                Ok(())
            })?;
        let _lock = context.exit().lock();
        self.supply.write()?;
        self.count.height.write()?;
        self.db.flush()?;
        Ok(())
    }
}
