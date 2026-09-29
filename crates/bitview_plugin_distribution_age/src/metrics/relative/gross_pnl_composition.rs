use bitview_cohort::{UTXOAggregate, UTXOAggregateId};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, LazyPercentPerBlock, import_cached};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Dollars, Height, PartsPerMillion32, PartsPerMillionSigned32, Version};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode};

use super::{RelativeSource, public_loss_share, public_profit_share, share_views};

const VERSION: Version = Version::ONE;

#[derive(Traversable)]
pub struct GrossPnlComposition<M: StorageMode = Rw> {
    #[traversable(wrap = "unrealized/profit", rename = "to_own_gross_pnl")]
    /// Share of an aggregate UTXO cohort's gross unrealized profit and loss
    /// attributable to profit: unrealized profit divided by unrealized profit
    /// plus unrealized loss. Returns zero when both are zero.
    pub unrealized_profit_to_own_gross_pnl: UTXOAggregate<LazyPercentPerBlock<PartsPerMillion32>>,
    #[traversable(wrap = "unrealized/loss", rename = "to_own_gross_pnl")]
    /// Share of an aggregate UTXO cohort's gross unrealized profit and loss
    /// attributable to loss: unrealized loss divided by unrealized profit plus
    /// unrealized loss. Returns zero when both are zero.
    pub unrealized_loss_to_own_gross_pnl: UTXOAggregate<LazyPercentPerBlock<PartsPerMillion32>>,
    #[traversable(wrap = "unrealized/net_pnl", rename = "to_own_gross_pnl")]
    /// Net composition of an aggregate UTXO cohort's gross unrealized profit
    /// and loss: `(unrealized profit - unrealized loss) / (unrealized profit +
    /// unrealized loss)`. It ranges from -1 for all loss to 1 for all profit;
    /// zero means equal profit and loss or no gross unrealized amount.
    pub net_unrealized_pnl_to_own_gross_pnl:
        UTXOAggregate<LazyPercentPerBlock<PartsPerMillionSigned32>>,
    #[traversable(hidden)]
    pub profit_share_source: UTXOAggregate<CachedSeries<Height, PartsPerMillion32, M>>,
}

impl GrossPnlComposition {
    pub fn forced_import(db: &Database, version: Version, mappings: &MappingsVecs) -> Result<Self> {
        let version = version + VERSION;
        let profit_share_source = UTXOAggregate::try_from_fn(|id| {
            import_cached(
                db,
                &id.metric_name("unrealized_profit_to_own_gross_pnl_ppm"),
                version + Version::ONE,
            )
        })?;
        let unrealized_profit_to_own_gross_pnl = share_views(
            &profit_share_source,
            "unrealized_profit_to_own_gross_pnl",
            version,
            public_profit_share,
            mappings,
        );
        let unrealized_loss_to_own_gross_pnl = share_views(
            &profit_share_source,
            "unrealized_loss_to_own_gross_pnl",
            version,
            public_loss_share,
            mappings,
        );
        let net_unrealized_pnl_to_own_gross_pnl = share_views(
            &profit_share_source,
            "net_unrealized_pnl_to_own_gross_pnl",
            version,
            Self::public_net_share,
            mappings,
        );

        Ok(Self {
            unrealized_profit_to_own_gross_pnl,
            unrealized_loss_to_own_gross_pnl,
            net_unrealized_pnl_to_own_gross_pnl,
            profit_share_source,
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
    fn public_net_share(_: Height, profit_share: PartsPerMillion32) -> PartsPerMillionSigned32 {
        if profit_share.is_nan() {
            PartsPerMillionSigned32::ZERO
        } else {
            PartsPerMillionSigned32::from(2.0 * f64::from(profit_share) - 1.0)
        }
    }

    pub fn compute(
        &mut self,
        max_from: Height,
        sources: &UTXOAggregate<RelativeSource<'_>>,
        exit: &Exit,
    ) -> Result<()> {
        for id in UTXOAggregateId::ALL {
            let source = id.select(sources);
            id.select_mut(&mut self.profit_share_source)
                .compute_transform2(
                    max_from,
                    &source.unrealized.profit.usd.height,
                    &source.unrealized_aggregate.gross_pnl.usd.height,
                    |(height, profit, total, _)| (height, Self::stored_profit_share(profit, total)),
                    exit,
                )?;
        }
        Ok(())
    }

    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        self.profit_share_source
            .iter_mut()
            .map(|v| v as &mut dyn AnyStoredVec)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use brk_types::{Dollars, Height, PartsPerMillion32, PartsPerMillionSigned32};

    use super::{GrossPnlComposition, public_loss_share, public_profit_share};

    #[test]
    fn derives_every_public_share_from_profit_composition() {
        let empty = GrossPnlComposition::stored_profit_share(Dollars::ZERO, Dollars::ZERO);
        assert!(empty.is_nan());
        assert_eq!(
            public_profit_share(Height::ZERO, empty),
            PartsPerMillion32::ZERO
        );
        assert_eq!(
            public_loss_share(Height::ZERO, empty),
            PartsPerMillion32::ZERO
        );
        assert_eq!(
            GrossPnlComposition::public_net_share(Height::ZERO, empty),
            PartsPerMillionSigned32::ZERO
        );

        let profit_share =
            GrossPnlComposition::stored_profit_share(Dollars::from(25.0), Dollars::from(100.0));
        assert_eq!(
            public_profit_share(Height::ZERO, profit_share),
            PartsPerMillion32::from(0.25)
        );
        assert_eq!(
            public_loss_share(Height::ZERO, profit_share),
            PartsPerMillion32::from(0.75)
        );
        assert_eq!(
            GrossPnlComposition::public_net_share(Height::ZERO, profit_share),
            PartsPerMillionSigned32::from(-0.5)
        );
    }
}
