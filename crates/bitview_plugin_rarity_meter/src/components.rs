use bitview_cohort::{AgeAggregate, AgeAggregateId};
use bitview_plugin_coinflow::Vecs as CoinflowVecs;
use bitview_plugin_cointime::Vecs as CointimeVecs;
use bitview_plugin_holders::Vecs as HoldersVecs;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_price::Vecs as PriceVecs;
use bitview_primitives::Lengths;
use bitview_traversable::Traversable;
use bitview_vecs::{CostBasisByCoin, CostBasisByDollar, LazyPerBlock, Price, PriceWithRatio};
use brk_error::{Error, Result};
use brk_exit::Exit;
use brk_types::{Cents, Version};
use rayon::prelude::*;
use vecdb::{Database, Rw, StorageMode};

use super::{Component, lazy_component::LazyComponent};

/// The reference prices whose spot ratios the meter ranks: every holder cohort's four creation
/// prices, of its coins, its awake coins and its mobile coins, and the cointime prices. Each
/// component's series follow its price's id: its bands `<price>_rarity_<percentile>`
/// (`sth_median_realized_price_rarity_p1`), their ratios `<price>_ratio_rarity_<percentile>`; a
/// price with no stored spot ratio (a median, CVDD, the balanced and delta prices) also publishes
/// its price and a lazy spot ratio (`sth_median_realized_price`, `..._ratio`).
#[derive(Traversable)]
pub struct Components<M: StorageMode = Rw> {
    #[traversable(flatten)]
    cohorts: AgeAggregate<BySupply<M>, AllComponents<M>>,
}

/// All coins' prices, and the cointime prices.
#[derive(Traversable)]
pub struct AllComponents<M: StorageMode = Rw> {
    #[traversable(flatten)]
    cohort: BySupply<M>,
    /// Cointime vaulted price: realized price divided by one minus liveliness,
    /// where liveliness is cumulative coinblocks destroyed divided by
    /// cumulative coinblocks created.
    #[traversable(lead = "Vaulted price: the realized price, scaled up by how much of the supply sits unmoved.")]
    vaulted_price: Component<M>,
    /// Cointime True Market Mean: realized capitalization minus cumulative
    /// issuance-date subsidy value, divided by active supply; active supply is
    /// circulating supply multiplied by liveliness.
    #[traversable(lead = "True market mean: the realized cap without what miners received, per coin of the supply in use.")]
    true_market_mean: Component<M>,
    /// Cointime price: cumulative cointime value destroyed divided by cumulative coinblocks
    /// stored, as a price per BTC.
    #[traversable(lead = "Cointime price: the value of the holding time spent, for each unit stored.")]
    cointime_price: Component<M>,
    /// Active price: realized price divided by liveliness, cumulative coinblocks destroyed over
    /// cumulative coinblocks created.
    #[traversable(lead = "Active price: the realized price, scaled by how much of the supply is in use.")]
    active_price: Component<M>,
    /// CVDD: coin days destroyed valued at spot and summed, over the market's age in days and
    /// 6 million.
    #[traversable(lead = "CVDD: the value of all coin days destroyed, over the market's age in days and 6 million.")]
    cvdd: LazyComponent<M>,
    /// Balanced price: the realized price less the transferred price.
    #[traversable(lead = "Balanced price: the realized price less the transferred price.")]
    balanced_price: LazyComponent<M>,
    /// Delta price: realized capitalization less average capitalization, per coin.
    #[traversable(lead = "Delta price: the realized cap less the average cap, per coin.")]
    delta_price: LazyComponent<M>,
}

/// A cohort's prices: of its coins, and of its awake and mobile coins.
#[derive(Traversable)]
pub struct BySupply<M: StorageMode = Rw> {
    #[traversable(flatten)]
    raw: CohortComponents<M>,
    /// The cohort's awake supply: each age range weighted by its wakefulness.
    #[traversable(lead = "The awake supply: coins weighted by how much of their holding time has been spent.")]
    awake: CohortComponents<M>,
    /// The cohort's mobile supply: each age range weighted by its mobility.
    #[traversable(lead = "The mobile supply: coins weighted by how likely they are to be spent.")]
    mobile: CohortComponents<M>,
}

