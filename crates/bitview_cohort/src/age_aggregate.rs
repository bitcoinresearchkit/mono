use crate::{AgeRangeId, LTH_AGE_RANGE_IDS, STH_AGE_RANGE_IDS};

#[cfg(feature = "storage")]
use bitview_traversable::Traversable;

/// Overlapping age filters. A family whose members differ types them apart: `All`, the all
/// filter's type, and `Old`, the type of those holding no output under an hour old (`lth`,
/// `over_4m`, `over_6m`), are `T`, the others' (`sth`, `under_4m`, `under_6m`), unless a
/// family's all or old members hold less.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "storage", derive(Traversable))]
pub struct AgeAggregate<T, All = T, Old = T> {
    /// Uses all UTXOs.
    pub all: All,
    /// Uses short-term-holder UTXOs younger than 150 days.
    #[cfg_attr(feature = "storage", traversable(lead = "Short-term holders: coins that moved within the last 150 days."))]
    pub sth: T,
    /// Uses long-term-holder UTXOs at least 150 days old.
    #[cfg_attr(feature = "storage", traversable(lead = "Long-term holders: coins unmoved for 150 days or more."))]
    pub lth: Old,
    /// Younger than 120 days.
    #[cfg_attr(feature = "storage", traversable(lead = "Coins that moved within the last 4 months (120 days)."))]
    pub under_4m: T,
    /// Younger than 180 days.
    #[cfg_attr(feature = "storage", traversable(lead = "Coins that moved within the last 6 months (180 days)."))]
    pub under_6m: T,
    /// At least 120 days old.
    #[cfg_attr(feature = "storage", traversable(lead = "Coins unmoved for 4 months (120 days) or more."))]
    pub over_4m: Old,
    /// At least 180 days old.
    #[cfg_attr(feature = "storage", traversable(lead = "Coins unmoved for 6 months (180 days) or more."))]
    pub over_6m: Old,
}

define_cohort_id!(
    AgeAggregateId for AgeAggregate {
        All => all,
        Sth => sth,
        Lth => lth,
        Under4M => under_4m,
        Under6M => under_6m,
        Over4M => over_4m,
        Over6M => over_6m,
    }
);

impl AgeAggregateId {
    pub fn age_range_ids(self) -> &'static [AgeRangeId] {
        match self {
            Self::All => AgeRangeId::ALL,
            Self::Sth => STH_AGE_RANGE_IDS,
            Self::Lth => LTH_AGE_RANGE_IDS,
            Self::Under4M => &AgeRangeId::ALL[..AgeRangeId::From4MTo5M.index()],
            Self::Under6M => &AgeRangeId::ALL[..AgeRangeId::From6MTo9M.index()],
            Self::Over4M => &AgeRangeId::ALL[AgeRangeId::From4MTo5M.index()..],
            Self::Over6M => &AgeRangeId::ALL[AgeRangeId::From6MTo9M.index()..],
        }
    }

    pub fn contains(self, age: AgeRangeId) -> bool {
        let ages = self.age_range_ids();
        age >= ages[0] && age <= ages[ages.len() - 1]
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Sth => "sth",
            Self::Lth => "lth",
            Self::Under4M => "under_4m",
            Self::Under6M => "under_6m",
            Self::Over4M => "over_4m",
            Self::Over6M => "over_6m",
        }
    }

    /// The cohort's name, its series' prefix: bare for `all` and the terms (`sth`), and the age
    /// bands' `age_` frame for the cutoffs (`age_under_4m`).
    pub fn full_name(self) -> String {
        match self {
            Self::All | Self::Sth | Self::Lth => self.name().to_owned(),
            Self::Under4M | Self::Under6M | Self::Over4M | Self::Over6M => {
                format!("age_{}", self.name())
            }
        }
    }

    pub fn from_full_name(name: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|id| id.full_name() == name)
    }

    /// The cohort's id for `metric`: bare for `all`, else its name's (`age_under_4m_supply`).
    pub fn metric_name(self, metric: &str) -> String {
        match self {
            Self::All => metric.to_owned(),
            _ => format!("{}_{metric}", self.full_name()),
        }
    }
}

impl<T> AgeAggregate<T> {
    /// Each age belongs to all and exactly one side of each cutoff.
    pub fn containing_mut(&mut self, age: AgeRangeId) -> [&mut T; 4] {
        [
            &mut self.all,
            if AgeAggregateId::Sth.contains(age) {
                &mut self.sth
            } else {
                &mut self.lth
            },
            if AgeAggregateId::Under4M.contains(age) {
                &mut self.under_4m
            } else {
                &mut self.over_4m
            },
            if AgeAggregateId::Under6M.contains(age) {
                &mut self.under_6m
            } else {
                &mut self.over_6m
            },
        ]
    }
}
