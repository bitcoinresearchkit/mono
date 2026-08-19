mod import;

use bitview_plugin::{ComputePlugin, Plugin, PluginGate, PluginId};
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Height, Sats};
use vecdb::Exit;

use super::cached_dca_sats::CachedDcaSats;
use super::{class_vecs::ClassVecs, period_vecs::PeriodVecs};
use bitview_compute::LazyPreviousDeltaVec;

#[derive(Clone, Traversable)]
pub struct Vecs {
    #[traversable(skip)]
    plugin_gate: PluginGate,
    #[traversable(skip)]
    cached_dca_sats: CachedDcaSats,
    /// Satoshis purchased by investing 100 USD at each UTC daily close newly
    /// crossed at this block. It is zero within a day, includes every
    /// intervening daily purchase when block time skips days, and treats a
    /// missing or zero daily close as a zero purchase.
    pub sats_per_day: LazyPreviousDeltaVec<Height, Sats>,
    pub period: PeriodVecs,
    pub class: ClassVecs,
}

impl Plugin for Vecs {
    fn id(&self) -> PluginId {
        super::ID
    }

    fn gate(&self) -> &PluginGate {
        &self.plugin_gate
    }
}

impl Vecs {
    fn invalidate_cache(&self) {
        self.cached_dca_sats.invalidate();
    }
}

impl ComputePlugin for Vecs {
    type Dependencies<'a> = ();
    type Output = ();

    fn compute(&mut self, (): Self::Dependencies<'_>, _exit: &Exit) -> Result<Self::Output> {
        self.invalidate_cache();
        Ok(())
    }
}
