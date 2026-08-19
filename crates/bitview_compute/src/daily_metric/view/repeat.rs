use brk_types::Day1;
use vecdb::VecIndex;

use super::strategy::DayStrategy;

pub struct RepeatDay;

impl DayStrategy for RepeatDay {
    fn mapping_end(to: usize, _mapping_len: usize) -> usize {
        to
    }

    fn source_index(mapping: &[Day1], index: usize, source_len: usize) -> Option<usize> {
        repeated_source_index(mapping, index, source_len)
    }
}

pub fn repeated_source_index(mapping: &[Day1], index: usize, source_len: usize) -> Option<usize> {
    let day = mapping[index].to_usize();
    (day < source_len).then_some(day)
}
