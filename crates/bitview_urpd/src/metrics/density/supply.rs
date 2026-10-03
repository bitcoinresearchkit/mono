use bitview_primitives::PartsPerMillion32;
use bitview_traversable::Traversable;

#[derive(Clone, Copy, Debug, PartialEq, Traversable)]
pub struct SupplyDensity<T> {
    /// Share of total weighted supply with cost basis within 5% below or above
    /// per-block closing spot, using rounded URPD creation-price buckets.
    pub total: T,
    /// Share of total weighted supply from the lower band boundary through spot.
    pub in_profit: T,
    /// Share of total weighted supply above spot through the upper band boundary.
    pub in_loss: T,
}

impl<T> SupplyDensity<T> {
    pub fn try_from_fn<E>(mut create: impl FnMut(&str) -> Result<T, E>) -> Result<Self, E> {
        Ok(Self {
            total: create("_total")?,
            in_profit: create("_in_profit")?,
            in_loss: create("_in_loss")?,
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        [&self.total, &self.in_profit, &self.in_loss].into_iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [&mut self.total, &mut self.in_profit, &mut self.in_loss].into_iter()
    }
}

impl SupplyDensity<PartsPerMillion32> {
    pub const NAN: Self = Self {
        total: PartsPerMillion32::NAN,
        in_profit: PartsPerMillion32::NAN,
        in_loss: PartsPerMillion32::NAN,
    };

    pub fn from_sums(total: u128, profit: u128, loss: u128) -> Self {
        if total == 0 {
            return Self::NAN;
        }
        Self {
            total: PartsPerMillion32::from((profit + loss) as f64 / total as f64),
            in_profit: PartsPerMillion32::from(profit as f64 / total as f64),
            in_loss: PartsPerMillion32::from(loss as f64 / total as f64),
        }
    }
}
