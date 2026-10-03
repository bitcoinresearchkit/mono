#![doc = include_str!("../README.md")]

#[macro_use]
mod macros;

mod age_aggregate;
mod age_range;
mod amount_range;
mod by_addr_type;
mod by_entry;
mod by_epoch;
mod by_term;
mod by_type;
mod class;
mod cohort_context;
mod cohort_id;
mod cohort_name;
mod profitability_range;
mod spendable_type;
mod unspendable_type;
mod utxo_aggregate;
mod utxo_all_and_sth;
mod with_addr_types;

pub use brk_types::{Age, Term};

pub use age_aggregate::{AgeAggregate, AgeAggregateId};
pub use age_range::*;
pub use amount_range::*;
pub use by_addr_type::*;
pub use by_entry::*;
pub use by_epoch::*;
pub use by_term::*;
pub use by_type::*;
pub use class::*;
pub use cohort_context::*;
pub use cohort_id::CohortId;
pub use cohort_name::*;
pub use profitability_range::*;
pub use spendable_type::*;
pub use unspendable_type::*;
pub use utxo_aggregate::*;
pub use utxo_all_and_sth::*;

pub use with_addr_types::WithAddrTypes;
mod utxo_groups;
pub use utxo_groups::UtxoGroups;

mod age_crossings;
pub use age_crossings::{for_each_age_crossing, for_each_age_cutoff};

mod creation_cohorts;
pub use creation_cohorts::CreationCohorts;

/// Cohort-group markers, kept off the root: the client glob-imports this crate and
/// `bitview_types`, whose `Utxo` would become ambiguous (workspace builds unify the
/// `storage` feature).
#[cfg(feature = "storage")]
pub mod cohort_group;
#[cfg(feature = "storage")]
pub use cohort_group::CohortGroup;
