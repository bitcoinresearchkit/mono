use brk_plugin::{Plugin, PluginGate};
use brk_traversable::Traversable;
use brk_types::{Height, Sats};

use super::cached_dca_sats::CachedDcaSats;
use super::{class_vecs::ClassVecs, period_vecs::PeriodVecs};
use crate::internal::LazyPreviousDeltaVec;

#[derive(Clone, Traversable)]
pub struct Vecs {
    #[traversable(skip)]
    pub(crate) plugin_gate: PluginGate,
    #[traversable(skip)]
    pub(super) cached_dca_sats: CachedDcaSats,
    pub sats_per_day: LazyPreviousDeltaVec<Height, Sats>,
    pub period: PeriodVecs,
    pub class: ClassVecs,
}

impl Plugin for Vecs {
    fn id(&self) -> &'static str {
        super::DB_NAME
    }

    fn gate(&self) -> &PluginGate {
        &self.plugin_gate
    }
}

impl Vecs {
    pub(crate) fn invalidate_cache(&self) {
        self.cached_dca_sats.invalidate();
    }
}
