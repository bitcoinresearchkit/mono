use bitview_cohort::{AgeAggregate, AgeAggregateId};
use rayon::prelude::*;

/// Builds a member of an [`AgeAggregate`] from a context its members share, whatever the
/// member's type: one implementation serves a family typed apart.
pub(super) trait BuildMember<Cx>: Sized {
    type Error;
    fn build(id: AgeAggregateId, cx: &Cx) -> Result<Self, Self::Error>;
}

/// Updates a member of an [`AgeAggregate`] from a context its members share, whatever the
/// member's type.
pub(super) trait UpdateMember<Cx> {
    type Error;
    fn update(&mut self, id: AgeAggregateId, cx: &Cx) -> Result<(), Self::Error>;
}

/// Each member built from `cx`.
pub(super) fn try_build<T, All, Old, Cx, E>(cx: &Cx) -> Result<AgeAggregate<T, All, Old>, E>
where
    T: BuildMember<Cx, Error = E>,
    All: BuildMember<Cx, Error = E>,
    Old: BuildMember<Cx, Error = E>,
{
    Ok(AgeAggregate {
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
pub(super) fn par_try_update<T, All, Old, Cx: Sync, E: Send>(
    aggregate: &mut AgeAggregate<T, All, Old>,
    cx: &Cx,
) -> Result<(), E>
where
    T: UpdateMember<Cx, Error = E> + Send,
    All: UpdateMember<Cx, Error = E> + Send,
    Old: UpdateMember<Cx, Error = E> + Send,
{
    let AgeAggregate {
        all,
        sth,
        lth,
        under_4m,
        under_6m,
        over_4m,
        over_6m,
    } = aggregate;
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
