use brk_cohort::{
    CohortContext, UTXO_AGGREGATE_FILTERS, UTXO_AGGREGATE_NAMES, UTXOAggregate, UTXOAggregateId,
};
use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{Height, PartsPerMillion32, Sats, Version};
use vecdb::{
    AnyStoredVec, BinaryTransform, Database, Exit, LazyVec, PcoVec, ReadOnlyClone,
    ReadOnlyColumnarVec, ReadableCloneableVec, ReadableColumnarVec, Rw, StorageMode,
};

use crate::{
    indexes,
    internal::{ColumnarPerBlock, LazyPercentPerBlock, RatioSats},
};

use super::RelativeSource;

const VERSION: Version = Version::ONE;

#[derive(Traversable)]
pub struct SupplyProfitabilityShares<M: StorageMode = Rw> {
    #[traversable(wrap = "supply/in_profit", rename = "share")]
    /// Share of the selected cohort's unspent supply whose creation price is
    /// less than or equal to spot price.
    pub supply_in_profit_share: UTXOAggregate<LazyPercentPerBlock<PartsPerMillion32>>,
    #[traversable(wrap = "supply/in_loss", rename = "share")]
    /// Share of the selected cohort's unspent supply whose creation price is
    /// greater than spot price.
    pub supply_in_loss_share: UTXOAggregate<LazyPercentPerBlock<PartsPerMillion32>>,
    #[traversable(hidden)]
    pub profit_share_source: ColumnarPerBlock<PartsPerMillion32, UTXOAggregateId, (), M>,
}

impl SupplyProfitabilityShares {
    pub fn forced_import(db: &Database, version: Version, indexes: &indexes::Vecs) -> Result<Self> {
        let version = version + VERSION;
        let profit_share_source = ColumnarPerBlock::forced_import(
            db,
            "supply_in_profit_share_ppm_by_aggregate",
            version,
            |_| (),
        )?;
        let source = profit_share_source.height.read_only_clone();
        let supply_in_profit_share = Self::views(
            &source,
            "supply_in_profit_share",
            version,
            Self::public_profit_share,
            indexes,
        );
        let supply_in_loss_share = Self::views(
            &source,
            "supply_in_loss_share",
            version,
            Self::public_loss_share,
            indexes,
        );

        Ok(Self {
            supply_in_profit_share,
            supply_in_loss_share,
            profit_share_source,
        })
    }

    fn views(
        source: &ReadOnlyColumnarVec<PcoVec<Height, PartsPerMillion32>, UTXOAggregateId>,
        metric: &str,
        version: Version,
        compute: fn(Height, PartsPerMillion32) -> PartsPerMillion32,
        indexes: &indexes::Vecs,
    ) -> UTXOAggregate<LazyPercentPerBlock<PartsPerMillion32>> {
        UTXOAggregate::from_fn(|id| {
            let name = CohortContext::Utxo.metric_name(
                id.select(&UTXO_AGGREGATE_FILTERS),
                id.select(&UTXO_AGGREGATE_NAMES).id,
                metric,
            );
            let source = source.column(&format!("{name}_source"), version, id);
            let source = LazyVec::init(
                &format!("{name}_ppm_source"),
                version,
                source.read_only_boxed_clone(),
                compute,
            );
            LazyPercentPerBlock::from_height_source(&name, version, source, indexes)
        })
    }

    #[inline(always)]
    fn stored_profit_share(profit: Sats, total: Sats) -> PartsPerMillion32 {
        if total.is_zero() {
            PartsPerMillion32::NAN
        } else {
            RatioSats::apply(profit, total)
        }
    }

    #[inline(always)]
    fn public_profit_share(_: Height, profit_share: PartsPerMillion32) -> PartsPerMillion32 {
        if profit_share.is_nan() {
            PartsPerMillion32::ZERO
        } else {
            profit_share
        }
    }

    #[inline(always)]
    fn public_loss_share(_: Height, profit_share: PartsPerMillion32) -> PartsPerMillion32 {
        if profit_share.is_nan() {
            PartsPerMillion32::ZERO
        } else {
            PartsPerMillion32::ONE - profit_share
        }
    }

    pub fn compute(
        &mut self,
        max_from: Height,
        sources: &UTXOAggregate<RelativeSource<'_>>,
        exit: &Exit,
    ) -> Result<()> {
        self.profit_share_source.compute_columns2(
            max_from,
            |id| &id.select(sources).supply.in_profit.sats.height,
            |id| &id.select(sources).supply.total.sats.height,
            |_, profit, total| Self::stored_profit_share(profit, total),
            exit,
        )
    }

    pub fn stored_mut(&mut self) -> &mut dyn AnyStoredVec {
        self.profit_share_source.stored_mut()
    }
}

#[cfg(test)]
mod tests {
    use brk_types::{Height, PartsPerMillion32, Sats};

    use super::SupplyProfitabilityShares;

    #[test]
    fn derives_both_public_shares_from_profit_share() {
        let empty = SupplyProfitabilityShares::stored_profit_share(Sats::ZERO, Sats::ZERO);
        assert!(empty.is_nan());
        assert_eq!(
            SupplyProfitabilityShares::public_profit_share(Height::ZERO, empty),
            PartsPerMillion32::ZERO
        );
        assert_eq!(
            SupplyProfitabilityShares::public_loss_share(Height::ZERO, empty),
            PartsPerMillion32::ZERO
        );

        let profit_share =
            SupplyProfitabilityShares::stored_profit_share(Sats::new(25), Sats::new(100));
        assert_eq!(
            SupplyProfitabilityShares::public_profit_share(Height::ZERO, profit_share),
            PartsPerMillion32::from(0.25)
        );
        assert_eq!(
            SupplyProfitabilityShares::public_loss_share(Height::ZERO, profit_share),
            PartsPerMillion32::from(0.75)
        );
    }
}
