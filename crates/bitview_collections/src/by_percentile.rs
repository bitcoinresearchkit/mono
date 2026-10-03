#[cfg(feature = "storage")]
use bitview_traversable::Traversable;
use brk_types::PercentileId;

#[derive(Clone)]
#[cfg_attr(feature = "storage", derive(Traversable))]
pub struct ByPercentile<T> {
    /// Uses the 5th percentile.
    pct05: T,
    /// Uses the 10th percentile.
    pct10: T,
    /// Uses the 15th percentile.
    pct15: T,
    /// Uses the 20th percentile.
    pct20: T,
    /// Uses the 25th percentile.
    pct25: T,
    /// Uses the 30th percentile.
    pct30: T,
    /// Uses the 35th percentile.
    pct35: T,
    /// Uses the 40th percentile.
    pct40: T,
    /// Uses the 45th percentile.
    pct45: T,
    /// Uses the 50th percentile.
    pub pct50: T,
    /// Uses the 55th percentile.
    pct55: T,
    /// Uses the 60th percentile.
    pct60: T,
    /// Uses the 65th percentile.
    pct65: T,
    /// Uses the 70th percentile.
    pct70: T,
    /// Uses the 75th percentile.
    pct75: T,
    /// Uses the 80th percentile.
    pct80: T,
    /// Uses the 85th percentile.
    pct85: T,
    /// Uses the 90th percentile.
    pct90: T,
    /// Uses the 95th percentile.
    pct95: T,
}

impl<T> ByPercentile<T> {
    pub fn try_from_fn<E>(mut f: impl FnMut(PercentileId) -> Result<T, E>) -> Result<Self, E> {
        Ok(Self {
            pct05: f(PercentileId::Pct05)?,
            pct10: f(PercentileId::Pct10)?,
            pct15: f(PercentileId::Pct15)?,
            pct20: f(PercentileId::Pct20)?,
            pct25: f(PercentileId::Pct25)?,
            pct30: f(PercentileId::Pct30)?,
            pct35: f(PercentileId::Pct35)?,
            pct40: f(PercentileId::Pct40)?,
            pct45: f(PercentileId::Pct45)?,
            pct50: f(PercentileId::Pct50)?,
            pct55: f(PercentileId::Pct55)?,
            pct60: f(PercentileId::Pct60)?,
            pct65: f(PercentileId::Pct65)?,
            pct70: f(PercentileId::Pct70)?,
            pct75: f(PercentileId::Pct75)?,
            pct80: f(PercentileId::Pct80)?,
            pct85: f(PercentileId::Pct85)?,
            pct90: f(PercentileId::Pct90)?,
            pct95: f(PercentileId::Pct95)?,
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [
            &mut self.pct05,
            &mut self.pct10,
            &mut self.pct15,
            &mut self.pct20,
            &mut self.pct25,
            &mut self.pct30,
            &mut self.pct35,
            &mut self.pct40,
            &mut self.pct45,
            &mut self.pct50,
            &mut self.pct55,
            &mut self.pct60,
            &mut self.pct65,
            &mut self.pct70,
            &mut self.pct75,
            &mut self.pct80,
            &mut self.pct85,
            &mut self.pct90,
            &mut self.pct95,
        ]
        .into_iter()
    }

    pub fn select(&self, id: PercentileId) -> &T {
        match id {
            PercentileId::Pct05 => &self.pct05,
            PercentileId::Pct10 => &self.pct10,
            PercentileId::Pct15 => &self.pct15,
            PercentileId::Pct20 => &self.pct20,
            PercentileId::Pct25 => &self.pct25,
            PercentileId::Pct30 => &self.pct30,
            PercentileId::Pct35 => &self.pct35,
            PercentileId::Pct40 => &self.pct40,
            PercentileId::Pct45 => &self.pct45,
            PercentileId::Pct50 => &self.pct50,
            PercentileId::Pct55 => &self.pct55,
            PercentileId::Pct60 => &self.pct60,
            PercentileId::Pct65 => &self.pct65,
            PercentileId::Pct70 => &self.pct70,
            PercentileId::Pct75 => &self.pct75,
            PercentileId::Pct80 => &self.pct80,
            PercentileId::Pct85 => &self.pct85,
            PercentileId::Pct90 => &self.pct90,
            PercentileId::Pct95 => &self.pct95,
        }
    }

    pub fn from_fn(mut f: impl FnMut(PercentileId) -> T) -> Self {
        Self {
            pct05: f(PercentileId::Pct05),
            pct10: f(PercentileId::Pct10),
            pct15: f(PercentileId::Pct15),
            pct20: f(PercentileId::Pct20),
            pct25: f(PercentileId::Pct25),
            pct30: f(PercentileId::Pct30),
            pct35: f(PercentileId::Pct35),
            pct40: f(PercentileId::Pct40),
            pct45: f(PercentileId::Pct45),
            pct50: f(PercentileId::Pct50),
            pct55: f(PercentileId::Pct55),
            pct60: f(PercentileId::Pct60),
            pct65: f(PercentileId::Pct65),
            pct70: f(PercentileId::Pct70),
            pct75: f(PercentileId::Pct75),
            pct80: f(PercentileId::Pct80),
            pct85: f(PercentileId::Pct85),
            pct90: f(PercentileId::Pct90),
            pct95: f(PercentileId::Pct95),
        }
    }
}
