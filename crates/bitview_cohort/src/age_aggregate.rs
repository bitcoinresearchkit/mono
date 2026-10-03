use crate::{AgeRangeId, UTXOAggregateId};

#[cfg(feature = "storage")]
use bitview_traversable::Traversable;

/// Overlapping age filters sharing the same metric layout.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "storage", derive(Traversable))]
pub struct AgeAggregate<T> {
    pub all: T,
    /// Uses short-term-holder UTXOs younger than 150 days.
    pub sth: T,
    /// Uses long-term-holder UTXOs at least 150 days old.
    pub lth: T,
    /// Younger than 120 days.
    pub under_4m: T,
    /// Younger than 180 days.
    pub under_6m: T,
    /// At least 120 days old.
    pub over_4m: T,
    /// At least 180 days old.
    pub over_6m: T,
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
            Self::All => UTXOAggregateId::All.age_range_ids(),
            Self::Sth => UTXOAggregateId::Sth.age_range_ids(),
            Self::Lth => UTXOAggregateId::Lth.age_range_ids(),
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

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|id| id.name() == name)
    }

    pub fn metric_name(self, metric: &str) -> String {
        if self == Self::All {
            metric.to_owned()
        } else {
            format!("{}_{metric}", self.name())
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
