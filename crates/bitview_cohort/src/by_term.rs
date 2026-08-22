use bitview_traversable::Traversable;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use schemars::JsonSchema;
use serde::Serialize;

use super::{CohortName, Filter, Term};

/// Term values
pub const TERM_VALUES: ByTerm<Term> = ByTerm {
    short: Term::Sth,
    long: Term::Lth,
};

/// Term filters
pub const TERM_FILTERS: ByTerm<Filter> = ByTerm {
    short: Filter::Term(TERM_VALUES.short),
    long: Filter::Term(TERM_VALUES.long),
};

/// Term names
pub const TERM_NAMES: ByTerm<CohortName> = ByTerm {
    short: CohortName::new("sth", "STH", "Short Term Holders"),
    long: CohortName::new("lth", "LTH", "Long Term Holders"),
};

#[derive(Debug, Default, Clone, Copy, Traversable, Serialize, JsonSchema)]
pub struct ByTerm<T> {
    /// Uses short-term-holder UTXOs younger than 150 days.
    pub short: T,
    /// Uses long-term-holder UTXOs at least 150 days old.
    pub long: T,
}

define_column_id!(
    TermId for ByTerm, version = 1 {
        Short => short,
        Long => long,
    }
);

impl ByTerm<CohortName> {
    pub const fn names() -> &'static Self {
        &TERM_NAMES
    }
}

impl<T> ByTerm<T> {
    pub fn new<F>(mut create: F) -> Self
    where
        F: FnMut(Filter, &'static str) -> T,
    {
        let f = TERM_FILTERS;
        let n = TERM_NAMES;
        Self {
            short: create(f.short, n.short.id),
            long: create(f.long, n.long.id),
        }
    }

    pub fn try_new<F, E>(mut create: F) -> Result<Self, E>
    where
        F: FnMut(Filter, &'static str) -> Result<T, E>,
    {
        let f = TERM_FILTERS;
        let n = TERM_NAMES;
        Ok(Self {
            short: create(f.short, n.short.id)?,
            long: create(f.long, n.long.id)?,
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        [&self.short, &self.long].into_iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [&mut self.short, &mut self.long].into_iter()
    }

    pub fn par_iter(&self) -> impl ParallelIterator<Item = &T>
    where
        T: Send + Sync,
    {
        [&self.short, &self.long].into_par_iter()
    }

    pub fn par_iter_mut(&mut self) -> impl ParallelIterator<Item = &mut T>
    where
        T: Send + Sync,
    {
        [&mut self.short, &mut self.long].into_par_iter()
    }
}
