use bitview_plugin::{ComputePlugin, UpdateContext};
use bitview_primitives::StoredU64;
use brk_error::Result;
use brk_types::Weight;
use vecdb::{Database, ReadableVec, VecIndex};

use super::{Dependencies, Vecs};

const NEAR_FULL_WEIGHT: u32 = 3_600_000;

fn next_streak(weight: Weight, previous: StoredU64) -> StoredU64 {
    if u32::from(weight) >= NEAR_FULL_WEIGHT {
        previous + StoredU64::from(1_u64)
    } else {
        StoredU64::ZERO
    }
}

impl ComputePlugin for Vecs {
    type Dependencies<'a> = Dependencies<'a>;

    fn database(&self) -> Option<&Database> {
        Some(&self.db)
    }

    fn compute_state(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<()> {
        let exit = context.exit();

        self.streak.compute_transform(
            dependencies.safe_height,
            dependencies.block_weights,
            |(height, weight, streak)| {
                let previous = height
                    .to_usize()
                    .checked_sub(1)
                    .and_then(|index| streak.collect_one_at(index))
                    .unwrap_or_default();
                (height, next_streak(weight, previous))
            },
            exit,
        )?;
        Ok(())
    }
}
