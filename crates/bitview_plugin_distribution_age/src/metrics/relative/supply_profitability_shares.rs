use bitview_cohort::{UTXOAggregate, UTXOAggregateId};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_transforms::RatioSats;
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, LazyPercentPerBlock, import_cached};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, PartsPerMillion32, Sats, Version};
use vecdb::{AnyStoredVec, BinaryTransform, Database, Rw, StorageMode};

use super::{RelativeSource, public_loss_share, public_profit_share, share_views};

const VERSION: Version = Version::ONE;

#[derive(Traversable)]
pub struct SupplyProfitabilityShares<M: StorageMode = Rw> {
    #[traversable(wrap = "supply/in_profit", rename = "share")]
    /// Share of an aggregate UTXO cohort's unspent satoshis whose creation price
    /// is less than or equal to the represented block's spot price. Exact
    /// break-even is assigned to profit. Returns zero when the cohort has no
    /// unspent supply.
    pub supply_in_profit_share: UTXOAggregate<LazyPercentPerBlock<PartsPerMillion32>>,
    #[traversable(wrap = "supply/in_loss", rename = "share")]
    /// Share of an aggregate UTXO cohort's unspent satoshis whose creation price
    /// is greater than the represented block's spot price. Returns zero when the
    /// cohort has no unspent supply.
    pub supply_in_loss_share: UTXOAggregate<LazyPercentPerBlock<PartsPerMillion32>>,
    #[traversable(hidden)]
    pub profit_share_source: UTXOAggregate<CachedSeries<Height, PartsPerMillion32, M>>,
}

impl SupplyProfitabilityShares {
    pub fn forced_import(db: &Database, version: Version, mappings: &MappingsVecs) -> Result<Self> {
        let version = version + VERSION;
        let profit_share_source = UTXOAggregate::try_from_fn(|id| {
            import_cached(
                db,
                &id.metric_name("supply_in_profit_share_ppm"),
                version + Version::ONE,
            )
        })?;
        let supply_in_profit_share = share_views(
            &profit_share_source,
            "supply_in_profit_share",
            version,
            public_profit_share,
            mappings,
        );
        let supply_in_loss_share = share_views(
            &profit_share_source,
            "supply_in_loss_share",
            version,
            public_loss_share,
            mappings,
        );

        Ok(Self {
            supply_in_profit_share,
            supply_in_loss_share,
            profit_share_source,
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
                    &source.supply.in_profit.sats.height,
                    &source.supply.total.sats.height,
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
    use brk_types::{Height, PartsPerMillion32, Sats};

    use super::{SupplyProfitabilityShares, public_loss_share, public_profit_share};

    #[test]
    fn derives_both_public_shares_from_profit_share() {
        let empty = SupplyProfitabilityShares::stored_profit_share(Sats::ZERO, Sats::ZERO);
        assert!(empty.is_nan());
        assert_eq!(
            public_profit_share(Height::ZERO, empty),
            PartsPerMillion32::ZERO
        );
        assert_eq!(
            public_loss_share(Height::ZERO, empty),
            PartsPerMillion32::ZERO
        );

        let profit_share =
            SupplyProfitabilityShares::stored_profit_share(Sats::new(25), Sats::new(100));
        assert_eq!(
            public_profit_share(Height::ZERO, profit_share),
            PartsPerMillion32::from(0.25)
        );
        assert_eq!(
            public_loss_share(Height::ZERO, profit_share),
            PartsPerMillion32::from(0.75)
        );
    }
}
