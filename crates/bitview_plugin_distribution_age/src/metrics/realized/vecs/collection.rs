use bitview_cohort::CreationCohorts;
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::CentsSquaredSats;
use bitview_traversable::Traversable;
use bitview_vecs::{DisjointAgeSources, LazyWindowStartVec};
use brk_error::Result;
use brk_types::{CentsSats, Version};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

use super::{
    CumulativeNetRealizedByCohort, CumulativeRealizedByCohort, CumulativeValueDestroyedByCohort,
    RealizedCapByCohort,
};
use crate::metrics::RealizedBlockData;

#[derive(Traversable)]
pub struct RealizedVecs<M: StorageMode = Rw> {
    /// Creation-date value of a creation cohort's unspent outputs: the sum of each
    /// output's BTC value multiplied by Bitcoin's spot price when that output
    /// was created.
    pub cap: RealizedCapByCohort<M>,
    /// Profit realized by outputs from a creation cohort: spending
    /// value minus creation-date value, counted only for profitable spends.
    pub profit: CumulativeRealizedByCohort<M>,
    /// Loss realized by outputs from a creation cohort:
    /// creation-date value minus spending value, counted only for losing spends.
    pub loss: CumulativeRealizedByCohort<M>,
    /// Net realized profit and loss of outputs from a UTXO cohort when
    /// spent: realized profit minus realized loss.
    pub net_pnl: CumulativeNetRealizedByCohort<M>,
    /// Creation-date value destroyed by spent outputs from a UTXO cohort:
    /// the sum of each spent output's creation price multiplied by its BTC
    /// value.
    pub value_destroyed: CumulativeValueDestroyedByCohort<M>,
    /// Raw sum of creation price in cents per BTC multiplied by unspent
    /// satoshis for a disjoint age band. Dividing by 100,000,000 converts
    /// it to realized capitalization in cents; dividing by unspent satoshis
    /// gives realized price in cents per BTC. It is an intermediate product,
    /// not itself a capitalization or price.
    #[traversable(hidden)]
    pub cap_raw: DisjointAgeSources<CentsSats, M>,
    /// Raw sum of squared creation price in cents per BTC multiplied by unspent
    /// satoshis for a disjoint age band. Dividing it by the cohort's raw
    /// creation-price-times-satoshis sum gives capitalized price in cents per
    /// BTC. It is an intermediate product, not itself a capitalization or
    /// price.
    #[traversable(hidden)]
    pub capitalized_cap_raw: DisjointAgeSources<CentsSquaredSats, M>,
    /// Exact peak-regret product for each disjoint age band in this block.
    #[traversable(hidden)]
    pub peak_regret_raw: DisjointAgeSources<CentsSats, M>,
}

impl RealizedVecs {
    pub fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Box<Self>> {
        let cap_raw = DisjointAgeSources::import(db, "cap_raw", version)?;
        let capitalized_cap_raw = DisjointAgeSources::import(db, "capitalized_cap_raw", version)?;
        let cap = RealizedCapByCohort::import(db, "realized_cap", version, mappings)?;
        let profit = CumulativeRealizedByCohort::import(
            db,
            "realized_profit",
            version + Version::ONE,
            mappings,
            window_starts,
        )?;
        let loss = CumulativeRealizedByCohort::import(
            db,
            "realized_loss",
            version + Version::ONE,
            mappings,
            window_starts,
        )?;
        let net_pnl = CumulativeNetRealizedByCohort::import(db, version, mappings, window_starts)?;
        let value_destroyed = CumulativeValueDestroyedByCohort::import(
            db,
            version + Version::ONE,
            mappings,
            window_starts,
        )?;
        let peak_regret_raw = DisjointAgeSources::import(db, "peak_regret_raw", version)?;

        Ok(Box::new(Self {
            cap,
            profit,
            loss,
            net_pnl,
            value_destroyed,
            cap_raw,
            capitalized_cap_raw,
            peak_regret_raw,
        }))
    }

    #[inline(always)]
    pub fn push(&mut self, cohort_values: &CreationCohorts<RealizedBlockData>) {
        self.cap
            .stored
            .push(&cohort_values.map(|values| values.cap));
        self.profit
            .stored
            .push_block(cohort_values.map(|values| values.profit));
        self.loss
            .stored
            .push_block(cohort_values.map(|values| values.loss));
        self.net_pnl
            .stored
            .push_block(cohort_values.map(|values| values.net_pnl));
        self.value_destroyed
            .stored
            .push_block(cohort_values.map(|values| values.value_destroyed));
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut vecs = self.cap.stored.stored_vecs_mut().collect::<Vec<_>>();
        vecs.extend(self.profit.stored.stored_vecs_mut());
        vecs.extend(self.loss.stored.stored_vecs_mut());
        vecs.extend(self.net_pnl.stored.stored_vecs_mut());
        vecs.extend(self.value_destroyed.stored.stored_vecs_mut());
        vecs.extend(self.cap_raw.collect_vecs_mut());
        vecs.extend(self.capitalized_cap_raw.collect_vecs_mut());
        vecs.extend(self.peak_regret_raw.collect_vecs_mut());
        vecs
    }
}
