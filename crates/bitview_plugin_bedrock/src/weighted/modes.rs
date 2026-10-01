use bitview_traversable::Traversable;

use super::WeightedModeId;

#[derive(Traversable)]
pub struct WeightedModes<T> {
    /// Bedrock's cointime mode weights each UTXO age range by wakefulness—the
    /// share of its accumulated coin days that has been consumed—and
    /// calibrates against the resulting weighted share of supply in loss.
    pub cointime: T,
    /// Bedrock's coinflow mode weights each UTXO age range by mobility—the
    /// estimated probability that UTXOs of that age will ever be spent—and
    /// calibrates against the resulting weighted share of supply in loss.
    pub coinflow: T,
}

impl<T> WeightedModes<T> {
    pub fn from_fn(mut create: impl FnMut(WeightedModeId) -> T) -> Self {
        Self {
            cointime: create(WeightedModeId::Cointime),
            coinflow: create(WeightedModeId::Coinflow),
        }
    }

    pub fn try_from_fn<E>(
        mut create: impl FnMut(WeightedModeId) -> Result<T, E>,
    ) -> Result<Self, E> {
        Ok(Self {
            cointime: create(WeightedModeId::Cointime)?,
            coinflow: create(WeightedModeId::Coinflow)?,
        })
    }

    pub fn select_mut(&mut self, id: WeightedModeId) -> &mut T {
        match id {
            WeightedModeId::Cointime => &mut self.cointime,
            WeightedModeId::Coinflow => &mut self.coinflow,
        }
    }

    pub fn select(&self, id: WeightedModeId) -> &T {
        match id {
            WeightedModeId::Cointime => &self.cointime,
            WeightedModeId::Coinflow => &self.coinflow,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        [&self.cointime, &self.coinflow].into_iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [&mut self.cointime, &mut self.coinflow].into_iter()
    }
}
