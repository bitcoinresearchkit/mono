use bitview_primitives::Float;
use vecdb::WritableVec;

use super::Vecs;
use crate::model::PrimaryValues;

impl Vecs {
    pub(crate) fn push(&mut self, values: &PrimaryValues) {
        for (((range, spending_rate), spending_exposure), mobility) in self
            .ranges
            .iter_mut()
            .zip(values.spending_rate.iter())
            .zip(values.spending_exposure.iter())
            .zip(values.mobility.iter())
        {
            range.spending_rate.height.push(*spending_rate);
            range
                .spending_exposure
                .height
                .push(Float::from(f64::from(*spending_exposure)));
            range.mobility_source.push(*mobility);
        }
    }
}
