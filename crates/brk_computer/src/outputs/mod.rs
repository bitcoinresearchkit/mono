pub mod by_type;
pub mod count;
pub mod spent;
pub mod unspent;
pub mod value;

mod compute;
mod import;

use brk_plugin::{Plugin, PluginGate};
use brk_traversable::Traversable;
use vecdb::{AnyVec, Database, Rw, StorageMode};

use crate::internal::LazyPerSecondWindows;

pub use by_type::Vecs as ByTypeVecs;
pub use count::Vecs as CountVecs;
pub use spent::Vecs as SpentVecs;
pub use unspent::Vecs as UnspentVecs;
pub use value::Vecs as ValueVecs;

pub const DB_NAME: &str = "outputs";

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    pub(crate) plugin_gate: PluginGate,
    #[traversable(skip)]
    pub(crate) db: Database,

    pub spent: SpentVecs<M>,
    pub count: CountVecs<M>,
    /// Transaction-output rate, including coinbase outputs.
    pub per_sec: LazyPerSecondWindows,
    pub unspent: UnspentVecs<M>,
    pub by_type: ByTypeVecs<M>,
    pub value: ValueVecs<M>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Send + Sync,
{
    fn id(&self) -> &'static str {
        DB_NAME
    }

    fn gate(&self) -> &PluginGate {
        &self.plugin_gate
    }

    fn mutates_existing(&self, vec: &dyn AnyVec) -> bool {
        std::ptr::addr_eq(vec, &self.spent.txin_index as &dyn AnyVec)
    }
}
