use brk_types::Day1;
use vecdb::VecIndex;

use super::strategy::DayStrategy;

pub struct LastDay;

impl DayStrategy for LastDay {
    fn mapping_end(to: usize, mapping_len: usize) -> usize {
        to.saturating_add(1).min(mapping_len)
    }

    fn source_index(mapping: &[Day1], index: usize, source_len: usize) -> Option<usize> {
        last_source_index(mapping, index, source_len)
    }
}

pub fn last_source_index(mapping: &[Day1], index: usize, source_len: usize) -> Option<usize> {
    let first = mapping[index].to_usize();
    let next_first = mapping
        .get(index + 1)
        .map(|day| day.to_usize())
        .unwrap_or(source_len)
        .min(source_len);
    (first < next_first).then_some(next_first - 1)
}
