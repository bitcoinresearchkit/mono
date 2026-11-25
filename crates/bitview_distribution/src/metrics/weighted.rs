use bitview_cohort::AgeAggregateId;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use std::ops::Add;

use bitview_primitives::{BoundedRatio, PartsPerMillion32, PartsPerMillionSigned32};
use bitview_traversable::Traversable;
use bitview_urpd::CostBasisVecs;
use bitview_vecs::{
    CachedSeries, LazyFiatPerBlock, LazyIndexedVec, LazyPercentPerBlock,
    LazyRollingDeltasAmountFromHeight, LazyRollingDeltasFiatFromHeight, LazySpotValuePerBlock,
    LazyWindowStartVec,
    ops::Quotient,
    shapes::Windows,
};
use brk_error::Result;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{
    BinaryTransform, Database, ReadableBoxedVec, ReadableCloneableVec, Rw, StorageMode,
};

use super::holding::{Capital, Part, Supply, Unrealized, net_pnl_source};
use crate::WeightedCohortSources;

/// A share, in parts per million of its whole, as a holder group's: of the same amounts, the same value.
type Share = LazyPercentPerBlock<PartsPerMillion32>;

/// A holder group's weighted coins (awake, mobile) in the group's own layout: each age range's
/// stock amounts (supply, capital, unrealized profit and loss) weighted by its model weight,
/// summed and rounded once. Flows from spent coins carry no weight and have no place here.
#[derive(Traversable)]
pub struct WeightedCohort<M: StorageMode = Rw> {
    pub supply: SharedSupply,
    pub capital: SharedCapital,
    pub unrealized: Unrealized,
    pub cost_basis: CostBasisVecs<M>,
    /// The supply in loss's share, of its rounded satoshis, for models calibrating on it.
    #[traversable(skip)]
    pub loss_share: ReadableBoxedVec<Height, BoundedRatio>,
}

/// The weighted coins' supply, beside their share of their group's. Named as holders' supply beside its share of
/// all: one shape, so one type in the clients.
#[derive(Clone, Traversable)]
pub struct SharedSupply {
    #[traversable(flatten)]
    pub supply: Supply<Share>,
    /// Share of the holder group's unspent supply that the weighted coins are; undefined (null)
    /// without supply.
    #[traversable(lead = "Supply share: their part of the group's supply.")]
    pub share: Share,
}

/// The weighted coins' capital, beside their share of their group's (named as holders', as the supply).
#[derive(Clone, Traversable)]
pub struct SharedCapital {
    #[traversable(flatten)]
    pub capital: Capital<Share>,
    /// Share of the holder group's capital that the weighted coins' is; undefined (null)
    /// without capital.
    #[traversable(lead = "Capital share: their part of the group's capital.")]
    pub share: Share,
}

