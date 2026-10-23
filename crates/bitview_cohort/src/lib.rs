#![doc = include_str!("../README.md")]

#[macro_use]
mod macros;

mod address_type;
mod age;
mod age_aggregate;
mod age_range;
mod amount_range;
mod by_entry;
mod by_term;
mod by_type;
mod cohort_context;
mod cohort_id;
mod cohort_name;
mod halving_epoch;
mod profitability_range;
mod spendable_type;
mod term;
mod unspendable_type;
mod vintage;

pub use address_type::*;
pub use age::Age;
pub use age_aggregate::{AgeAggregate, AgeAggregateId, BuildMember, UpdateMember};
pub use age_range::*;
pub use amount_range::*;
pub use by_entry::*;
pub use by_term::*;
pub use by_type::*;
pub use cohort_context::*;
pub use cohort_id::CohortId;
pub use cohort_name::*;
pub use halving_epoch::*;
pub use profitability_range::*;
pub use spendable_type::*;
pub use term::Term;
pub use unspendable_type::*;
pub use vintage::*;

mod age_crossings;
pub use age_crossings::{for_each_age_crossing, for_each_age_cutoff};
