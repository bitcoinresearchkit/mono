use bitview_plugin::ImportContext;
use bitview_plugin_distribution_aggregated::Vecs as AggregatedVecs;
use bitview_plugin_distribution_common::AllChainSources;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_mining::Vecs as MiningVecs;
use bitview_plugin_transactions::Vecs as TransactionsVecs;
use bitview_primitives::{BasisPoints32, Days, Float32, Years};
use bitview_vecs::{
    BasisPointsPerBlock, LazyBasisPointsPerBlock, LazyPerBlock, PerBlock, PercentPerBlock,
    RatioPerBlock,
};
use brk_error::Result;
use brk_types::{Bitcoin, Cents, Sats, Version};
use vecdb::{Ident, unlikely};

use crate::{STORAGE, Vecs, dormancy_vecs::DormancyVecs};

const COINYEARS_DESTROYED_SUPPLY_ADJ_VERSION: Version = Version::ONE;

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &MappingsVecs,
        all_chain: &AllChainSources,
        mining: &MiningVecs,
        distribution_aggregated: &AggregatedVecs,
        transactions: &TransactionsVecs,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 100_000)?;
        let v = STORAGE.schema_version();

        let bps_version = v + Version::ONE;
        let puell_multiple =
            BasisPointsPerBlock::import(&db, "puell_multiple", bps_version, mappings)?;
        let nvt_source = all_chain.with_market_cap(
            "nvt_bps_source",
            bps_version,
            &transactions
                .volume
                .transfer_volume
                .rolling
                .sum
                ._24h
                .cents
                .height,
            |_, volume, market_cap| Self::market_ratio(market_cap, volume),
        );
        let nvt =
            LazyBasisPointsPerBlock::from_height_source("nvt", bps_version, &nvt_source, mappings);
        let gini = PercentPerBlock::import(&db, "gini", v, mappings)?;
        let rhodl_ratio = RatioPerBlock::import_ppm(&db, "rhodl_ratio", v, mappings)?;
        let thermo_source = all_chain.with_market_cap(
            "thermo_cap_multiple_bps_source",
            bps_version,
            mining
                .rewards
                .subsidy
                .cumulative
                .cents
                .resolutions
                .height_source(),
            |_, thermo_cap, market_cap| Self::market_ratio(market_cap, thermo_cap),
        );
        let thermo_cap_multiple = LazyBasisPointsPerBlock::from_height_source(
            "thermo_cap_multiple",
            bps_version,
            &thermo_source,
            mappings,
        );

        let activity = &distribution_aggregated.cohorts.all.activity;
        let cdd_source = all_chain.with_supply(
            "coindays_destroyed_supply_adj_source",
            v,
            &activity.coindays_destroyed.sum._24h.height,
            |_, cdd, supply| Days::new(Self::supply_adjusted(f64::from(cdd), supply)),
        );
        let coindays_destroyed_supply_adj = LazyPerBlock::from_height_source::<Ident>(
            "coindays_destroyed_supply_adj",
            v,
            &cdd_source,
            mappings,
        );
        let cyd_version = v + COINYEARS_DESTROYED_SUPPLY_ADJ_VERSION;
        let cyd_source = all_chain.with_supply(
            "coinyears_destroyed_supply_adj_source",
            cyd_version,
            &activity.coinyears_destroyed.height,
            |_, cyd, supply| Years::new(Self::supply_adjusted(f64::from(cyd), supply)),
        );
        let coinyears_destroyed_supply_adj = LazyPerBlock::from_height_source::<Ident>(
            "coinyears_destroyed_supply_adj",
            cyd_version,
            &cyd_source,
            mappings,
        );
        let dormancy_24h = distribution_aggregated
            .cohorts
            .all
            .ratios
            .dormancy
            ._24h
            .resolutions
            .height_source();
        let dormancy_supply_source = all_chain.with_supply(
            "dormancy_supply_adj_source",
            v,
            dormancy_24h,
            |_, dormancy, supply| Float32::new(Self::supply_adjusted(f64::from(dormancy), supply)),
        );
        let dormancy_flow_source = all_chain.with_supply(
            "dormancy_flow_source",
            v,
            dormancy_24h,
            |_, dormancy, supply| Self::dormancy_flow(dormancy, supply),
        );
        let dormancy = DormancyVecs {
            supply_adj: LazyPerBlock::from_height_source::<Ident>(
                "dormancy_supply_adj",
                v,
                &dormancy_supply_source,
                mappings,
            ),
            flow: LazyPerBlock::from_height_source::<Ident>(
                "dormancy_flow",
                v,
                &dormancy_flow_source,
                mappings,
            ),
        };
        let stock_source = all_chain.with_supply(
            "stock_to_flow_source",
            v,
            &mining.rewards.subsidy.block.sats,
            |_, subsidy, supply| Self::stock_to_flow(supply, subsidy),
        );
        let stock_to_flow =
            LazyPerBlock::from_height_source::<Ident>("stock_to_flow", v, &stock_source, mappings);
        let seller_exhaustion = PerBlock::import(&db, "seller_exhaustion", v, mappings)?;

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
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }

    fn market_ratio(numerator: Cents, denominator: Cents) -> BasisPoints32 {
        let ratio = f64::from(numerator) / f64::from(denominator);
        if unlikely(!ratio.is_finite()) {
            BasisPoints32::ZERO
        } else {
            BasisPoints32::from(ratio)
        }
    }

    /// The value per bitcoin of supply; zero without supply.
    fn supply_adjusted(value: f64, supply: Sats) -> f32 {
        let supply = f64::from(Bitcoin::from(supply));
        if supply == 0.0 {
            0.0
        } else {
            (value / supply) as f32
        }
    }

    fn stock_to_flow(supply: Sats, subsidy: Sats) -> Years {
        let annual_flow = subsidy.as_u128() as f64 * 52_560.0;
        if annual_flow == 0.0 {
            Years::ZERO
        } else {
            Years::new((supply.as_u128() as f64 / annual_flow) as f32)
        }
    }

    fn dormancy_flow(dormancy: Days, supply: Sats) -> Float32 {
        let dormancy = f64::from(dormancy);
        if dormancy == 0.0 {
            Float32::ZERO
        } else {
            Float32::new((f64::from(Bitcoin::from(supply)) / dormancy) as f32)
        }
    }
}
