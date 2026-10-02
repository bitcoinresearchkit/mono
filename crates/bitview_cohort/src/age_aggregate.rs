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

#[cfg(test)]
mod tests {
    use super::*;
    use brk_types::Timestamp;

    use crate::{Age, HOURS_4M, HOURS_5M, HOURS_6M};

    #[test]
    fn filters_partition_each_boundary_once() {
        for (hours, under, over) in [
            (HOURS_4M, AgeAggregateId::Under4M, AgeAggregateId::Over4M),
            (HOURS_5M, AgeAggregateId::Sth, AgeAggregateId::Lth),
            (HOURS_6M, AgeAggregateId::Under6M, AgeAggregateId::Over6M),
        ] {
            for seconds in [
                hours as u32 * 3600 - 1,
                hours as u32 * 3600,
                hours as u32 * 3600 + 1,
            ] {
                let age = AgeRangeId::from(Age::new(Timestamp::new(seconds), Timestamp::ZERO));
                assert_eq!(under.contains(age), seconds < hours as u32 * 3600);
                assert_ne!(under.contains(age), over.contains(age));
            }
        }
        for &age in AgeRangeId::ALL {
            let mut included = AgeAggregate::<bool>::default();
            for entry in included.containing_mut(age) {
                *entry = true;
            }
            for &id in AgeAggregateId::ALL {
                assert_eq!(*id.select(&included), id.contains(age));
                assert_eq!(AgeAggregateId::from_name(id.name()), Some(id));
            }
        }
        assert_eq!(AgeAggregateId::from_name("unknown"), None);
    }
}
