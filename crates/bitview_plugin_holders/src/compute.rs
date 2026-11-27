use bitview_cohort::AgeAggregateId;
use bitview_compute::{Inputs, write_chunks, write_locked};
use bitview_plugin::{ComputePlugin, UpdateContext, compute_items};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Height, Version};
use vecdb::{AnyStoredVec, AnyVec, ReadableCloneableVec, ReadableVec};

use crate::{Dependencies, Vecs, day_models, section::Rest, sources::Sources};

impl ComputePlugin for Vecs {
    compute_items!(Dependencies, db);

    fn compute_state(&mut self, deps: Dependencies<'_>, context: UpdateContext<'_>) -> Result<()> {
        let exit = context.exit();
        let sources = Sources::new(deps.age);
        let spot = &deps.price.spot.cents.height;
        let inputs = Inputs::new(
            Version::ONE,
            [spot as &dyn AnyVec].into_iter().chain(sources.vectors()),
        );
        let range = inputs.prepare(self.state_vecs_mut(), usize::from(deps.from), exit)?;
        let (start, end) = (range.start, range.end);
        if end == 0 {
            return Ok(());
        }
        for chunk in write_chunks(range) {
            let rows = sources.collect(chunk.start, chunk.end);
            let spots = spot.collect_range_at(chunk.start, chunk.end);
            for (row, &spot) in rows.iter().zip(&spots) {
                for id in AgeAggregateId::ALL {
                    id.select_mut(&mut self.columns).push(id.select(row), spot);
                }
            }
            write_locked(self.state_vecs_mut(), exit)?;
            // The update's last save flushes once everything is written.
            if chunk.end < end {
                self.db.flush();
            }
        }
        let all_capital = self.columns.all.cap.read_only_boxed_clone();
        self.cohorts.par_try_update(&Rest {
            from: Height::from(start),
            columns: &self.columns,
            all_supply: &self.all_supply,
            all_capital: &all_capital,
            spot,
            exit,
        })?;
        write_locked(self.cohorts.iter_any_stored_mut(), exit)?;
        let all = &self.columns.all;
        self.day_models.compute(
            Height::from(start),
            &day_models::Inputs {
                supply: &all.supply.read_only_boxed_clone(),
                realized_cap: &all.cap.read_only_boxed_clone(),
                spot,
            },
            exit,
        )?;
        debug_assert_eq!(
            self.iter_any_stored_mut().count(),
            self.columns.iter_any_stored_mut().count()
                + self.cohorts.iter_any_stored_mut().count()
                + self.day_models.iter_any_stored_mut().count(),
            "every output is a column, derived from them, or a day model"
        );
        self.db.flush();
        Ok(())
    }
}

impl Vecs {
    /// The block loop's outputs: each cohort's columns. The rest derive from them after the
    /// loop, each validating its own version.
    fn state_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        self.columns.iter_any_stored_mut().collect()
    }
}
