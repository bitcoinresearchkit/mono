use bitview_traversable::Traversable;
use brk_types::RarityPercentileId;

#[derive(Clone, Traversable)]
pub struct RarityPercentiles<T> {
    /// Uses the 0.1% quantile.
    pub pct0_1: T,
    /// Uses the 0.5% quantile.
    pub pct0_5: T,
    /// Uses the 1% quantile.
    pub pct1: T,
    /// Uses the 2% quantile.
    pub pct2: T,
    /// Uses the 5% quantile.
    pub pct5: T,
    /// Uses the 10% quantile.
    pub pct10: T,
    /// Uses the 20% quantile.
    pub pct20: T,
    /// Uses the 30% quantile.
    pub pct30: T,
    /// Uses the 40% quantile.
    pub pct40: T,
    /// Uses the 50% quantile.
    pub pct50: T,
    /// Uses the 60% quantile.
    pub pct60: T,
    /// Uses the 70% quantile.
    pub pct70: T,
    /// Uses the 80% quantile.
    pub pct80: T,
    /// Uses the 90% quantile.
    pub pct90: T,
    /// Uses the 95% quantile.
    pub pct95: T,
    /// Uses the 98% quantile.
    pub pct98: T,
    /// Uses the 99% quantile.
    pub pct99: T,
    /// Uses the 99.5% quantile.
    pub pct99_5: T,
    /// Uses the 99.9% quantile.
    pub pct99_9: T,
}

impl<T> RarityPercentiles<T> {
    pub fn from_fn(mut f: impl FnMut(RarityPercentileId) -> T) -> Self {
        use RarityPercentileId::*;

        Self {
            pct0_1: f(Pct0_1),
            pct0_5: f(Pct0_5),
            pct1: f(Pct1),
            pct2: f(Pct2),
            pct5: f(Pct5),
            pct10: f(Pct10),
            pct20: f(Pct20),
            pct30: f(Pct30),
            pct40: f(Pct40),
            pct50: f(Pct50),
            pct60: f(Pct60),
            pct70: f(Pct70),
            pct80: f(Pct80),
            pct90: f(Pct90),
            pct95: f(Pct95),
            pct98: f(Pct98),
            pct99: f(Pct99),
            pct99_5: f(Pct99_5),
            pct99_9: f(Pct99_9),
        }
    }

    fn get(&self, id: RarityPercentileId) -> &T {
        use RarityPercentileId::*;

        match id {
            Pct0_1 => &self.pct0_1,
            Pct0_5 => &self.pct0_5,
            Pct1 => &self.pct1,
            Pct2 => &self.pct2,
            Pct5 => &self.pct5,
            Pct10 => &self.pct10,
            Pct20 => &self.pct20,
            Pct30 => &self.pct30,
            Pct40 => &self.pct40,
            Pct50 => &self.pct50,
            Pct60 => &self.pct60,
            Pct70 => &self.pct70,
            Pct80 => &self.pct80,
            Pct90 => &self.pct90,
            Pct95 => &self.pct95,
            Pct98 => &self.pct98,
            Pct99 => &self.pct99,
            Pct99_5 => &self.pct99_5,
            Pct99_9 => &self.pct99_9,
        }
    }

    pub fn boundary_refs(&self) -> [&T; 10] {
        RarityPercentileId::BOUNDARIES.map(|id| self.get(id))
    }
}
