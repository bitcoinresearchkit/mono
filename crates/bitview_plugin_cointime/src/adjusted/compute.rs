use bitview_plugin_indexer::Indexer;
use bitview_primitives::{PartsPerMillionSigned32, PartsPerMillionSigned64, Ratio64};
use bitview_vecs::PerBlock;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::Height;
use vecdb::ReadableVec;

use super::{super::activity, Vecs};

#[inline]
fn adjusted_inflation(
    active_to_vaulted: Ratio64,
    inflation: PartsPerMillionSigned64,
) -> PartsPerMillionSigned32 {
    PartsPerMillionSigned32::from(f64::from(active_to_vaulted) * f64::from(inflation))
}

impl Vecs {
    pub(crate) fn compute(
        &mut self,
        indexer: &Indexer,
        inflation_rate: &impl ReadableVec<Height, PartsPerMillionSigned64>,
        velocity_btc: &impl ReadableVec<Height, Ratio64>,
        velocity_usd: &impl ReadableVec<Height, Ratio64>,
        activity: &activity::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;

        self.inflation_rate.fixed.height.compute_transform2(
            starting_height,
            &activity.liveliness_to_vaultedness.height,
            inflation_rate,
            |(h, a2vr, inflation, ..)| (h, adjusted_inflation(a2vr, inflation)),
            exit,
        )?;

        // Velocity of the active supply: velocity over liveliness.
        active_velocity(
            &mut self.velocity.btc,
            velocity_btc,
            activity,
            starting_height,
            exit,
        )?;
        active_velocity(
            &mut self.velocity.usd,
            velocity_usd,
            activity,
            starting_height,
            exit,
        )?;

        Ok(())
    }
}

fn active_velocity(
    target: &mut PerBlock<Ratio64>,
    velocity: &impl ReadableVec<Height, Ratio64>,
    activity: &activity::Vecs,
    starting_height: Height,
    exit: &Exit,
) -> Result<()> {
    target.height.compute_transform2(
        starting_height,
        velocity,
        &activity.liveliness.height,
        |(h, velocity, liveliness, ..)| {
            (
                h,
                Ratio64::from(f64::from(velocity) / f64::from(liveliness)),
            )
        },
        exit,
    )?;
    Ok(())
}
