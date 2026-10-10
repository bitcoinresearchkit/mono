use bitview_distribution::state::UnrealizedState;
use bitview_primitives::PartsPerMillionSigned32;
use bitview_transforms::{Quotient, SatsToCents};
use brk_types::{Cents, CentsSigned};
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
        let (cap_in_profit, cap_in_loss) = UnrealizedState {
            supply_in_profit: d.supply_profit,
            supply_in_loss: d.supply_loss,
            unrealized_profit: d.unrealized_profit,
            unrealized_loss: d.unrealized_loss,
        }
        .capital_split(spot);
        let net_pnl =
            CentsSigned::new(d.unrealized_profit.inner() as i64 - d.unrealized_loss.inner() as i64);
        Self {
            net_pnl,
            nupl: Quotient::apply(net_pnl, SatsToCents::apply(d.supply, spot)),
            cap_in_profit,
            cap_in_loss,
        }
    }
}
