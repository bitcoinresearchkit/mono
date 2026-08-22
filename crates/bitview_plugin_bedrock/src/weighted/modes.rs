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
    /// Bedrock's eight-year coinflow mode weights each UTXO age range by its
    /// estimated probability of being spent within eight years, derived from
    /// observed spending rates, and calibrates against the resulting weighted
    /// share of supply in loss.
    pub coinflow_8y: T,
    /// Bedrock's four-year coinflow mode weights each UTXO age range by its
    /// estimated probability of being spent within four years, derived from
    /// observed spending rates, and calibrates against the resulting weighted
    /// share of supply in loss.
    pub coinflow_4y: T,
    /// Bedrock's two-year coinflow mode weights each UTXO age range by its
    /// estimated probability of being spent within two years, derived from
    /// observed spending rates, and calibrates against the resulting weighted
    /// share of supply in loss.
    pub coinflow_2y: T,
    /// Bedrock's one-year coinflow mode weights each UTXO age range by its
    /// estimated probability of being spent within one year, derived from
    /// observed spending rates, and calibrates against the resulting weighted
    /// share of supply in loss.
    pub coinflow_1y: T,
    /// Bedrock's six-month coinflow mode weights each UTXO age range by its
    /// estimated probability of being spent within six months, derived from
    /// observed spending rates, and calibrates against the resulting weighted
    /// share of supply in loss.
    pub coinflow_6m: T,
    /// Bedrock's three-month coinflow mode weights each UTXO age range by its
    /// estimated probability of being spent within three months, derived from
    /// observed spending rates, and calibrates against the resulting weighted
    /// share of supply in loss.
    pub coinflow_3m: T,
    /// Bedrock's one-month coinflow mode weights each UTXO age range by its
    /// estimated probability of being spent within one month, derived from
    /// observed spending rates, and calibrates against the resulting weighted
    /// share of supply in loss.
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
