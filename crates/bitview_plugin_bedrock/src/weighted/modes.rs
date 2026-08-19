use bitview_traversable::Traversable;

use super::WeightedModeId;

#[derive(Traversable)]
pub struct WeightedModes<T> {
    /// Weights age cohorts by cointime wakefulness and calibrates against active
    /// supply in loss share.
    pub cointime: T,
    /// Weights age cohorts by coinflow mobility and calibrates against
    /// coinflow-weighted supply in loss share.
    pub coinflow: T,
    /// Weights age cohorts by their probability of spending within eight years,
    /// derived from coinflow spending rates, and calibrates against the
    /// corresponding horizon supply-in-loss share.
    pub coinflow_8y: T,
    /// Weights age cohorts by their probability of spending within four years,
    /// derived from coinflow spending rates, and calibrates against the
    /// corresponding horizon supply-in-loss share.
    pub coinflow_4y: T,
    /// Weights age cohorts by their probability of spending within two years,
    /// derived from coinflow spending rates, and calibrates against the
    /// corresponding horizon supply-in-loss share.
    pub coinflow_2y: T,
    /// Weights age cohorts by their probability of spending within one year,
    /// derived from coinflow spending rates, and calibrates against the
    /// corresponding horizon supply-in-loss share.
    pub coinflow_1y: T,
    /// Weights age cohorts by their probability of spending within six months,
    /// derived from coinflow spending rates, and calibrates against the
    /// corresponding horizon supply-in-loss share.
    pub coinflow_6m: T,
    /// Weights age cohorts by their probability of spending within three months,
    /// derived from coinflow spending rates, and calibrates against the
    /// corresponding horizon supply-in-loss share.
    pub coinflow_3m: T,
    /// Weights age cohorts by their probability of spending within one month,
    /// derived from coinflow spending rates, and calibrates against the
    /// corresponding horizon supply-in-loss share.
    pub coinflow_1m: T,
}

impl<T> WeightedModes<T> {
    pub fn from_fn(mut create: impl FnMut(WeightedModeId) -> T) -> Self {
        Self {
            cointime: create(WeightedModeId::Cointime),
            coinflow: create(WeightedModeId::Coinflow),
            coinflow_8y: create(WeightedModeId::Coinflow8Y),
            coinflow_4y: create(WeightedModeId::Coinflow4Y),
            coinflow_2y: create(WeightedModeId::Coinflow2Y),
            coinflow_1y: create(WeightedModeId::Coinflow1Y),
            coinflow_6m: create(WeightedModeId::Coinflow6M),
            coinflow_3m: create(WeightedModeId::Coinflow3M),
            coinflow_1m: create(WeightedModeId::Coinflow1M),
        }
    }

    pub fn try_from_fn<E>(
        mut create: impl FnMut(WeightedModeId) -> Result<T, E>,
    ) -> Result<Self, E> {
        Ok(Self {
            cointime: create(WeightedModeId::Cointime)?,
            coinflow: create(WeightedModeId::Coinflow)?,
            coinflow_8y: create(WeightedModeId::Coinflow8Y)?,
            coinflow_4y: create(WeightedModeId::Coinflow4Y)?,
            coinflow_2y: create(WeightedModeId::Coinflow2Y)?,
            coinflow_1y: create(WeightedModeId::Coinflow1Y)?,
            coinflow_6m: create(WeightedModeId::Coinflow6M)?,
            coinflow_3m: create(WeightedModeId::Coinflow3M)?,
            coinflow_1m: create(WeightedModeId::Coinflow1M)?,
        })
    }

    pub fn select_mut(&mut self, id: WeightedModeId) -> &mut T {
        match id {
            WeightedModeId::Cointime => &mut self.cointime,
            WeightedModeId::Coinflow => &mut self.coinflow,
            WeightedModeId::Coinflow8Y => &mut self.coinflow_8y,
            WeightedModeId::Coinflow4Y => &mut self.coinflow_4y,
            WeightedModeId::Coinflow2Y => &mut self.coinflow_2y,
            WeightedModeId::Coinflow1Y => &mut self.coinflow_1y,
            WeightedModeId::Coinflow6M => &mut self.coinflow_6m,
            WeightedModeId::Coinflow3M => &mut self.coinflow_3m,
            WeightedModeId::Coinflow1M => &mut self.coinflow_1m,
        }
    }

    pub fn select(&self, id: WeightedModeId) -> &T {
        match id {
            WeightedModeId::Cointime => &self.cointime,
            WeightedModeId::Coinflow => &self.coinflow,
            WeightedModeId::Coinflow8Y => &self.coinflow_8y,
            WeightedModeId::Coinflow4Y => &self.coinflow_4y,
            WeightedModeId::Coinflow2Y => &self.coinflow_2y,
            WeightedModeId::Coinflow1Y => &self.coinflow_1y,
            WeightedModeId::Coinflow6M => &self.coinflow_6m,
            WeightedModeId::Coinflow3M => &self.coinflow_3m,
            WeightedModeId::Coinflow1M => &self.coinflow_1m,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        [
            &self.cointime,
            &self.coinflow,
            &self.coinflow_8y,
            &self.coinflow_4y,
            &self.coinflow_2y,
            &self.coinflow_1y,
            &self.coinflow_6m,
            &self.coinflow_3m,
            &self.coinflow_1m,
        ]
        .into_iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [
            &mut self.cointime,
            &mut self.coinflow,
            &mut self.coinflow_8y,
            &mut self.coinflow_4y,
            &mut self.coinflow_2y,
            &mut self.coinflow_1y,
            &mut self.coinflow_6m,
            &mut self.coinflow_3m,
            &mut self.coinflow_1m,
        ]
        .into_iter()
    }
}
