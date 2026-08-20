/// Controls the level of state tracking for a cohort.
///
/// - `None`: No state tracking. Values are computed from stateful sub-cohorts.
/// - `PriceOnly`: Only tracks `price_to_amount` for percentile calculations.
///   Used by aggregate cohorts (all, sth, lth) that compute other values from sub-cohorts.
/// - `Full`: Full state tracking including supply, realized values, and `price_to_amount`.
///   Used by stateful cohorts like individual age ranges and epochs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StateLevel {
    #[default]
    None,
    PriceOnly,
    Full,
}

impl StateLevel {
    pub fn is_none(&self) -> bool {
        matches!(self, StateLevel::None)
    }

    pub fn is_price_only(&self) -> bool {
        matches!(self, StateLevel::PriceOnly)
    }

    pub fn is_full(&self) -> bool {
        matches!(self, StateLevel::Full)
    }

    pub fn has_price_to_amount(&self) -> bool {
        matches!(self, StateLevel::PriceOnly | StateLevel::Full)
    }
}
