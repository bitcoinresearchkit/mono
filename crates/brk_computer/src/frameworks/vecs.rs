use bitview_plugin::{Plugin, PluginGate, PluginId};
use brk_traversable::Traversable;
use vecdb::{Database, Rw, StorageMode};

use super::{coinflow, cointime};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    pub(crate) plugin_gate: PluginGate,
    #[traversable(skip)]
    pub(crate) db: Database,

    pub cointime: cointime::Vecs<M>,
    pub coinflow: coinflow::Vecs<M>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Send + Sync,
{
    fn id(&self) -> PluginId {
        PluginId::new(super::DB_NAME)
    }

    fn gate(&self) -> &PluginGate {
        &self.plugin_gate
    }
}
