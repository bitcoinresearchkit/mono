#![doc = include_str!("../README.md")]

#[macro_use]
mod macros;

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
mod utxo;
mod utxo_aggregate;
mod utxo_all_and_sth;
mod utxo_and_addr_groups;
mod utxo_groups_without_amount;
mod utxo_groups_without_amount_or_type;
mod utxo_values;
mod with_addr_types;

pub use brk_types::{Age, Term};

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
pub use utxo::*;
pub use utxo_aggregate::*;
pub use utxo_all_and_sth::*;
pub use utxo_and_addr_groups::UTXOAndAddrGroups;
pub use utxo_groups_without_amount::*;
pub use utxo_groups_without_amount_or_type::*;
pub use utxo_values::UTXOValues;

pub use with_addr_types::WithAddrTypes;
mod utxo_group_core;
pub use utxo_group_core::UTXOGroupCore;
mod utxo_core_values;
pub use utxo_core_values::UTXOCoreValues;

mod age_crossings;
pub use age_crossings::for_each_age_crossing;
