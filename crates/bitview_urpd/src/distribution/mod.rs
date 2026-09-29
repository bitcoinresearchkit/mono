//! Build weighted price distributions from age-cohort buckets.
mod age_cutoffs;
mod buckets;
mod entries;

pub use age_cutoffs::AgeCutoffs;
pub use buckets::{COST_BASIS_PRICE_DIGITS, accumulate_masses, collect_mass};
pub use entries::{rounded_entries, weighted_entries};
