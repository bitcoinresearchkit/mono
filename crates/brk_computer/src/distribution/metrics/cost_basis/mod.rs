use brk_error::Result;
use brk_indexer::Lengths;
use brk_traversable::Traversable;
use brk_types::CentsSquaredSats;
use brk_types::{Cents, Height, PartsPerMillion32, Sats, Version};
use vecdb::{AnyStoredVec, AnyVec, Exit, ReadableVec, Rw, StorageMode, WritableVec};

use crate::internal::{PERCENTILES_LEN, PerBlock, PercentPerBlock, PercentilesVecs, Price};

use super::ImportConfig;

#[derive(Traversable)]
pub struct CostBasisSide<M: StorageMode = Rw> {
    pub per_coin: Price<PerBlock<Cents, M>>,
    pub per_dollar: Price<PerBlock<Cents, M>>,
}

/// Cost basis metrics: min/max + profit/loss splits + percentiles + supply density.
/// Used by all/sth/lth cohorts only.
#[derive(Traversable)]
pub struct CostBasis<M: StorageMode = Rw> {
    pub in_profit: CostBasisSide<M>,
    pub in_loss: CostBasisSide<M>,
    pub min: Price<PerBlock<Cents, M>>,
    pub max: Price<PerBlock<Cents, M>>,
    pub per_coin: PercentilesVecs<M>,
    pub per_dollar: PercentilesVecs<M>,
    pub supply_density: PercentPerBlock<PartsPerMillion32, M>,
}

impl CostBasis {
    pub(crate) fn forced_import(cfg: &ImportConfig) -> Result<Self> {
        Ok(Self {
            in_profit: CostBasisSide {
                per_coin: Price::forced_import(
                    cfg.db,
                    &cfg.name("cost_basis_in_profit_per_coin"),
                    cfg.version + Version::ONE,
                    cfg.indexes,
                )?,
                per_dollar: Price::forced_import(
                    cfg.db,
                    &cfg.name("cost_basis_in_profit_per_dollar"),
                    cfg.version + Version::ONE,
                    cfg.indexes,
                )?,
            },
            in_loss: CostBasisSide {
                per_coin: Price::forced_import(
                    cfg.db,
                    &cfg.name("cost_basis_in_loss_per_coin"),
                    cfg.version + Version::ONE,
                    cfg.indexes,
                )?,
                per_dollar: Price::forced_import(
                    cfg.db,
                    &cfg.name("cost_basis_in_loss_per_dollar"),
                    cfg.version + Version::ONE,
                    cfg.indexes,
                )?,
            },
            min: cfg.import("cost_basis_min", Version::ZERO)?,
            max: cfg.import("cost_basis_max", Version::ZERO)?,
            per_coin: PercentilesVecs::forced_import(
                cfg.db,
                &cfg.name("cost_basis_per_coin"),
                cfg.version,
                cfg.indexes,
            )?,
            per_dollar: PercentilesVecs::forced_import(
                cfg.db,
                &cfg.name("cost_basis_per_dollar"),
                cfg.version,
                cfg.indexes,
            )?,
            supply_density: PercentPerBlock::forced_import(
                cfg.db,
                &cfg.name("supply_density"),
                cfg.version,
                cfg.indexes,
            )?,
        })
    }

    pub(crate) fn min_stateful_len(&self) -> usize {
        self.min
            .cents
            .height
            .len()
            .min(self.max.cents.height.len())
            .min(self.supply_density.ppm.height.len())
            .min(self.per_coin.prices.height.len())
            .min(self.per_dollar.prices.height.len())
    }

    #[inline(always)]
    pub(crate) fn push_minmax(&mut self, min_price: Cents, max_price: Cents) {
        self.min.cents.height.push(min_price);
        self.max.cents.height.push(max_price);
    }

    #[inline(always)]
    pub(crate) fn push_percentiles(
        &mut self,
        sat_prices: &[Cents; PERCENTILES_LEN],
        usd_prices: &[Cents; PERCENTILES_LEN],
    ) {
        self.per_coin.push(sat_prices);
        self.per_dollar.push(usd_prices);
    }

