use bitview_cohort::{AmountRange, AmountRangeId, CohortContext, CohortId};
use bitview_distribution::{
    families::{CountWithDeltas, CumulativeFiat, CumulativeValue},
    metrics::{CohortCapital, CohortSupply, ShareTotals},
};
use bitview_primitives::Count;
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, import_cached};
use brk_error::Result;
use brk_types::{Cents, CentsSats, Height, Sats, Version};
use vecdb::{ReadableBoxedVec, ReadableCloneableVec, ReadableVec, Rw, StorageMode, WritableVec};

use crate::{
    addr::ImportContext,
    state::{AddrCohortState, RealizedOps},
};

/// One balance band: the funded addresses whose balance falls in it, ids
/// `balance_<band>_...`.
#[derive(Traversable)]
pub struct BalanceVecs<M: StorageMode = Rw> {
    /// Funded addresses whose balance falls in the band.
    #[traversable(rename = "funded")]
    pub address_count: CountWithDeltas<M>,
    // Its amounts and values: none for the empty band, whose amounts are all zero.
    #[traversable(flatten)]
    pub values: Option<BalanceValues<M>>,
    pub utxos: BalanceUtxos<M>,
}

/// A band's amounts and what they're worth.
#[derive(Traversable)]
pub struct BalanceValues<M: StorageMode = Rw> {
    pub supply: CohortSupply<M>,
    pub capital: CohortCapital<M>,
    pub activity: BalanceActivity<M>,
    pub realized: BalanceRealized<M>,
}

#[derive(Traversable)]
pub struct BalanceUtxos<M: StorageMode = Rw> {
    /// Number of unspent outputs the band's addresses hold.
    pub count: CountWithDeltas<M>,
}

#[derive(Traversable)]
pub struct BalanceActivity<M: StorageMode = Rw> {
    /// Value the band's addresses spent in each block, each spend attributed by its address's
    /// balance immediately before that input is applied, so one address can contribute to
    /// several bands within a block. BTC representations use the spent output value; USD
    /// representations value it at the spending block's spot price.
    pub transfer_volume: CumulativeValue<M>,
}

#[derive(Traversable)]
pub struct BalanceRealized<M: StorageMode = Rw> {
    /// Profit realized by the band's spends, each attributed by its address's balance
    /// immediately before that input is applied: spending value minus creation-date value,
    /// counted only for profitable spends.
    pub profit: CumulativeFiat<Cents, M>,
    /// Loss realized by the band's spends, each attributed by its address's balance immediately
    /// before that input is applied: creation-date value minus spending value, counted only for
    /// losing spends.
    pub loss: CumulativeFiat<Cents, M>,
}

impl BalanceVecs {
    /// `totals` are the supply and capital all addresses hold, the band's share denominators.
    pub fn import(
        ctx: &ImportContext<'_>,
        cohort: CohortId,
        totals: ShareTotals<'_>,
    ) -> Result<Self> {
        let name = |metric: &str| CohortContext::Balance.metric_name(cohort, metric);
        let version = ctx.version + Version::ONE;
        let flow_version = ctx.version + Version::TWO;
        Ok(Self {
            address_count: CountWithDeltas::import(
                ctx.db,
                &name("address_count"),
                version,
                ctx.mappings,
                ctx.windows,
            )?,
            values: (cohort != CohortId::Amount(AmountRangeId::Zero))
                .then(|| -> Result<_> {
                    Ok(BalanceValues {
                        supply: CohortSupply::import(
                            ctx.db,
                            &name("supply"),
                            version,
                            ctx.mappings,
                            ctx.windows,
                            ctx.spot,
                            totals.supply,
                        )?,
                        capital: CohortCapital::import(
                            ctx.db,
                            name,
                            version,
                            ctx.mappings,
                            ctx.windows,
                            totals.capital,
                        )?,
                        activity: BalanceActivity {
                            transfer_volume: CumulativeValue::import(
                                ctx.db,
                                &name("transfer_volume"),
                                flow_version,
                                ctx.mappings,
                                ctx.windows,
                            )?,
                        },
                        realized: BalanceRealized {
                            profit: CumulativeFiat::import(
                                ctx.db,
                                &name("realized_profit"),
                                flow_version,
                                ctx.mappings,
                                ctx.windows,
                            )?,
                            loss: CumulativeFiat::import(
                                ctx.db,
                                &name("realized_loss"),
                                flow_version,
                                ctx.mappings,
                                ctx.windows,
                            )?,
                        },
                    })
                })
                .transpose()?,
            utxos: BalanceUtxos {
                count: CountWithDeltas::import(
                    ctx.db,
                    &name("utxo_count"),
                    version,
                    ctx.mappings,
                    ctx.windows,
                )?,
            },
        })
    }

    #[inline(always)]
    pub fn push(&mut self, state: &AddrCohortState, price: Cents) {
        let inner = &state.inner;
        self.address_count.push(Count::from(state.addr_count));
        self.utxos.count.push(Count::from(inner.supply.utxo_count));
        let Some(values) = &mut self.values else {
            return;
        };
        values.supply.total.push(inner.supply.value);
        values
            .activity
            .transfer_volume
            .push_block(inner.sent, inner.sent.value_at(price));
        values.capital.push(inner.realized.cap());
        values.realized.profit.push_block(inner.realized.profit());
        values.realized.loss.push_block(inner.realized.loss());
    }

    /// The band's state at the end of `height`, `None` without that block.
    pub fn restore(&self, state: &mut AddrCohortState, height: Height) -> Option<()> {
        state.inner.supply.value = match &self.values {
            Some(values) => values.supply.total.stored.collect_one(height)?,
            None => Sats::ZERO,
        };
        state.inner.supply.utxo_count = u64::from(self.utxos.count.stored.collect_one(height)?);
        state.addr_count = u64::from(self.address_count.stored.collect_one(height)?);
        Some(())
    }
}

/// The balance bands and the capital all addresses hold: the address root's capital and the bands'
/// capital share denominator.
#[derive(Traversable)]
pub struct Balances<M: StorageMode = Rw> {
    /// Address balance bands classify addresses by their aggregate balance.
    #[traversable(flatten)]
    pub bands: AmountRange<BalanceVecs<M>>,
    #[traversable(hidden)]
    pub capital: CachedSeries<Height, Cents, M>,
}

impl Balances {
    /// `address_supply` is the supply all addresses hold, the bands' supply share denominator.
    pub fn import(
        ctx: &ImportContext<'_>,
        address_supply: &ReadableBoxedVec<Height, Sats>,
    ) -> Result<Self> {
        let capital = import_cached(
            ctx.db,
            "address_capital_cents",
            ctx.version + Version::new(3),
        )?;
        let address_capital = capital.read_only_boxed_clone();
        let totals = ShareTotals {
            supply: address_supply,
            capital: &address_capital,
        };
        Ok(Self {
            bands: AmountRange::try_from_fn(|id| BalanceVecs::import(ctx, id.cohort(), totals))?,
            capital,
        })
    }

    /// Writes each band's values and the capital they hold together, summed exactly and
    /// converted to cents once.
    #[inline(always)]
    pub fn push(&mut self, states: &AmountRange<AddrCohortState>, price: Cents) {
        let mut capital = CentsSats::ZERO;
        for (band, state) in self.bands.iter_mut().zip(states.iter()) {
            band.push(state, price);
            capital += state.inner.realized.cap_raw();
        }
        self.capital.push(capital.to_cents());
    }
}
