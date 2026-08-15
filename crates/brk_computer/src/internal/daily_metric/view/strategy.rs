use brk_types::Day1;

pub trait DayStrategy: Send + Sync + 'static {
    fn mapping_end(to: usize, mapping_len: usize) -> usize;
    fn source_index(mapping: &[Day1], index: usize, source_len: usize) -> Option<usize>;
}
