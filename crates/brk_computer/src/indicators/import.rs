use std::path::Path;

use brk_error::Result;
use brk_types::{Bitcoin, Cents, PartsPerMillion64, Sats, StoredF32, Version};

use super::Vecs;
use crate::{
    distribution::{self, AllChainSources},
    indexes,
    internal::{
        Identity, LazyPerBlock, LazyRatioPerBlock, PerBlock, PercentPerBlock, RatioPerBlock,
        db_utils::{finalize_db, open_db},
    },
    mining, transactions,
};

const VERSION: Version = Version::new(1);

impl Vecs {
    pub(crate) fn forced_import(
        parent_path: &Path,
        parent_version: Version,
        indexes: &indexes::Vecs,
        all_chain: &AllChainSources,
        mining: &mining::Vecs,
        distribution: &distribution::Vecs,
        transactions: &transactions::Vecs,
    ) -> Result<Self> {
        let db = open_db(parent_path, super::DB_NAME, 100_000)?;
        let v = parent_version + VERSION;

        let puell_multiple = RatioPerBlock::forced_import_ppm(&db, "puell_multiple", v, indexes)?;
        let nvt_source = all_chain.with_market_cap(
            "nvt_ppm_source",
            v,
            &transactions.volume.transfer_volume.sum._24h.cents.height,
            |_, volume, market_cap| Self::market_ratio(market_cap, volume),
        );
        let nvt = LazyRatioPerBlock::from_uncached_height_source("nvt", v, nvt_source, indexes);
        let gini = PercentPerBlock::forced_import(&db, "gini", v, indexes)?;
        let rhodl_ratio = RatioPerBlock::forced_import_ppm(&db, "rhodl_ratio", v, indexes)?;
        let thermo_source = all_chain.with_market_cap(
            "thermo_cap_multiple_ppm_source",
            v,
            &mining.rewards.subsidy.cumulative.cents.height,
            |_, thermo_cap, market_cap| Self::market_ratio(market_cap, thermo_cap),
        );
        let thermo_cap_multiple = LazyRatioPerBlock::from_uncached_height_source(
            "thermo_cap_multiple",
            v,
            thermo_source,
            indexes,
        );

        let activity = &distribution.cohorts.activity;
        let cdd_source = all_chain.with_supply(
            "coindays_destroyed_supply_adj_source",
            v,
            &activity.coindays_destroyed.cohorts.all.sum._24h.height,
            |_, cdd, supply| Self::supply_adjusted(f64::from(cdd), supply),
        );
        let coindays_destroyed_supply_adj =
            LazyPerBlock::from_uncached_height_source::<Identity<StoredF32>, _>(
                "coindays_destroyed_supply_adj",
                v,
                cdd_source,
                indexes,
            );
        let cyd_source = all_chain.with_supply(
            "coinyears_destroyed_supply_adj_source",
            v,
            &activity.coinyears_destroyed.all.height,
            |_, cyd, supply| Self::supply_adjusted(f64::from(cyd), supply),
        );
        let coinyears_destroyed_supply_adj =
            LazyPerBlock::from_uncached_height_source::<Identity<StoredF32>, _>(
                "coinyears_destroyed_supply_adj",
                v,
                cyd_source,
                indexes,
            );
        let dormancy_24h = &activity.dormancy.all._24h.height;
        let dormancy_supply_source = all_chain.with_supply(
            "dormancy_supply_adj_source",
            v,
            dormancy_24h,
            |_, dormancy, supply| Self::supply_adjusted(f64::from(dormancy), supply),
        );
        let dormancy_flow_source = all_chain.with_supply(
            "dormancy_flow_source",
            v,
            dormancy_24h,
            |_, dormancy, supply| Self::dormancy_flow(dormancy, supply),
        );
        let dormancy = super::vecs::DormancyVecs {
            supply_adj: LazyPerBlock::from_uncached_height_source::<Identity<StoredF32>, _>(
                "dormancy_supply_adj",
                v,
                dormancy_supply_source,
                indexes,
            ),
            flow: LazyPerBlock::from_uncached_height_source::<Identity<StoredF32>, _>(
                "dormancy_flow",
                v,
                dormancy_flow_source,
                indexes,
            ),
        };
        let stock_source = all_chain.with_supply(
            "stock_to_flow_source",
            v,
            &mining.rewards.subsidy.block.sats,
            |_, subsidy, supply| Self::stock_to_flow(supply, subsidy),
        );
        let stock_to_flow = LazyPerBlock::from_uncached_height_source::<Identity<StoredF32>, _>(
            "stock_to_flow",
            v,
            stock_source,
            indexes,
        );
        let seller_exhaustion = PerBlock::forced_import(&db, "seller_exhaustion", v, indexes)?;

        let this = Self {
            db,
            puell_multiple,
            nvt,
            gini,
            rhodl_ratio,
            thermo_cap_multiple,
            coindays_destroyed_supply_adj,
            coinyears_destroyed_supply_adj,
            dormancy,
            stock_to_flow,
            seller_exhaustion,
        };
        finalize_db(&this.db, &this)?;
        Ok(this)
    }

    fn market_ratio(numerator: Cents, denominator: Cents) -> PartsPerMillion64 {
        let ratio = f64::from(numerator) / f64::from(denominator);
        if ratio.is_finite() {
            PartsPerMillion64::from(ratio)
        } else {
            PartsPerMillion64::default()
        }
    }

    fn supply_adjusted(value: f64, supply: Sats) -> StoredF32 {
        let supply = f64::from(Bitcoin::from(supply));
        if supply == 0.0 {
            StoredF32::from(0.0f32)
        } else {
            StoredF32::from((value / supply) as f32)
        }
    }

    fn stock_to_flow(supply: Sats, subsidy: Sats) -> StoredF32 {
        let annual_flow = subsidy.as_u128() as f64 * 52_560.0;
        if annual_flow == 0.0 {
            StoredF32::from(0.0f32)
        } else {
            StoredF32::from((supply.as_u128() as f64 / annual_flow) as f32)
        }
    }

    fn dormancy_flow(dormancy: StoredF32, supply: Sats) -> StoredF32 {
        let dormancy = f64::from(dormancy);
        if dormancy == 0.0 {
            StoredF32::from(0.0f32)
        } else {
            StoredF32::from((f64::from(Bitcoin::from(supply)) / dormancy) as f32)
        }
    }
}
