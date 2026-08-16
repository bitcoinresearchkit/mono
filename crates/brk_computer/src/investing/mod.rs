mod by_class;
mod by_period;
mod cached_dca_sats;
mod class_vecs;
mod dca_stack;
mod import;
mod lump_sum_stack;
mod period_vecs;
mod vecs;

pub use by_class::*;
pub use by_period::*;
pub use vecs::Vecs;

use brk_types::Dollars;

pub const DB_NAME: &str = "investing";
pub(crate) const DCA_AMOUNT: Dollars = Dollars::mint(100.0);
