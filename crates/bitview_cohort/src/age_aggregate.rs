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
    pub sth: T,
    /// Uses long-term-holder UTXOs at least 150 days old.
    pub lth: Old,
    /// Younger than 120 days.
    pub under_4m: T,
    /// Younger than 180 days.
    pub under_6m: T,
    /// At least 120 days old.
    pub over_4m: Old,
    /// At least 180 days old.
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

/// Builds a member of an [`AgeAggregate`] from a context its members share, whatever the
/// member's type: one implementation serves a family typed apart.
pub trait BuildMember<Cx>: Sized {
    type Error;
    fn build(id: AgeAggregateId, cx: &Cx) -> Result<Self, Self::Error>;
}

/// Updates a member of an [`AgeAggregate`] from a context its members share, whatever the
/// member's type.
pub trait UpdateMember<Cx> {
    type Error;
    fn update(&mut self, id: AgeAggregateId, cx: &Cx) -> Result<(), Self::Error>;
}

impl<T, All, Old> AgeAggregate<T, All, Old> {
    /// Each member built from `cx`.
    pub fn try_build<Cx, E>(cx: &Cx) -> Result<Self, E>
    where
        T: BuildMember<Cx, Error = E>,
        All: BuildMember<Cx, Error = E>,
        Old: BuildMember<Cx, Error = E>,
    {
        Ok(Self {
            all: All::build(AgeAggregateId::All, cx)?,
            sth: T::build(AgeAggregateId::Sth, cx)?,
            lth: Old::build(AgeAggregateId::Lth, cx)?,
            under_4m: T::build(AgeAggregateId::Under4M, cx)?,
            under_6m: T::build(AgeAggregateId::Under6M, cx)?,
            over_4m: Old::build(AgeAggregateId::Over4M, cx)?,
            over_6m: Old::build(AgeAggregateId::Over6M, cx)?,
        })
    }

    /// Each member updated from `cx`, all seven in parallel.
    #[cfg(feature = "storage")]
    pub fn par_try_update<Cx: Sync, E: Send>(&mut self, cx: &Cx) -> Result<(), E>
    where
        T: UpdateMember<Cx, Error = E> + Send,
        All: UpdateMember<Cx, Error = E> + Send,
        Old: UpdateMember<Cx, Error = E> + Send,
    {
        use rayon::prelude::*;
        let Self {
            all,
            sth,
            lth,
            under_4m,
            under_6m,
            over_4m,
            over_6m,
        } = self;
        let young = [
            (AgeAggregateId::Sth, sth),
            (AgeAggregateId::Under4M, under_4m),
            (AgeAggregateId::Under6M, under_6m),
        ];
        let old = [
            (AgeAggregateId::Lth, lth),
            (AgeAggregateId::Over4M, over_4m),
            (AgeAggregateId::Over6M, over_6m),
        ];
        let (all, (young, old)) = rayon::join(
            || all.update(AgeAggregateId::All, cx),
            || {
                rayon::join(
                    || {
                        young
                            .into_par_iter()
                            .try_for_each(|(id, m)| m.update(id, cx))
                    },
                    || old.into_par_iter().try_for_each(|(id, m)| m.update(id, cx)),
                )
            },
        );
        all.and(young).and(old)
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
