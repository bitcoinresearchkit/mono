use brk_error::Result;

use bitview_cohort::{
    AgeRangeId, AmountRangeId, ClassId, EntryId, EpochId, Filter, OVER_AMOUNT_FILTERS,
    SPENDABLE_TYPE_FILTERS, SpendableTypeId, UNDER_AMOUNT_FILTERS,
};
use bitview_traversable::Traversable;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{AnyStoredVec, ColumnId, Database, ReadableBoxedVec, Rw, StorageMode};

use bitview_compute::ColumnarValuePerBlockCumulativeRolling;

use super::super::UTXORows;
use super::CumulativeUTXOValueColumnarMetricWithoutAmountOrType;

#[derive(Traversable)]
pub struct CumulativeUTXOValueColumnarMetric<M: StorageMode = Rw> {
    /// Height-indexed matrices with one column per exact UTXO age range,
    /// ordered from youngest to oldest.
    pub age_range: ColumnarValuePerBlockCumulativeRolling<AgeRangeId, (), M>,
    /// Height-indexed matrices with one column per subsidy-halving epoch,
    /// ordered from earliest to latest.
    pub epoch: ColumnarValuePerBlockCumulativeRolling<EpochId, (), M>,
    /// Height-indexed matrices with one column per output-creation year, in
    /// chronological order.
    pub class: ColumnarValuePerBlockCumulativeRolling<ClassId, (), M>,
    /// Height-indexed matrices with discount-entry followed by premium-entry
    /// UTXOs.
    pub entry: ColumnarValuePerBlockCumulativeRolling<EntryId, (), M>,
    /// Height-indexed matrices with one column per exact UTXO value range,
    /// ordered from smallest to largest.
    pub amount_range: ColumnarValuePerBlockCumulativeRolling<AmountRangeId, (), M>,
    #[traversable(rename = "type")]
    /// Height-indexed matrices with one column per spendable BRK output type, in
    /// canonical `SpendableTypeId` order.
    pub type_: ColumnarValuePerBlockCumulativeRolling<SpendableTypeId, (), M>,
}

impl CumulativeUTXOValueColumnarMetric {
    pub fn forced_import(db: &Database, name: &str, version: Version) -> Result<Self> {
        let version = version + Version::ONE;
        Ok(Self {
            age_range: Self::import(db, &format!("utxos_{name}_by_age_range"), version)?,
            epoch: Self::import(db, &format!("{name}_by_epoch"), version)?,
            class: Self::import(db, &format!("{name}_by_class"), version)?,
            entry: Self::import(db, &format!("{name}_by_entry"), version)?,
            amount_range: Self::import(db, &format!("utxos_{name}_by_amount_range"), version)?,
            type_: Self::import(db, &format!("{name}_by_type"), version)?,
        })
    }

    fn import<C>(
        db: &Database,
        name: &str,
        version: Version,
    ) -> Result<ColumnarValuePerBlockCumulativeRolling<C, ()>>
    where
        C: ColumnId,
    {
        ColumnarValuePerBlockCumulativeRolling::forced_import(db, name, version, |_, _| ())
    }

    pub fn sources(
        &self,
        filter: &Filter,
        name: &str,
        version: Version,
    ) -> Option<(
        ReadableBoxedVec<Height, Sats>,
        ReadableBoxedVec<Height, Cents>,
    )> {
        self.direct_sources(filter, name, version)
            .or_else(|| self.amount_aggregate_sources(filter, name, version))
            .or_else(|| self.age_aggregate_sources(filter, name, version))
    }

    fn direct_sources(
        &self,
        filter: &Filter,
        name: &str,
        version: Version,
    ) -> Option<(
        ReadableBoxedVec<Height, Sats>,
        ReadableBoxedVec<Height, Cents>,
    )> {
        match filter {
            Filter::Amount(_) => AmountRangeId::matching(filter)
                .map(|column| Self::matrix_sources(&self.amount_range, name, version, [column])),
            Filter::Type(_) => SpendableTypeId::ALL
                .iter()
                .copied()
                .find(|column| column.select(&SPENDABLE_TYPE_FILTERS) == filter)
                .map(|column| Self::matrix_sources(&self.type_, name, version, [column])),
            _ => CumulativeUTXOValueColumnarMetricWithoutAmountOrType::direct_sources_from(
                &self.age_range,
                &self.epoch,
                &self.class,
                &self.entry,
                filter,
                name,
                version,
            ),
        }
    }

    fn amount_aggregate_sources(
        &self,
        filter: &Filter,
        name: &str,
        version: Version,
    ) -> Option<(
        ReadableBoxedVec<Height, Sats>,
        ReadableBoxedVec<Height, Cents>,
    )> {
        let filter = UNDER_AMOUNT_FILTERS
            .iter()
            .chain(OVER_AMOUNT_FILTERS.iter())
            .find(|candidate| *candidate == filter)?;
        Some(Self::matrix_sources(
            &self.amount_range,
            name,
            version,
            AmountRangeId::included_by(filter),
        ))
    }

    fn age_aggregate_sources(
        &self,
        filter: &Filter,
        name: &str,
        version: Version,
    ) -> Option<(
        ReadableBoxedVec<Height, Sats>,
        ReadableBoxedVec<Height, Cents>,
    )> {
        let columns = CumulativeUTXOValueColumnarMetricWithoutAmountOrType::age_columns(filter)?;
        Some(Self::matrix_sources(
            &self.age_range,
            name,
            version,
            columns,
        ))
    }

    fn matrix_sources<C>(
        matrix: &ColumnarValuePerBlockCumulativeRolling<C, ()>,
        name: &str,
        version: Version,
        columns: impl IntoIterator<Item = C>,
    ) -> (
        ReadableBoxedVec<Height, Sats>,
        ReadableBoxedVec<Height, Cents>,
    )
    where
        C: ColumnId,
    {
        CumulativeUTXOValueColumnarMetricWithoutAmountOrType::matrix_sources(
            matrix, name, version, columns,
        )
    }

    #[inline(always)]
    pub fn push_block(&mut self, sats: UTXORows<Sats>, cents: UTXORows<Cents>) {
        self.age_range.push_block(sats.age_range, cents.age_range);
        self.epoch.push_block(sats.epoch, cents.epoch);
        self.class.push_block(sats.class, cents.class);
        self.entry.push_block(sats.entry, cents.entry);
        self.amount_range
            .push_block(sats.amount_range, cents.amount_range);
        self.type_.push_block(sats.type_, cents.type_);
    }

    pub fn min_len(&self) -> usize {
        self.age_range
            .len()
            .min(self.epoch.len())
            .min(self.class.len())
            .min(self.entry.len())
            .min(self.amount_range.len())
            .min(self.type_.len())
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let Self {
            age_range,
            epoch,
            class,
            entry,
            amount_range,
            type_,
        } = self;
        let mut vecs = age_range.collect_vecs_mut();
        vecs.extend(epoch.collect_vecs_mut());
        vecs.extend(class.collect_vecs_mut());
        vecs.extend(entry.collect_vecs_mut());
        vecs.extend(amount_range.collect_vecs_mut());
        vecs.extend(type_.collect_vecs_mut());
        vecs
    }
}
