use brk_types::{Cents, CentsSats, CentsSigned, Sats};

#[derive(Clone, Default)]
pub struct RealizedBlockData {
    pub cap_raw: CentsSats,
    pub(crate) supply: Sats,
    pub cap: Cents,
    pub profit: Cents,
    pub loss: Cents,
    pub net_pnl: CentsSigned,
    pub value_destroyed: Cents,
}

impl RealizedBlockData {
    /// Compute mean creation price only for consumers that publish it.
    pub fn price(&self) -> Cents {
        self.cap_raw
            .as_u128()
            .checked_div(self.supply.as_u128())
            .map(|price| Cents::new(price as u64))
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn price_uses_unrounded_cap_and_is_zero_without_supply() {
        let data = RealizedBlockData {
            cap_raw: CentsSats::new(101 * 3 + 112 * 5),
            supply: Sats::new(8),
            ..Default::default()
        };
        assert_eq!(data.price(), Cents::new(107));
        assert_eq!(RealizedBlockData::default().price(), Cents::ZERO);
    }
}
