mod by_class;
mod cached_dca_sats;
mod class_vecs;
mod dca_stack;
mod has;
mod lump_sum_stack;
mod period_vecs;
mod vecs;

use by_class::ByDcaClass;
pub use has::HasInvesting;
pub use vecs::Vecs;

use brk_types::Dollars;

pub const ID: bitview_plugin::PluginId = bitview_plugin::PluginId::new("investing");
const DCA_AMOUNT: Dollars = Dollars::mint(100.0);
