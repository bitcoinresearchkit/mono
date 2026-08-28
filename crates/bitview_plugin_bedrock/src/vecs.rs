mod compute;
mod import;
mod urpd;

use std::path::PathBuf;

use bitview_plugin::{Plugin, PluginGate, PluginStorage};
use bitview_traversable::Traversable;
use brk_types::Sats;
use derive_more::{Deref, DerefMut};
use vecdb::{Database, Rw, StorageMode};

use super::{DailyPercentilesVecs, ModeVecs, Modes, STORAGE, WeightedPair};

#[derive(Deref, DerefMut, Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    plugin_gate: PluginGate,
    #[traversable(skip)]
    db: Database,
    #[traversable(skip)]
    states_path: PathBuf,

    pub cost_basis: WeightedPair<DailyPercentilesVecs<M>>,

    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    modes: Modes<ModeVecs<M>>,
}

impl<M: StorageMode> Vecs<M> {
    fn resolve_age_value<T>(value: Option<T>, supply: Sats) -> Option<f64>
    where
        f64: From<T>,
    {
        match value.map(f64::from) {
            Some(value) if value.is_finite() => Some(value),
            _ if supply == Sats::ZERO => Some(0.0),
            _ => None,
        }
    }
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }

    fn gate(&self) -> &PluginGate {
        &self.plugin_gate
    }
}

#[cfg(test)]
mod tests {
    use brk_types::{Sats, StoredF64};
    use vecdb::Rw;

    use super::Vecs;

    #[test]
    fn empty_age_cohort_uses_zero_weight() {
        assert_eq!(
            Vecs::<Rw>::resolve_age_value::<StoredF64>(None, Sats::ZERO),
            Some(0.0)
        );
        assert_eq!(
            Vecs::<Rw>::resolve_age_value(Some(StoredF64::NAN), Sats::ZERO),
            Some(0.0)
        );
    }

    #[test]
    fn non_empty_age_cohort_requires_finite_weight() {
        let supply = Sats::from(1_u64);
        assert_eq!(
            Vecs::<Rw>::resolve_age_value::<StoredF64>(None, supply),
            None
        );
        assert_eq!(
            Vecs::<Rw>::resolve_age_value(Some(StoredF64::NAN), supply),
            None
        );
        assert_eq!(
            Vecs::<Rw>::resolve_age_value(Some(StoredF64::from(0.25)), supply),
            Some(0.25)
        );
    }
}
