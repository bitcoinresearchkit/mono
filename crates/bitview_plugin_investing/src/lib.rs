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

use bitview_plugin::{PluginId, PluginStorage};
use brk_types::{Dollars, Version};

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("investing"), Version::new(9));
pub const ID: PluginId = STORAGE.id();
const DCA_DOLLARS_PER_DAY: f64 = 100.0;
const DCA_AMOUNT: Dollars = Dollars::mint(DCA_DOLLARS_PER_DAY);
