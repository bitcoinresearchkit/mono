use bitview_plugin::{ComputePlugin, UpdateContext};
use brk_error::Result;
use brk_types::{StoredU64, Weight};
use vecdb::{ReadableVec, VecIndex};

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
    type Output = ();

    fn compute(
        &mut self,
        dependencies: Self::Dependencies<'_>,
        context: UpdateContext<'_>,
    ) -> Result<Self::Output> {
        let exit = context.exit();
        self.db.sync_bg_tasks()?;

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

        let exit = exit.clone();
        self.db.run_bg(move |db| {
            let _lock = exit.lock();
            db.compact_deferred_default()
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_only_consecutive_near_full_blocks() {
        assert_eq!(
            next_streak(Weight::from(NEAR_FULL_WEIGHT), StoredU64::ZERO),
            StoredU64::from(1_u64)
        );
        assert_eq!(
            next_streak(Weight::from(NEAR_FULL_WEIGHT + 1), StoredU64::from(4_u64)),
            StoredU64::from(5_u64)
        );
        assert_eq!(
            next_streak(Weight::from(NEAR_FULL_WEIGHT - 1), StoredU64::from(5_u64)),
            StoredU64::ZERO
        );
    }
}
