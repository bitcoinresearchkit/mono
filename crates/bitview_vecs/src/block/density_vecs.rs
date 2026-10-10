use bitview_primitives::PartsPerMillion32;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Height, Version};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode, WritableVec};

use crate::{CachedSeries, IndexSources, LazyPercentPerBlock, import_cached};

/// A cohort's share of a quantity whose creation price lies within 5% of spot,
/// split at spot.
#[derive(Clone, Copy, Debug, Default, PartialEq, Traversable)]
pub struct Density<T> {
    /// Within 5% below or above the represented block's spot price.
    total: T,
    /// From 5% below spot through spot: the part in profit.
    in_profit: T,
    /// Above spot through 5% above it: the part in loss.
    in_loss: T,
}

impl<T> Density<T> {
    /// Builds every member from its id suffix (`""`, `"_in_profit"`, `"_in_loss"`).
    fn try_from_fn<E>(
        mut create: impl FnMut(&str) -> std::result::Result<T, E>,
    ) -> std::result::Result<Self, E> {
        Ok(Self {
            total: create("")?,
            in_profit: create("_in_profit")?,
            in_loss: create("_in_loss")?,
        })
    }

    fn iter(&self) -> impl Iterator<Item = &T> {
        [&self.total, &self.in_profit, &self.in_loss].into_iter()
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [&mut self.total, &mut self.in_profit, &mut self.in_loss].into_iter()
    }
}

impl Density<PartsPerMillion32> {
    pub const NAN: Self = Self {
        total: PartsPerMillion32::NAN,
        in_profit: PartsPerMillion32::NAN,
        in_loss: PartsPerMillion32::NAN,
    };

    /// Shares of `total` in the band's profit and loss halves; NaN when `total` is zero.
    pub fn from_sums(total: u128, profit: u128, loss: u128) -> Self {
        if total == 0 {
            return Self::NAN;
        }
        let total = total as f64;
        Self {
            total: PartsPerMillion32::from((profit + loss) as f64 / total),
            in_profit: PartsPerMillion32::from(profit as f64 / total),
            in_loss: PartsPerMillion32::from(loss as f64 / total),
        }
    }
}

/// Stored density shares with their percent views.
#[derive(Traversable)]
pub struct DensityVecs<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub series: Density<LazyPercentPerBlock<PartsPerMillion32>>,
    #[traversable(hidden)]
    stored: Density<CachedSeries<Height, PartsPerMillion32, M>>,
}

impl DensityVecs {
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        let stored = Density::try_from_fn(|suffix| {
            import_cached(db, &format!("{name}{suffix}_ppm"), version)
        })?;
        let view = |source: &CachedSeries<Height, PartsPerMillion32>, suffix| {
            LazyPercentPerBlock::from_height_source(
                &format!("{name}{suffix}"),
                version,
                source,
                indexes,
            )
        };
        let series = Density {
            total: view(&stored.total, ""),
            in_profit: view(&stored.in_profit, "_in_profit"),
            in_loss: view(&stored.in_loss, "_in_loss"),
        };
        Ok(Self { series, stored })
    }

    pub fn push(&mut self, density: &Density<PartsPerMillion32>) {
        for (target, &value) in self.stored.iter_mut().zip(density.iter()) {
            target.push(value);
        }
    }

    pub fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.stored
            .iter_mut()
            .map(|vec| vec as &mut dyn AnyStoredVec)
    }
}
