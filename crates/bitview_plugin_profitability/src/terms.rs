use bitview_cohort::AgeAggregateId;
use bitview_traversable::Traversable;

/// The age filters the bands are published for.
pub(crate) const TERMS: [AgeAggregateId; 3] = [
    AgeAggregateId::All,
    AgeAggregateId::Sth,
    AgeAggregateId::Lth,
];

/// All, short-term and long-term holders.
#[derive(Clone, Traversable)]
pub(crate) struct Terms<T> {
    /// Uses all UTXOs.
    all: T,
    /// Uses short-term-holder UTXOs younger than 150 days.
    sth: T,
    /// Uses long-term-holder UTXOs at least 150 days old.
    lth: T,
}

impl<T> Terms<T> {
    pub(crate) fn from_fn(mut f: impl FnMut(AgeAggregateId) -> T) -> Self {
        Self {
            all: f(AgeAggregateId::All),
            sth: f(AgeAggregateId::Sth),
            lth: f(AgeAggregateId::Lth),
        }
    }

    pub(crate) fn try_from_fn<E>(
        mut f: impl FnMut(AgeAggregateId) -> Result<T, E>,
    ) -> Result<Self, E> {
        Ok(Self {
            all: f(AgeAggregateId::All)?,
            sth: f(AgeAggregateId::Sth)?,
            lth: f(AgeAggregateId::Lth)?,
        })
    }

    pub(crate) fn select(&self, term: AgeAggregateId) -> &T {
        match term {
            AgeAggregateId::Sth => &self.sth,
            AgeAggregateId::Lth => &self.lth,
            _ => &self.all,
        }
    }

    pub(crate) fn select_mut(&mut self, term: AgeAggregateId) -> &mut T {
        match term {
            AgeAggregateId::Sth => &mut self.sth,
            AgeAggregateId::Lth => &mut self.lth,
            _ => &mut self.all,
        }
    }

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [&mut self.all, &mut self.sth, &mut self.lth].into_iter()
    }
}
