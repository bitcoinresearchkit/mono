use super::{ModeId, Modes, Percentiles, Thresholds, WeightedModeId};
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
        cointime: &impl ReadableVec<Height, U>,
        coinflow: &impl ReadableVec<Height, U>,
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
                Some(WeightedModeId::Cointime) => Self::history(cointime, end),
                Some(WeightedModeId::Coinflow) => Self::history(coinflow, end),
            }),
        }
    }
    /// One block's loss shares by mode; non-finite shares are missing.
    pub fn loss_shares(
        raw: Option<f64>,
        cointime: Option<f64>,
        coinflow: Option<f64>,
    ) -> Modes<Option<f64>> {
        Modes::from_fn(|mode| {
            match mode.weighted() {
                None => raw,
                Some(WeightedModeId::Cointime) => cointime,
                Some(WeightedModeId::Coinflow) => coinflow,
            }
            .filter(|v| v.is_finite())
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
    fn history<T>(source: &impl ReadableVec<Height, T>, end: usize) -> ExactOrderStats
    where
        T: VecValue,
        f64: From<T>,
    {
        let mut values = Vec::with_capacity(end);
        source.for_each_range_at(0, end, |value| {
            let value = f64::from(value);
            if value.is_finite() {
                values.push(value.clamp(0.0, 1.0));
            }
        });
        ExactOrderStats::from_unsorted(values)
    }
}