/// A supply's means and medians.
#[derive(Traversable)]
pub struct CohortComponents<M: StorageMode = Rw> {
    /// Realized price: the satoshi-weighted mean creation price.
    #[traversable(lead = "Realized price: what the bitcoin held cost, on average.")]
    realized_price: Component<M>,
    /// Capitalized price: the mean creation price weighted by value at creation.
    #[traversable(lead = "Capitalized price: what the bitcoin held cost, on average, each coin weighted by the dollars it cost.")]
    capitalized_price: Component<M>,
    /// The median creation price, weighted by BTC supply.
    #[traversable(lead = "Median realized price: about half the coins last moved below it, half above.")]
    median_realized_price: LazyComponent<M>,
    /// The median creation price, weighted by creation-date USD value.
    #[traversable(lead = "Median capitalized price: about half the coins, by the dollars they cost, last moved below it.")]
    median_capitalized_price: LazyComponent<M>,
}

struct Imports<'a> {
    db: &'a Database,
    version: Version,
    mappings: &'a MappingsVecs,
    price: &'a PriceVecs,
}

impl Imports<'_> {
    /// A mean's component over its owner's price and stored spot ratio.
    fn mean(&self, name: &str, price: &PriceWithRatio) -> Result<Component> {
        Component::import(
            self.db,
            name,
            self.version,
            self.mappings,
            &price.cents.height,
            &price.relative.fixed.height,
        )
    }

    /// The component of a price with no stored spot ratio, with a lazy one.
    fn lazy(&self, name: &str, price: &Price<LazyPerBlock<Cents>>) -> Result<LazyComponent> {
        LazyComponent::import(
            self.db,
            name,
            self.version,
            self.mappings,
            &price.cents.height,
            &self.price.spot.cents.height,
        )
    }

    /// One cohort's components, its supply's prices from each owner's cost basis.
    fn by_supply(
        &self,
        id: AgeAggregateId,
        [raw, awake, mobile]: [(&CostBasisByCoin, &CostBasisByDollar); 3],
    ) -> Result<BySupply> {
        let cohort = |supply: &str, (by_coin, by_dollar): (&CostBasisByCoin, &CostBasisByDollar)| {
            let name = |basis: &str| id.metric_name(&format!("{supply}{basis}"));
            Ok::<_, Error>(CohortComponents {
                realized_price: self.mean(&name("realized_price"), &by_coin.avg)?,
                capitalized_price: self.mean(&name("capitalized_price"), &by_dollar.avg)?,
                median_realized_price: self
                    .lazy(&name("median_realized_price"), &by_coin.percentiles.median)?,
                median_capitalized_price: self
                    .lazy(&name("median_capitalized_price"), &by_dollar.percentiles.median)?,
            })
        };
        Ok(BySupply {
            raw: cohort("", raw)?,
            awake: cohort("awake_", awake)?,
            mobile: cohort("mobile_", mobile)?,
        })
    }
}

