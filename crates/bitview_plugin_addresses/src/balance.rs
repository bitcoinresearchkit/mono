use bitview_cohort::{CohortContext, CohortId};
use bitview_distribution::{
    families::{CountWithDeltas, CumulativeFiat, CumulativeValue, Fiat},
    metrics::CohortSupply,
};
use bitview_primitives::Count;
use bitview_transforms::SatsToCents;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{AnyStoredVec, BinaryTransform, ReadableBoxedVec, ReadableVec, Rw, StorageMode};

use crate::{
    addr::ImportContext,
    state::{AddrCohortState, RealizedOps},
};

/// One balance band: the funded addresses whose balance falls in it, ids
/// `balance_<band>_...`.
#[derive(Traversable)]
pub struct BalanceVecs<M: StorageMode = Rw> {
    /// Funded addresses whose balance falls in the band.
    pub address_count: CountWithDeltas<M>,
    pub supply: CohortSupply<M>,
    pub outputs: BalanceOutputs<M>,
    pub activity: BalanceActivity<M>,
    pub realized: BalanceRealized<M>,
}

#[derive(Traversable)]
pub struct BalanceOutputs<M: StorageMode = Rw> {
    /// Number of unspent outputs the band's addresses hold.
    pub unspent_count: CountWithDeltas<M>,
}

#[derive(Traversable)]
pub struct BalanceActivity<M: StorageMode = Rw> {
    /// Value the band's addresses spent in each block. BTC representations use
    /// the spent output value; USD representations value it at the spending
    /// block's spot price.
    pub transfer_volume: CumulativeValue<M>,
}

#[derive(Traversable)]
pub struct BalanceRealized<M: StorageMode = Rw> {
    /// Creation-date value of the band's unspent outputs.
    pub cap: Fiat<Cents, M>,
    /// Profit realized by the band's spends: spending value minus creation-date
    /// value, counted only for profitable spends.
    pub profit: CumulativeFiat<Cents, M>,
    /// Loss realized by the band's spends: creation-date value minus spending
    /// value, counted only for losing spends.
    pub loss: CumulativeFiat<Cents, M>,
}

impl BalanceVecs {
    /// `address_supply` is the supply all addresses hold, the band supply's share
    /// denominator.
    pub fn import(
        ctx: &ImportContext<'_>,
        cohort: CohortId,
        address_supply: &ReadableBoxedVec<Height, Sats>,
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
            supply: CohortSupply::import(
                ctx.db,
                &name("supply"),
                version,
                ctx.mappings,
                ctx.windows,
                ctx.spot,
                address_supply,
            )?,
            outputs: BalanceOutputs {
                unspent_count: CountWithDeltas::import(
                    ctx.db,
                    &name("utxo_count"),
                    version,
                    ctx.mappings,
                    ctx.windows,
                )?,
            },
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
                cap: Fiat::import(ctx.db, &name("realized_cap"), version, ctx.mappings)?,
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
    }

    #[inline(always)]
    pub fn push(&mut self, state: &AddrCohortState, price: Cents) {
        let inner = &state.inner;
        self.address_count.push(Count::from(state.addr_count));
        self.supply.total.push(inner.supply.value);
        self.outputs
            .unspent_count
            .push(Count::from(inner.supply.utxo_count));
        self.activity
            .transfer_volume
            .push_block(inner.sent, SatsToCents::apply(inner.sent, price));
        self.realized.cap.push(inner.realized.cap());
        self.realized.profit.push_block(inner.realized.profit());
        self.realized.loss.push_block(inner.realized.loss());
    }

    /// The band's state at the end of `height`, `None` without that block.
    pub fn restore(&self, state: &mut AddrCohortState, height: Height) -> Option<()> {
        state.inner.supply.value = self.supply.total.stored.collect_one(height)?;
        state.inner.supply.utxo_count =
            u64::from(self.outputs.unspent_count.stored.collect_one(height)?);
        state.addr_count = u64::from(self.address_count.stored.collect_one(height)?);
        Some(())
    }

    pub fn vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        let [sats, cents] = self.activity.transfer_volume.stored_vecs_mut();
        [
            self.address_count.stored_mut(),
            self.supply.total.stored_mut(),
            self.outputs.unspent_count.stored_mut(),
            sats,
            cents,
            self.realized.cap.stored_mut(),
            self.realized.profit.stored_mut(),
            self.realized.loss.stored_mut(),
        ]
        .into_iter()
    }
}
