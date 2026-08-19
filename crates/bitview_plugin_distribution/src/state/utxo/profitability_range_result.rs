use brk_cohort::ByTerm;
use brk_types::{Cents, CentsSats, Sats};

#[derive(Debug, Clone, Copy, Default)]
pub struct ProfitabilityRangeResult {
    pub supply: ByTerm<Sats>,
    pub realized_cap: ByTerm<Cents>,
}

impl ProfitabilityRangeResult {
    pub fn from_all_and_sth(all_sats: u64, all_usd: u128, sth_sats: u64, sth_usd: u128) -> Self {
        let all_realized_cap = Self::cents(all_usd);
        let short_realized_cap = Self::cents(sth_usd);
        Self {
            supply: ByTerm {
                short: Sats::from(sth_sats),
                long: Sats::from(all_sats.saturating_sub(sth_sats)),
            },
            realized_cap: ByTerm {
                short: short_realized_cap,
                long: all_realized_cap - short_realized_cap,
            },
        }
    }

    #[inline(always)]
    fn cents(raw: u128) -> Cents {
        CentsSats::new(raw).to_cents_rounded()
    }
}
