use bitview_plugin_indexer::Indexer;
use bitview_primitives::{PartsPerMillionSigned32, PartsPerMillionSigned64, Ratio64};
use bitview_vecs::ops::Quotient;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::Height;
use vecdb::ReadableVec;

use super::{super::activity, Vecs};

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
        let starting_height = indexer.compute_start().height;

        self.inflation_rate.fixed.height.compute_transform2(
            starting_height,
            &activity.liveliness_to_vaultedness.height,
            inflation_rate,
            |(h, active_to_vaulted, inflation, ..)| {
                let adjusted = f64::from(active_to_vaulted) * f64::from(inflation);
                (h, PartsPerMillionSigned32::from(adjusted))
            },
            exit,
        )?;

        // Velocity of the active supply: velocity over liveliness.
        let liveliness = &activity.liveliness.height;
        self.velocity
            .btc
            .height
            .compute_binary::<_, _, Quotient<Ratio64>>(
                starting_height,
                velocity_btc,
                liveliness,
                exit,
            )?;
        self.velocity
            .usd
            .height
            .compute_binary::<_, _, Quotient<Ratio64>>(
                starting_height,
                velocity_usd,
                liveliness,
                exit,
            )?;

        Ok(())
    }
}
