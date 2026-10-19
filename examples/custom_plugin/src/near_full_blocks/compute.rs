use bitview_plugin::{ComputePlugin, UpdateContext, compute_items};
use bitview_primitives::Count;
use brk_error::Result;
use vecdb::{ComputeVec, ReadableVec, VecIndex};

use super::{Dependencies, Vecs};

const NEAR_FULL_WEIGHT: u32 = 3_600_000;

impl ComputePlugin for Vecs {
    compute_items!(Dependencies, db);

    fn compute_state(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        let exit = context.exit();
        // Seeded from the stored previous value on the first row, once the vec is validated
        // and truncated, then carried in memory.
        let mut current = None;

        self.streak.compute_transform(
            dependencies.from,
            dependencies.block_weights,
            |(height, weight, streak)| {
                let current =
                    current.get_or_insert_with(|| match height.to_usize().checked_sub(1) {
                        Some(index) => streak
                            .collect_one_at(index)
                            .expect("retained streak prefix"),
                        None => Count::ZERO,
                    });
                *current = if u32::from(weight) >= NEAR_FULL_WEIGHT {
                    *current + Count::new(1)
                } else {
                    Count::ZERO
                };
                *current
            },
            exit,
        )?;
        Ok(())
    }
}
