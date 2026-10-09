use bitview_plugin_indexer::Indexer;
use bitview_primitives::{PartsPerMillionSigned32, PartsPerMillionSigned64, Ratio64};
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
        velocity_native: &impl ReadableVec<Height, Ratio64>,
        velocity_fiat: &impl ReadableVec<Height, Ratio64>,
        activity: &activity::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;

        self.inflation_rate.fixed.height.compute_transform2(
            starting_height,
            &activity.ratio.height,
            inflation_rate,
            |(h, a2vr, inflation, ..)| (h, adjusted_inflation(a2vr, inflation)),
            exit,
        )?;

        self.tx_velocity_native.height.compute_multiply(
            starting_height,
            &activity.ratio.height,
            velocity_native,
            exit,
        )?;

        self.tx_velocity_fiat.height.compute_multiply(
            starting_height,
            &activity.ratio.height,
            velocity_fiat,
            exit,
        )?;

        Ok(())
    }
}
