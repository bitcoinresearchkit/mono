#[cfg(feature = "storage")]
use bitview_traversable::Traversable;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{CohortId, CohortName, Term};

/// Term names
pub const TERM_NAMES: ByTerm<CohortName> = ByTerm {
    short: CohortName::new("sth", "STH", "Short Term Holders"),
    long: CohortName::new("lth", "LTH", "Long Term Holders"),
};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "storage", derive(Traversable))]
pub struct ByTerm<T> {
    /// Uses short-term-holder UTXOs younger than 150 days.
    pub short: T,
    /// Uses long-term-holder UTXOs at least 150 days old.
    pub long: T,
}

impl_cohort_collection!(
    Term for ByTerm {
        Sth => short,
        Lth => long,
    }
);

impl<T> ByTerm<T> {
    pub fn try_new<E>(mut create: impl FnMut(CohortId) -> Result<T, E>) -> Result<Self, E> {
        Self::try_from_fn(|term| create(CohortId::Term(term)))
    }

    pub fn get(&self, term: Term) -> &T {
        match term {
            Term::Sth => &self.short,
            Term::Lth => &self.long,
        }
    }

    pub fn get_mut(&mut self, term: Term) -> &mut T {
        match term {
            Term::Sth => &mut self.short,
            Term::Lth => &mut self.long,
        }
    }
}
