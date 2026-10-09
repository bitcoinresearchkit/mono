use vecdb::{AnyStoredVec, WritableVec};

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
            range.spending_exposure.height.push(*spending_exposure);
            range.mobility_source.push(*mobility);
        }
    }

    pub(crate) fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.ranges.iter_mut().flat_map(|range| {
            [
                &mut range.spending_rate.height as &mut dyn AnyStoredVec,
                &mut range.spending_exposure.height,
                &mut range.mobility_source,
            ]
        })
    }
}
