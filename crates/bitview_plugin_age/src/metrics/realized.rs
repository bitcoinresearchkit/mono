use bitview_cohort::{CohortContext, CohortId};
use bitview_collections::Windows;
use bitview_distribution::{
    families::{CumulativeFiat, Fiat},
    metrics::RealizedBlockData,
};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;
use brk_types::{Cents, CentsSigned, Version};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

#[derive(Traversable)]
pub struct RealizedVecs<M: StorageMode = Rw> {
    /// Creation-date value of the cohort's unspent outputs: the sum of each
    /// output's BTC value multiplied by Bitcoin's spot price when that output
    /// was created.
    pub cap: Fiat<Cents, M>,
    /// Profit realized by the cohort's outputs: spending value minus
    /// creation-date value, counted only for profitable spends.
    pub profit: CumulativeFiat<Cents, M>,
    /// Loss realized by the cohort's outputs: creation-date value minus
    /// spending value, counted only for losing spends.
    pub loss: CumulativeFiat<Cents, M>,
    /// Net realized profit and loss of the cohort's outputs when spent:
    /// realized profit minus realized loss.
    pub net_pnl: CumulativeFiat<CentsSigned, M>,
    /// Creation-date value destroyed by the cohort's spent outputs: the sum of
    /// each spent output's creation price multiplied by its BTC value.
    pub value_destroyed: CumulativeFiat<Cents, M>,
}

impl RealizedVecs {
    pub fn import(
        db: &Database,
        cohort: CohortId,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let name = |metric: &str| CohortContext::Utxo.metric_name(cohort, metric);
        let flow_version = version + Version::ONE;
        Ok(Self {
            cap: Fiat::import(db, &name("realized_cap"), version, mappings)?,
            profit: CumulativeFiat::import(
                db,
                &name("realized_profit"),
                flow_version,
                mappings,
                window_starts,
            )?,
            loss: CumulativeFiat::import(
                db,
                &name("realized_loss"),
                flow_version,
                mappings,
                window_starts,
            )?,
            net_pnl: CumulativeFiat::import(
                db,
                &name("net_realized_pnl"),
                flow_version,
                mappings,
                window_starts,
            )?,
            value_destroyed: CumulativeFiat::import(
                db,
                &name("value_destroyed"),
                flow_version,
                mappings,
                window_starts,
            )?,
        })
    }

    #[inline(always)]
    pub fn push(&mut self, values: &RealizedBlockData) {
        self.cap.push(values.cap);
        self.profit.push_block(values.profit);
        self.loss.push_block(values.loss);
        self.net_pnl.push_block(values.net_pnl);
        self.value_destroyed.push_block(values.value_destroyed);
    }

    pub fn stored_vecs_mut(&mut self) -> [&mut dyn AnyStoredVec; 5] {
        [
            self.cap.stored_mut(),
            self.profit.stored_mut(),
            self.loss.stored_mut(),
            self.net_pnl.stored_mut(),
            self.value_destroyed.stored_mut(),
        ]
    }
}
