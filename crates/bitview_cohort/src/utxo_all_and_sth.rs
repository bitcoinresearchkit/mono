use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cfg(feature = "storage")]
use bitview_traversable::Traversable;

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "storage", derive(Traversable))]
pub struct UTXOAllAndSth<T> {
    /// Uses all UTXOs.
    all: T,
    /// Uses short-term-holder UTXOs younger than 150 days.
    sth: T,
}

define_cohort_id!(
    UTXOAllAndSthId for UTXOAllAndSth {
        All => all,
        Sth => sth,
    }
);
