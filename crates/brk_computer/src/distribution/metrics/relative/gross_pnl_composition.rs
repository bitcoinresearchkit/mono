use brk_cohort::{UTXO_AGGREGATE_FILTERS, UTXO_AGGREGATE_NAMES, UTXOAggregate, UTXOAggregateId};
use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::{Dollars, Height, PartsPerMillion32, PartsPerMillionSigned32, Version};
use vecdb::{
    AnyStoredVec, Database, Exit, PcoVec, ReadOnlyClone, ReadOnlyColumnarVec, ReadableColumnarVec,
    Rw, StorageMode,
};

use crate::{
    distribution::metrics::utxo_metric_name,
    indexes,
    internal::{ColumnarPerBlock, FixedRatio, LazyPercentPerBlock},
};

use super::RelativeSource;

const VERSION: Version = Version::ONE;

#[derive(Traversable)]
pub struct GrossPnlComposition<M: StorageMode = Rw> {
    #[traversable(wrap = "unrealized/profit", rename = "to_own_gross_pnl")]
    pub unrealized_profit_to_own_gross_pnl: UTXOAggregate<LazyPercentPerBlock<PartsPerMillion32>>,
    #[traversable(wrap = "unrealized/loss", rename = "to_own_gross_pnl")]
    pub unrealized_loss_to_own_gross_pnl: UTXOAggregate<LazyPercentPerBlock<PartsPerMillion32>>,
    #[traversable(wrap = "unrealized/net_pnl", rename = "to_own_gross_pnl")]
    pub net_unrealized_pnl_to_own_gross_pnl:
        UTXOAggregate<LazyPercentPerBlock<PartsPerMillionSigned32>>,
    #[traversable(hidden)]
    pub profit_share_source: ColumnarPerBlock<PartsPerMillion32, UTXOAggregateId, (), M>,
}

impl GrossPnlComposition {
    pub(crate) fn forced_import(
        db: &Database,
        version: Version,
        indexes: &indexes::Vecs,
    ) -> Result<Self> {
        let version = version + VERSION;
        let profit_share_source = ColumnarPerBlock::forced_import(
            db,
            "unrealized_profit_to_own_gross_pnl_ppm_by_aggregate",
            version,
            |_| (),
        )?;
        let source = profit_share_source.height.read_only_clone();
        let unrealized_profit_to_own_gross_pnl = Self::views(
            &source,
            "unrealized_profit_to_own_gross_pnl",
            version,
            Self::public_profit_share,
            indexes,
        );
        let unrealized_loss_to_own_gross_pnl = Self::views(
            &source,
            "unrealized_loss_to_own_gross_pnl",
            version,
            Self::public_loss_share,
            indexes,
        );
        let net_unrealized_pnl_to_own_gross_pnl = Self::views(
            &source,
            "net_unrealized_pnl_to_own_gross_pnl",
            version,
            Self::public_net_share,
            indexes,
        );

        Ok(Self {
            unrealized_profit_to_own_gross_pnl,
            unrealized_loss_to_own_gross_pnl,
            net_unrealized_pnl_to_own_gross_pnl,
            profit_share_source,
        })
    }

    fn views<B: FixedRatio>(
        source: &ReadOnlyColumnarVec<PcoVec<Height, PartsPerMillion32>, UTXOAggregateId>,
        metric: &str,
        version: Version,
        compute: fn(Height, PartsPerMillion32) -> B,
        indexes: &indexes::Vecs,
    ) -> UTXOAggregate<LazyPercentPerBlock<B>> {
        UTXOAggregate::from_fn(|id| {
            let name = utxo_metric_name(
                id.select(&UTXO_AGGREGATE_FILTERS),
                id.select(&UTXO_AGGREGATE_NAMES).id,
                metric,
            );
            let source = source.column(&format!("{name}_source"), version, id);
            LazyPercentPerBlock::from_uncached_indexed_source(
                &name, version, &source, compute, indexes,
            )
        })
    }

    #[inline(always)]
    fn stored_profit_share(profit: Dollars, gross: Dollars) -> PartsPerMillion32 {
        if gross.is_zero() {
            PartsPerMillion32::NAN
        } else {
            PartsPerMillion32::from(f64::from(profit) / f64::from(gross))
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

    #[inline(always)]
    fn public_net_share(_: Height, profit_share: PartsPerMillion32) -> PartsPerMillionSigned32 {
        if profit_share.is_nan() {
            PartsPerMillionSigned32::ZERO
        } else {
            PartsPerMillionSigned32::from(2.0 * f64::from(profit_share) - 1.0)
        }
    }

    pub(crate) fn compute(
        &mut self,
        max_from: Height,
        sources: &UTXOAggregate<RelativeSource<'_>>,
        exit: &Exit,
    ) -> Result<()> {
        self.profit_share_source.compute_columns2(
            max_from,
            |id| &id.select(sources).unrealized.profit.usd.height,
            |id| &id.select(sources).unrealized_aggregate.gross_pnl.usd.height,
            |_, profit, gross| Self::stored_profit_share(profit, gross),
            exit,
        )
    }

    pub(crate) fn stored_mut(&mut self) -> &mut dyn AnyStoredVec {
        self.profit_share_source.stored_mut()
    }
}

#[cfg(test)]
mod tests {
    use brk_types::{Dollars, Height, PartsPerMillion32, PartsPerMillionSigned32};

    use super::GrossPnlComposition;

    #[test]
    fn derives_every_public_share_from_profit_composition() {
        let empty = GrossPnlComposition::stored_profit_share(Dollars::ZERO, Dollars::ZERO);
        assert!(empty.is_nan());
        assert_eq!(
            GrossPnlComposition::public_profit_share(Height::ZERO, empty),
            PartsPerMillion32::ZERO
        );
        assert_eq!(
            GrossPnlComposition::public_loss_share(Height::ZERO, empty),
            PartsPerMillion32::ZERO
        );
        assert_eq!(
            GrossPnlComposition::public_net_share(Height::ZERO, empty),
            PartsPerMillionSigned32::ZERO
        );

        let profit_share =
            GrossPnlComposition::stored_profit_share(Dollars::from(25.0), Dollars::from(100.0));
        assert_eq!(
            GrossPnlComposition::public_profit_share(Height::ZERO, profit_share),
            PartsPerMillion32::from(0.25)
        );
        assert_eq!(
            GrossPnlComposition::public_loss_share(Height::ZERO, profit_share),
            PartsPerMillion32::from(0.75)
        );
        assert_eq!(
            GrossPnlComposition::public_net_share(Height::ZERO, profit_share),
            PartsPerMillionSigned32::from(-0.5)
        );
    }
}
