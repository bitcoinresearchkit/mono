use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{PartsPerMillion32, PartsPerMillionSigned64};
use bitview_transforms::Quotient;
use bitview_traversable::Traversable;
use bitview_vecs::{
    CachedSeries, LazyFiatPerBlock, LazyIndexedVec, LazyPercentPerBlock,
    LazyRollingDeltasFiatFromHeight, LazyWindowStartVec,
};
use brk_error::Result;
use brk_types::{Cents, CentsSigned, Height, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{
    AnyStoredVec, BinaryTransform, Database, ReadableCloneableVec, Rw, StorageMode, WritableVec,
};

use crate::families::import_stored;

/// A cohort's capital views over its stored cents, the mirror of its supply: the capital,
/// its jargon name, its change over each window and its share of all capital. `name` gives each
/// series name from its metric (`capital` -> `utxos_1d_to_1w_old_capital`).
#[derive(Clone, Traversable)]
pub struct CapitalViews {
    /// Capital: the cohort's unspent outputs valued at Bitcoin's spot price when each was
    /// created.
    pub total: LazyFiatPerBlock<Cents>,
    /// Realized cap: the capital, under its jargon name.
    realized_cap: LazyFiatPerBlock<Cents>,
    /// Change in the cohort's capital over a trailing window.
    delta: LazyRollingDeltasFiatFromHeight<Cents, CentsSigned, PartsPerMillionSigned64>,
    /// The cohort's share of the capital it is part of: all unspent outputs', or for address
    /// balance bands, all addresses'.
    share: LazyPercentPerBlock<PartsPerMillion32>,
}

impl CapitalViews {
    /// `all_capital` is the capital of every cohort together, the share's denominator.
    pub fn new(
        name: impl Fn(&str) -> String,
        version: Version,
        capital: &impl ReadableCloneableVec<Height, Cents>,
        all_capital: &impl ReadableCloneableVec<Height, Cents>,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Self {
        let fiat = |metric: &str| {
            LazyFiatPerBlock::from_cents_source(&name(metric), version, capital, mappings)
        };
        let share_name = name("capital_share");
        let source = LazyIndexedVec::new(
            &format!("{share_name}_ppm_source"),
            version,
            capital,
            all_capital,
            |_, capital, all_capital| Quotient::<PartsPerMillion32>::apply(capital, all_capital),
        );
        Self {
            total: fiat("capital"),
            realized_cap: fiat("realized_cap"),
            delta: LazyRollingDeltasFiatFromHeight::new(
                &name("capital_delta"),
                version + Version::TWO,
                capital,
                window_starts,
                mappings,
            ),
            share: LazyPercentPerBlock::from_height_source(&share_name, version, &source, mappings),
        }
    }
}

/// A cohort's capital, stored as `realized_cap_cents`, and its views.
#[derive(Deref, DerefMut, Traversable)]
pub struct CohortCapital<M: StorageMode = Rw> {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    views: CapitalViews,
    #[traversable(hidden)]
    pub stored: CachedSeries<Height, Cents, M>,
}

impl CohortCapital {
    pub fn import(
        db: &Database,
        name: impl Fn(&str) -> String,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        all_capital: &impl ReadableCloneableVec<Height, Cents>,
    ) -> Result<Self> {
        let stored = import_stored(db, &name("realized_cap_cents"), version)?;
        Ok(Self {
            views: CapitalViews::new(name, version, &stored, all_capital, mappings, window_starts),
            stored,
        })
    }

    #[inline(always)]
    pub fn push(&mut self, capital: Cents) {
        self.stored.push(capital);
    }

    pub fn stored_mut(&mut self) -> &mut dyn AnyStoredVec {
        &mut self.stored
    }
}
