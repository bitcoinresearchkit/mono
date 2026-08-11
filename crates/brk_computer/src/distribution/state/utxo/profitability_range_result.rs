use brk_cohort::ByTerm;
use brk_types::{Dollars, Sats};

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ProfitabilityRangeResult {
    pub supply: ByTerm<Sats>,
    pub realized_cap: ByTerm<Dollars>,
}

impl ProfitabilityRangeResult {
    pub(super) fn from_all_and_sth(
        all_sats: u64,
        all_usd: u128,
        sth_sats: u64,
        sth_usd: u128,
    ) -> Self {
        let all_realized_cap = Self::dollars(all_usd);
        let short_realized_cap = Self::dollars(sth_usd);
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
    fn dollars(raw: u128) -> Dollars {
        Dollars::from(raw as f64 / 1e10)
    }
}
