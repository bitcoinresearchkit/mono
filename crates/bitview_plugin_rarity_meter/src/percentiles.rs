use bitview_traversable::Traversable;
use brk_types::RarityPercentileId;

#[derive(Clone, Traversable)]
pub struct RarityPercentiles<T> {
    pub pct0_1: T,
    pub pct0_5: T,
    pub pct1: T,
    pub pct2: T,
    pub pct5: T,
    pub pct10: T,
    pub pct20: T,
    pub pct30: T,
    pub pct40: T,
    pub pct50: T,
    pub pct60: T,
    pub pct70: T,
    pub pct80: T,
    pub pct90: T,
    pub pct95: T,
    pub pct98: T,
    pub pct99: T,
    pub pct99_5: T,
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