impl Components {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        holders: &HoldersVecs,
        cointime: &CointimeVecs,
        coinflow: &CoinflowVecs,
        price: &PriceVecs,
    ) -> Result<Self> {
        let imports = Imports {
            db,
            version,
            mappings,
            price,
        };
        let raw = &holders.cohorts;
        let awake = &cointime.aggregate.cohorts;
        let mobile = &coinflow.aggregate.cohorts;
        // Each cohort's raw, awake and mobile cost bases, one row per cohort.
        macro_rules! by_supply {
            ($id:ident, $cohort:ident) => {
                imports.by_supply(
                    AgeAggregateId::$id,
                    [
                        (&raw.$cohort.cost_basis.by_coin, &raw.$cohort.cost_basis.by_dollar),
                        (
                            &awake.$cohort.awake.cost_basis.by_coin,
                            &awake.$cohort.awake.cost_basis.by_dollar,
                        ),
                        (
                            &mobile.$cohort.mobile.cost_basis.by_coin,
                            &mobile.$cohort.mobile.cost_basis.by_dollar,
                        ),
                    ],
                )?
            };
        }
        Ok(Self {
            cohorts: AgeAggregate {
                all: AllComponents {
                    cohort: by_supply!(All, all),
                    vaulted_price: imports.mean("vaulted_price", &cointime.prices.vaulted)?,
                    true_market_mean: imports
                        .mean("true_market_mean", &cointime.prices.true_market_mean)?,
                    cointime_price: imports.mean("cointime_price", &cointime.prices.cointime)?,
                    active_price: imports.mean("active_price", &cointime.prices.active)?,
                    cvdd: imports.lazy("cvdd", &cointime.prices.cvdd)?,
                    balanced_price: imports.lazy("balanced_price", &cointime.prices.balanced)?,
                    delta_price: imports.lazy("delta_price", &cointime.prices.delta)?,
                },
                sth: by_supply!(Sth, sth),
                lth: by_supply!(Lth, lth),
                under_4m: by_supply!(Under4M, under_4m),
                under_6m: by_supply!(Under6M, under_6m),
                over_4m: by_supply!(Over4M, over_4m),
                over_6m: by_supply!(Over6M, over_6m),
            },
        })
    }

    /// Every component, each from where it resumes.
    pub(crate) fn compute(&mut self, starting_lengths: &Lengths, exit: &Exit) -> Result<()> {
        let components: Vec<_> = self.iter_mut().collect();
        components
            .into_par_iter()
            .try_for_each(|component| component.compute(starting_lengths, exit))
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut Component> {
        let AgeAggregate {
            all:
                AllComponents {
                    cohort: all,
                    vaulted_price,
                    true_market_mean,
                    cointime_price,
                    active_price,
                    cvdd,
                    balanced_price,
                    delta_price,
                },
            sth,
            lth,
            under_4m,
            under_6m,
            over_4m,
            over_6m,
        } = &mut self.cohorts;
        [all, sth, lth, under_4m, under_6m, over_4m, over_6m]
            .into_iter()
            .flat_map(|cohort| [&mut cohort.raw, &mut cohort.awake, &mut cohort.mobile])
            .flat_map(|supply| {
                [
                    &mut supply.realized_price,
                    &mut supply.capitalized_price,
                    &mut supply.median_realized_price.component,
                    &mut supply.median_capitalized_price.component,
                ]
            })
            .chain([
                vaulted_price,
                true_market_mean,
                cointime_price,
                active_price,
                &mut cvdd.component,
                &mut balanced_price.component,
                &mut delta_price.component,
            ])
    }

    pub(crate) fn local(&self) -> [&Component; 6] {
        let sth = &self.cohorts.sth.raw;
        let under_6m = &self.cohorts.under_6m.raw;
        [
            &sth.realized_price,
            &sth.capitalized_price,
            &under_6m.realized_price,
            &under_6m.capitalized_price,
            &sth.median_realized_price.component,
            &sth.median_capitalized_price.component,
        ]
    }

    pub(crate) fn cycle(&self) -> [&Component; 10] {
        let AllComponents {
            cohort,
            vaulted_price,
            true_market_mean,
            ..
        } = &self.cohorts.all;
        [
            &cohort.raw.realized_price,
            &cohort.raw.capitalized_price,
            &cohort.raw.median_realized_price.component,
            &cohort.raw.median_capitalized_price.component,
            &cohort.mobile.median_realized_price.component,
            &cohort.mobile.median_capitalized_price.component,
            vaulted_price,
            true_market_mean,
            &cohort.awake.realized_price,
            &cohort.mobile.realized_price,
        ]
    }
}