    #[inline(always)]
    pub(crate) fn push_density(&mut self, density: PartsPerMillion32) {
        self.supply_density.ppm.height.push(density);
    }

    pub(crate) fn validate_computed_versions(&mut self, base_version: Version) -> Result<()> {
        self.per_coin
            .validate_computed_version_or_reset(base_version)?;
        self.per_dollar
            .validate_computed_version_or_reset(base_version)?;
        Ok(())
    }

    pub(crate) fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut vecs: Vec<&mut dyn AnyStoredVec> = vec![
            &mut self.in_profit.per_coin.cents.height,
            &mut self.in_profit.per_dollar.cents.height,
            &mut self.in_loss.per_coin.cents.height,
            &mut self.in_loss.per_dollar.cents.height,
            &mut self.min.cents.height,
            &mut self.max.cents.height,
            &mut self.supply_density.ppm.height,
        ];
        vecs.push(self.per_coin.prices.stored_mut());
        vecs.push(self.per_dollar.prices.stored_mut());
        vecs
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compute_prices(
        &mut self,
        starting_lengths: &Lengths,
        spot: &impl ReadableVec<Height, Cents>,
        invested_cap_in_profit: &impl ReadableVec<Height, Cents>,
        invested_cap_in_loss: &impl ReadableVec<Height, Cents>,
        supply_in_profit_sats: &impl ReadableVec<Height, Sats>,
        supply_in_loss_sats: &impl ReadableVec<Height, Sats>,
        capitalized_cap_in_profit_raw: &impl ReadableVec<Height, CentsSquaredSats>,
        capitalized_cap_in_loss_raw: &impl ReadableVec<Height, CentsSquaredSats>,
        exit: &Exit,
    ) -> Result<()> {
        self.in_profit.per_coin.cents.height.compute_transform3(
            starting_lengths.height,
            invested_cap_in_profit,
            supply_in_profit_sats,
            spot,
            |(h, invested_cents, supply_sats, spot, ..)| {
                let supply = supply_sats.as_u128();
                if supply == 0 {
                    return (h, spot);
                }
                (
                    h,
                    Cents::new((invested_cents.as_u128() * Sats::ONE_BTC_U128 / supply) as u64),
                )
            },
            exit,
        )?;
        self.in_loss.per_coin.cents.height.compute_transform3(
            starting_lengths.height,
            invested_cap_in_loss,
            supply_in_loss_sats,
            spot,
            |(h, invested_cents, supply_sats, spot, ..)| {
                let supply = supply_sats.as_u128();
                if supply == 0 {
                    return (h, spot);
                }
                (
                    h,
                    Cents::new((invested_cents.as_u128() * Sats::ONE_BTC_U128 / supply) as u64),
                )
            },
            exit,
        )?;
        self.in_profit.per_dollar.cents.height.compute_transform3(
            starting_lengths.height,
            capitalized_cap_in_profit_raw,
            invested_cap_in_profit,
            spot,
            |(h, capitalized_cap, invested_cents, spot, ..)| {
                let invested_raw = invested_cents.as_u128() * Sats::ONE_BTC_U128;
                if invested_raw == 0 {
                    return (h, spot);
                }
                (
                    h,
                    Cents::new((capitalized_cap.inner() / invested_raw) as u64),
                )
            },
            exit,
        )?;
        self.in_loss.per_dollar.cents.height.compute_transform3(
            starting_lengths.height,
            capitalized_cap_in_loss_raw,
            invested_cap_in_loss,
            spot,
            |(h, capitalized_cap, invested_cents, spot, ..)| {
                let invested_raw = invested_cents.as_u128() * Sats::ONE_BTC_U128;
                if invested_raw == 0 {
                    return (h, spot);
                }
                (
                    h,
                    Cents::new((capitalized_cap.inner() / invested_raw) as u64),
                )
            },
            exit,
        )?;
        Ok(())
    }
}