impl WeightedCohort {
    /// `model` names the weighted side (`awake`: `sth_awake_supply`); `group_cap` is the holder
    /// group's realized cap in cents, its capital share's whole.
    #[allow(clippy::too_many_arguments)]
    pub fn import(
        db: &Database,
        id: AgeAggregateId,
        model: &str,
        version: Version,
        sources: &WeightedCohortSources,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        spot: &ReadableBoxedVec<Height, Cents>,
        group_cap: &ReadableBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let name = |metric: &str| id.metric_name(&format!("{model}_{metric}"));
        let supply_sats = id.select(&sources.weighted_supply);
        let loss_sats = id.select(&sources.weighted_supply_in_loss);
        let realized_cap = id.select(&sources.weighted_realized_cap);
        let capital_in_profit = id.select(&sources.weighted_capital_in_profit);
        let capital_in_loss = id.select(&sources.weighted_capital_in_loss);
        let profit = id.select(&sources.weighted_unrealized_profit);
        let loss = id.select(&sources.weighted_unrealized_loss);

        let total = LazySpotValuePerBlock::from_sats_source(
            &name("supply"),
            version,
            supply_sats,
            mappings,
            spot,
        );
        let profit_sats = LazyIndexedVec::new(
            &name("supply_in_profit_sats_source"),
            version,
            supply_sats,
            loss_sats,
            |_, supply: Sats, in_loss: Sats| supply - in_loss,
        );
        let loss_share = LazyIndexedVec::new(
            &name("supply_in_loss_share_bounded"),
            version,
            loss_sats,
            supply_sats,
            |_, in_loss: Sats, supply: Sats| Quotient::<BoundedRatio>::apply(in_loss, supply),
        )
        .read_only_boxed_clone();
        // Each part's share of the supply, as a holder group's.
        let spot_part = |metric: &str, sats: &ReadableBoxedVec<Height, Sats>| Part {
            value: LazySpotValuePerBlock::from_sats_source(
                &name(metric),
                version,
                sats,
                mappings,
                spot,
            ),
            share: Share::from_ratio::<Sats, Sats, Quotient<PartsPerMillion32>>(
                &format!("{}_share", name(metric)),
                version,
                sats,
                supply_sats,
                mappings,
            ),
        };
        let supply = SharedSupply {
            supply: Supply {
                total,
                in_profit: spot_part("supply_in_profit", &profit_sats.read_only_boxed_clone()),
                in_loss: spot_part("supply_in_loss", &loss_sats.read_only_boxed_clone()),
                delta: LazyRollingDeltasAmountFromHeight::new(
                    &name("supply_delta"),
                    version,
                    supply_sats,
                    window_starts,
                    mappings,
                ),
            },
            share: Share::from_ratio::<Sats, Sats, SideShare>(
                &name("supply_share"),
                version,
                supply_sats,
                id.select(&sources.complement_supply),
                mappings,
            ),
        };

        let fiat = |metric: &str, source: &CachedSeries<Cents>| {
            LazyFiatPerBlock::from_cents_source(&name(metric), version, source, mappings)
        };
        // Each side's share of both sides together, as a holder group's.
        let side = |metric: &str,
                    side: &CachedSeries<Cents>, other: &CachedSeries<Cents>| Part {
            value: fiat(metric, side),
            share: Share::from_ratio::<Cents, Cents, SideShare>(
                &format!("{}_share", name(metric)),
                version,
                side,
                other,
                mappings,
            ),
        };
        let capital = SharedCapital {
            capital: Capital {
                total: fiat("capital", realized_cap),
                realized_cap: fiat("realized_cap", realized_cap),
                in_profit: side("capital_in_profit", capital_in_profit, capital_in_loss),
                in_loss: side("capital_in_loss", capital_in_loss, capital_in_profit),
                delta: LazyRollingDeltasFiatFromHeight::new(
                    &name("capital_delta"),
                    version,
                    realized_cap,
                    window_starts,
                    mappings,
                ),
            },
            share: Share::from_ratio::<Cents, Cents, Quotient<PartsPerMillion32>>(
                &name("capital_share"),
                version,
                realized_cap,
                group_cap,
                mappings,
            ),
        };

        // NUPL: the net over the weighted coins' market value, as a holder group's.
        let market_cap = LazyIndexedVec::new(
            &name("market_cap_cents_source"),
            version,
            supply_sats,
            spot,
            |_, sats: Sats, spot: Cents| sats.value_at(spot),
        );
        let nupl = LazyIndexedVec::new(
            &name("nupl_ppm_source"),
            version,
            &net_pnl_source(&name("nupl_net_cents_source"), version, profit, loss),
            &market_cap,
            |_, net, market_cap: Cents| Quotient::<PartsPerMillionSigned32>::apply(net, market_cap),
        );
        let unrealized = Unrealized::new(name, version, profit, loss, &nupl, mappings);

        let cost_basis = CostBasisVecs::import(
            db,
            &id.metric_name(model),
            version,
            mappings,
            id.select(&sources.weighted_realized_price),
            id.select(&sources.weighted_capitalized_price),
            &supply.supply.total.market_cap.height,
            realized_cap,
        )?;

        Ok(Self {
            supply,
            capital,
            unrealized,
            cost_basis,
            loss_share,
        })
    }
}

/// One side's share of both sides together (see the holder groups' capital): a capital side of
/// both, the weighted coins of their group (weighted and complement).
struct SideShare;

impl<T> BinaryTransform<T, T, PartsPerMillion32> for SideShare
where
    T: Add<Output = T> + Copy,
    Quotient<PartsPerMillion32>: BinaryTransform<T, T, PartsPerMillion32>,
{
    #[inline(always)]
    fn apply(side: T, other: T) -> PartsPerMillion32 {
        Quotient::apply(side, side + other)
    }
}
