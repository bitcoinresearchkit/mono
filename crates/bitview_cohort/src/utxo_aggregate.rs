use bitview_traversable::Traversable;
use schemars::JsonSchema;
use serde::Serialize;

use crate::{CohortContext, CohortName, Filter, TERM_FILTERS, TERM_NAMES, TermId};

/// Canonical name for the aggregate cohort containing every UTXO.
pub const UTXO_ALL_NAME: CohortName = CohortName::new("all", "All", "All UTXOs");

pub const UTXO_AGGREGATE_FILTERS: UTXOAggregate<Filter> = UTXOAggregate {
    all: Filter::All,
    sth: TERM_FILTERS.short,
    lth: TERM_FILTERS.long,
};

/// Canonical names for the aggregate UTXO cohorts.
pub const UTXO_AGGREGATE_NAMES: UTXOAggregate<CohortName> = UTXOAggregate {
    all: UTXO_ALL_NAME,
    sth: TERM_NAMES.short,
    lth: TERM_NAMES.long,
};

#[derive(Debug, Default, Clone, Traversable, Serialize, JsonSchema)]
pub struct UTXOAggregate<T> {
    /// Uses all UTXOs.
    pub all: T,
    /// Uses short-term-holder UTXOs younger than 150 days.
    pub sth: T,
    /// Uses long-term-holder UTXOs at least 150 days old.
    pub lth: T,
}

define_column_id!(
    UTXOAggregateId for UTXOAggregate, version = 1 {
        All => all,
        Sth => sth,
        Lth => lth,
    }
);

impl UTXOAggregateId {
    pub const fn cohort_name(self) -> CohortName {
        match self {
            Self::All => UTXO_AGGREGATE_NAMES.all,
            Self::Sth => UTXO_AGGREGATE_NAMES.sth,
            Self::Lth => UTXO_AGGREGATE_NAMES.lth,
        }
    }

    pub const fn term(self) -> Option<TermId> {
        match self {
            Self::All => None,
            Self::Sth => Some(TermId::Short),
            Self::Lth => Some(TermId::Long),
        }
    }

    pub fn metric_name(self, metric: &str) -> String {
        CohortContext::Utxo.metric_name(
            self.select(&UTXO_AGGREGATE_FILTERS),
            self.cohort_name().id,
            metric,
        )
    }
}

impl<T> UTXOAggregate<T> {
    pub fn map<U>(&self, mut f: impl FnMut(&T) -> U) -> UTXOAggregate<U> {
        UTXOAggregate {
            all: f(&self.all),
            sth: f(&self.sth),
            lth: f(&self.lth),
        }
    }

    pub fn try_from_fn<E>(mut f: impl FnMut(UTXOAggregateId) -> Result<T, E>) -> Result<Self, E> {
        Ok(Self {
            all: f(UTXOAggregateId::All)?,
            sth: f(UTXOAggregateId::Sth)?,
            lth: f(UTXOAggregateId::Lth)?,
        })
    }

    pub fn get(&self, filter: &Filter) -> Option<&T> {
        match filter {
            Filter::All => Some(&self.all),
            Filter::Term(crate::Term::Sth) => Some(&self.sth),
            Filter::Term(crate::Term::Lth) => Some(&self.lth),
            _ => None,
        }
    }

    pub fn get_mut(&mut self, filter: &Filter) -> Option<&mut T> {
        match filter {
            Filter::All => Some(&mut self.all),
            Filter::Term(crate::Term::Sth) => Some(&mut self.sth),
            Filter::Term(crate::Term::Lth) => Some(&mut self.lth),
            _ => None,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        [&self.all, &self.sth, &self.lth].into_iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [&mut self.all, &mut self.sth, &mut self.lth].into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregate_metric_names_omit_only_the_all_prefix() {
        assert_eq!(
            UTXOAggregateId::All.metric_name("capitalized_price"),
            "capitalized_price"
        );
        assert_eq!(
            UTXOAggregateId::Sth.metric_name("capitalized_price"),
            "sth_capitalized_price"
        );
        assert_eq!(
            UTXOAggregateId::Lth.metric_name("capitalized_price"),
            "lth_capitalized_price"
        );
    }
}
