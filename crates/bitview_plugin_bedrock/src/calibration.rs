use super::{ModeId, Modes, Percentiles, Thresholds, WeightedModes};
use bitview_compute::ExactOrderStats;
use brk_types::{Height, Version};
use vecdb::{ReadableVec, VecValue};

// One nominal year of block observations. The represented block is excluded.
const MINIMUM_BLOCKS: usize = 365 * 144;
const PERCENTILES: [f64; 5] = [0.95, 0.98, 0.99, 0.995, 0.999];

pub struct Calibration {
    pub(crate) end: usize,
    pub(crate) version: Version,
    histories: Modes<ExactOrderStats>,
}
impl Calibration {
    pub fn from_sources<T, U>(
        raw: &impl ReadableVec<Height, T>,
        weighted: &WeightedModes<&dyn ReadableVec<Height, U>>,
        end: usize,
        version: Version,
    ) -> Self
    where
        T: VecValue,
        U: VecValue,
        f64: From<T> + From<U>,
    {
        Self {
            end,
            version,
            histories: Modes::from_fn(|mode| match mode.weighted() {
                None => Self::history(raw, end),
                Some(id) => Self::history(*weighted.select(id), end),
            }),
        }
    }
    pub fn loss_shares<T, U>(
        raw: &impl ReadableVec<Height, T>,
        weighted: &WeightedModes<&dyn ReadableVec<Height, U>>,
        height: Height,
    ) -> Modes<Option<f64>>
    where
        T: VecValue,
        U: VecValue,
        f64: From<T> + From<U>,
    {
        Modes::from_fn(|mode| match mode.weighted() {
            None => Self::loss_share(raw, height),
            Some(id) => Self::loss_share(*weighted.select(id), height),
        })
    }
    pub fn thresholds(&self, current: &Modes<Option<f64>>) -> Thresholds {
        Thresholds::from_fn(|mode| {
            let history = self.histories.select(mode);
            (current.select(mode).is_some() && history.len() >= MINIMUM_BLOCKS).then(|| {
                let [pct95, pct98, pct99, pct99_5, pct99_9] = history.percentiles(&PERCENTILES);
                Percentiles {
                    pct95,
                    pct98,
                    pct99,
                    pct99_5,
                    pct99_9,
                }
            })
        })
    }
    pub fn observe(&mut self, shares: Modes<Option<f64>>) {
        for mode in ModeId::ALL {
            if let Some(share) = *shares.select(mode) {
                self.histories
                    .select_mut(mode)
                    .insert(share.clamp(0.0, 1.0));
            }
        }
        self.end += 1;
    }
    fn history<T>(source: &(impl ReadableVec<Height, T> + ?Sized), end: usize) -> ExactOrderStats
    where
        T: VecValue,
        f64: From<T>,
    {
        let mut values = Vec::with_capacity(end);
        source.for_each_range_dyn_at(0, end, &mut |value| {
            let value = f64::from(value);
            if value.is_finite() {
                values.push(value.clamp(0.0, 1.0));
            }
        });
        ExactOrderStats::from_unsorted(values)
    }
    fn loss_share<T>(source: &(impl ReadableVec<Height, T> + ?Sized), height: Height) -> Option<f64>
    where
        T: VecValue,
        f64: From<T>,
    {
        source
            .collect_one(height)
            .map(f64::from)
            .filter(|v| v.is_finite())
    }
}
