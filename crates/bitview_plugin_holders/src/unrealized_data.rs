use bitview_primitives::PartsPerMillionSigned32;
use bitview_transforms::{Quotient, SatsToCents};
use brk_types::{Cents, CentsSigned, Sats};
use vecdb::BinaryTransform;

use crate::data::Data;

/// The unrealized values that depend on the block's spot price.
#[derive(Clone, Copy)]
pub(crate) struct UnrealizedData {
    pub net_pnl: CentsSigned,
    pub nupl: PartsPerMillionSigned32,
    pub cap_in_profit: Cents,
    pub cap_in_loss: Cents,
}
impl UnrealizedData {
    pub fn new(spot: Cents, d: &Data) -> Self {
        let profit_value = d.supply_profit.as_u128() * spot.as_u128() / Sats::ONE_BTC_U128;
        let loss_value = d.supply_loss.as_u128() * spot.as_u128() / Sats::ONE_BTC_U128;
        let cap_in_profit = profit_value.saturating_sub(d.unrealized_profit.as_u128());
        let cap_in_loss = loss_value + d.unrealized_loss.as_u128();
        let net_pnl =
            CentsSigned::new(d.unrealized_profit.inner() as i64 - d.unrealized_loss.inner() as i64);
        Self {
            net_pnl,
            nupl: Quotient::apply(net_pnl, SatsToCents::apply(d.supply, spot)),
            cap_in_profit: Cents::new(cap_in_profit as u64),
            cap_in_loss: Cents::new(cap_in_loss as u64),
        }
    }
}
