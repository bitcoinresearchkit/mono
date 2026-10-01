use crate::data::Data;
use bitview_transforms::{RatioCentsSignedCents, SatsToCents};
use brk_types::{Cents, CentsSigned, PartsPerMillionSigned32, Sats};
use vecdb::BinaryTransform;

/// The side prices and sentiment metrics share one invested-capital calculation.
#[derive(Clone, Copy)]
pub(crate) struct UnrealizedData {
    pub net_pnl: CentsSigned,
    pub nupl: PartsPerMillionSigned32,
    pub invested_capital_in_profit: Cents,
    pub invested_capital_in_loss: Cents,
    pub pain_index: Cents,
    pub greed_index: Cents,
    pub net_sentiment: CentsSigned,
    pub profit_per_coin: Cents,
    pub profit_per_dollar: Cents,
    pub loss_per_coin: Cents,
    pub loss_per_dollar: Cents,
}
impl UnrealizedData {
    pub fn new(spot: Cents, d: &Data) -> Self {
        let profit_value = d.supply_profit.as_u128() * spot.as_u128() / Sats::ONE_BTC_U128;
        let loss_value = d.supply_loss.as_u128() * spot.as_u128() / Sats::ONE_BTC_U128;
        let invested_profit = profit_value.saturating_sub(d.unrealized_profit.as_u128());
        let invested_loss = loss_value + d.unrealized_loss.as_u128();
        let profit_capitalized = d
            .capitalized_profit
            .inner()
            .checked_div(invested_profit * Sats::ONE_BTC_U128)
            .unwrap_or_default();
        let loss_capitalized = d
            .capitalized_loss
            .inner()
            .checked_div(invested_loss * Sats::ONE_BTC_U128)
            .unwrap_or_default();
        let greed_index = Cents::new(spot.as_u128().saturating_sub(profit_capitalized) as u64);
        let pain_index = Cents::new(loss_capitalized.saturating_sub(spot.as_u128()) as u64);
        let net_pnl =
            CentsSigned::new(d.unrealized_profit.inner() as i64 - d.unrealized_loss.inner() as i64);
        Self {
            net_pnl,
            nupl: RatioCentsSignedCents::apply(net_pnl, SatsToCents::apply(d.supply, spot)),
            invested_capital_in_profit: Cents::new(invested_profit as u64),
            invested_capital_in_loss: Cents::new(invested_loss as u64),
            pain_index,
            greed_index,
            net_sentiment: CentsSigned::new(
                i64::try_from(i128::from(greed_index.inner()) - i128::from(pain_index.inner()))
                    .expect("net sentiment overflowed CentsSigned"),
            ),
            profit_per_coin: Self::per_coin(spot, invested_profit, d.supply_profit),
            loss_per_coin: Self::per_coin(spot, invested_loss, d.supply_loss),
            profit_per_dollar: if invested_profit == 0 {
                spot
            } else {
                Cents::new(profit_capitalized as u64)
            },
            loss_per_dollar: if invested_loss == 0 {
                spot
            } else {
                Cents::new(loss_capitalized as u64)
            },
        }
    }
    fn per_coin(spot: Cents, invested: u128, supply: Sats) -> Cents {
        invested
            .checked_mul(Sats::ONE_BTC_U128)
            .and_then(|n| n.checked_div(supply.as_u128()))
            .map(|n| Cents::new(n as u64))
            .unwrap_or(spot)
    }
}
