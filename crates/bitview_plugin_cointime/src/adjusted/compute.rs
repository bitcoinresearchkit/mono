use brk_error::Result;

use bitview_plugin_indexer::Indexer;
use brk_types::{Height, PartsPerMillionSigned64, StoredF64};
use vecdb::{Exit, ReadableVec};

use super::super::activity;
use super::Vecs;

pub fn compute(
    vecs: &mut Vecs,
    indexer: &Indexer,
    inflation_rate: &impl ReadableVec<Height, PartsPerMillionSigned64>,
    velocity_native: &impl ReadableVec<Height, StoredF64>,
    velocity_fiat: &impl ReadableVec<Height, StoredF64>,
    activity: &activity::Vecs,
    exit: &Exit,
) -> Result<()> {
    vecs.compute(
        indexer,
        inflation_rate,
        velocity_native,
        velocity_fiat,
        activity,
        exit,
    )
}

impl Vecs {
    fn compute(
        &mut self,
        indexer: &Indexer,
        inflation_rate: &impl ReadableVec<Height, PartsPerMillionSigned64>,
        velocity_native: &impl ReadableVec<Height, StoredF64>,
        velocity_fiat: &impl ReadableVec<Height, StoredF64>,
        activity: &activity::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;

        self.inflation_rate.ppm.height.compute_transform2(
            starting_height,
            &activity.ratio.height,
            inflation_rate,
            |(h, a2vr, inflation, ..)| {
                (
                    h,
                    PartsPerMillionSigned64::from(f64::from(a2vr) * f64::from(inflation)),
                )
            },
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
