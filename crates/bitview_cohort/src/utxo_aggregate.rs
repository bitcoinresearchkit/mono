use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AgeRangeId, CohortContext, CohortId, CohortName, LTH_AGE_RANGE_IDS, STH_AGE_RANGE_IDS, Term,
};

#[cfg(feature = "storage")]
use bitview_traversable::Traversable;

/// Canonical name for the aggregate cohort containing every UTXO.
pub(crate) const UTXO_ALL_NAME: CohortName = CohortName::new("all", "All", "All UTXOs");

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "storage", derive(Traversable))]
pub struct UTXOAggregate<T> {
    /// Uses all UTXOs.
    pub all: T,
    /// Uses short-term-holder UTXOs younger than 150 days.
    pub sth: T,
    /// Uses long-term-holder UTXOs at least 150 days old.
    pub lth: T,
}

define_cohort_id!(
    UTXOAggregateId for UTXOAggregate {
        All => all,
        Sth => sth,
        Lth => lth,
    }
);

impl UTXOAggregateId {
    pub(crate) const fn age_range_ids(self) -> &'static [AgeRangeId] {
        match self {
            Self::All => AgeRangeId::ALL,
            Self::Sth => STH_AGE_RANGE_IDS,
            Self::Lth => LTH_AGE_RANGE_IDS,
        }
    }

    const fn cohort(self) -> CohortId {
        match self {
            Self::All => CohortId::All,
            Self::Sth => CohortId::Term(Term::Sth),
            Self::Lth => CohortId::Term(Term::Lth),
        }
    }

    pub fn metric_name(self, metric: &str) -> String {
        CohortContext::Utxo.metric_name(self.cohort(), metric)
    }
}
