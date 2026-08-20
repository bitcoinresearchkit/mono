use std::ops::AddAssign;

use bitview_cohort::{ByTerm, ProfitabilityRange, ProfitabilityRangeId, TermId, UTXOAggregateId};
use brk_types::{Height, Version};
use vecdb::{
    ColumnId, PcoVec, PcoVecValue, ReadOnlyColumnarVec, ReadableBoxedVec, ReadableCloneableVec,
    ReadableColumnarVec, VecValue,
};

use bitview_compute::CACHE_BUDGET;

const RANGE_COUNT: usize = ProfitabilityRangeId::ALL.len();
const COLUMN_COUNT: usize = TermId::ALL.len() * RANGE_COUNT;

const COLUMNS: [TermProfitabilityRangeId; COLUMN_COUNT] = {
    let first = TermProfitabilityRangeId {
        term: TermId::Short,
        range: ProfitabilityRangeId::ALL[0],
    };
    let mut columns = [first; COLUMN_COUNT];
    let mut term_index = 0;
    while term_index < TermId::ALL.len() {
        let mut range_index = 0;
        while range_index < RANGE_COUNT {
            columns[term_index * RANGE_COUNT + range_index] = TermProfitabilityRangeId {
                term: TermId::ALL[term_index],
                range: ProfitabilityRangeId::ALL[range_index],
            };
            range_index += 1;
        }
        term_index += 1;
    }
    columns
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TermProfitabilityRangeId {
    term: TermId,
    range: ProfitabilityRangeId,
}

impl TermProfitabilityRangeId {
    pub fn source<T>(
        source: &ReadOnlyColumnarVec<PcoVec<Height, T>, Self>,
        name: &str,
        version: Version,
        aggregate: UTXOAggregateId,
        ranges: &[ProfitabilityRangeId],
    ) -> ReadableBoxedVec<Height, T>
    where
        T: PcoVecValue + AddAssign,
    {
        if let (Some(term), [range]) = (aggregate.term(), ranges) {
            return source
                .column(
                    name,
                    version,
                    Self {
                        term,
                        range: *range,
                    },
                )
                .read_only_boxed_clone();
        }

        let selected_term = aggregate.term();
        CACHE_BUDGET
            .wrap(
                source.sum_columns(
                    name,
                    version,
                    TermId::ALL
                        .iter()
                        .copied()
                        .filter(move |&term| selected_term.is_none_or(|selected| selected == term))
                        .flat_map(|term| {
                            ranges
                                .iter()
                                .copied()
                                .map(move |range| Self { term, range })
                        }),
                ),
            )
            .read_only_boxed_clone()
    }
}

impl ColumnId for TermProfitabilityRangeId {
    type Row<T>
        = ByTerm<ProfitabilityRange<T>>
    where
        T: VecValue;

    const VERSION: Version = Version::ONE;
    const ALL: &'static [Self] = &COLUMNS;

    #[inline(always)]
    fn index(self) -> usize {
        self.term.index() * RANGE_COUNT + self.range.index()
    }

    #[inline(always)]
    fn get<T: VecValue>(self, row: &Self::Row<T>) -> &T {
        self.range.get(self.term.get(row))
    }

    #[inline(always)]
    fn get_mut<T: VecValue>(self, row: &mut Self::Row<T>) -> &mut T {
        self.range.get_mut(self.term.get_mut(row))
    }

    fn from_fn<T, F>(mut create: F) -> Self::Row<T>
    where
        T: VecValue,
        F: FnMut(Self) -> T,
    {
        ByTerm {
            short: ProfitabilityRangeId::from_fn(|range| {
                create(Self {
                    term: TermId::Short,
                    range,
                })
            }),
            long: ProfitabilityRangeId::from_fn(|range| {
                create(Self {
                    term: TermId::Long,
                    range,
                })
            }),
        }
    }

    fn map<T, U, F>(row: Self::Row<T>, mut map: F) -> Self::Row<U>
    where
        T: VecValue,
        U: VecValue,
        F: FnMut(T) -> U,
    {
        ByTerm {
            short: ProfitabilityRangeId::map(row.short, &mut map),
            long: ProfitabilityRangeId::map(row.long, map),
        }
    }
}

#[cfg(test)]
mod tests {
    use bitview_cohort::{ProfitabilityRangeId, TermId};
    use vecdb::ColumnId;

    use super::TermProfitabilityRangeId;

    #[test]
    fn stores_short_then_long_ranges() {
        let columns = TermProfitabilityRangeId::ALL;
        let range_count = ProfitabilityRangeId::ALL.len();

        assert_eq!(columns.len(), range_count * TermId::ALL.len());
        for (term_index, &term) in TermId::ALL.iter().enumerate() {
            for (range_index, &range) in ProfitabilityRangeId::ALL.iter().enumerate() {
                assert_eq!(
                    columns[term_index * range_count + range_index],
                    TermProfitabilityRangeId { term, range },
                );
            }
        }
    }
}
