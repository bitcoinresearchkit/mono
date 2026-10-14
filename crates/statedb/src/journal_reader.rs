use crate::journal::Journal;
use std::io::Result;

/// A bounded, contiguous read window for one immutable published prefix.
pub(crate) struct JournalReader<'a> {
    journal: &'a Journal,
    end: usize,
    start: usize,
    index: Vec<u8>,
    records: Vec<usize>,
    bytes: Vec<u8>,
}
impl<'a> JournalReader<'a> {
    pub fn new(journal: &'a Journal, end: usize) -> Self {
        Self {
            journal,
            end,
            start: usize::MAX,
            index: Vec::new(),
            records: Vec::new(),
            bytes: Vec::new(),
        }
    }
    /// The record at `height` and its parent's (empty at genesis), from one read of the file.
    pub fn read(&mut self, height: usize) -> Result<(&[u8], &[u8])> {
        let first = height.saturating_sub(1);
        if first < self.start || height - self.start >= self.records.len() {
            self.start = usize::MAX;
            self.journal.read_batch(
                first,
                self.end.min(height.saturating_add(128)),
                &mut self.index,
                &mut self.records,
                &mut self.bytes,
            )?;
            self.start = first;
        }
        let record = |i: usize| {
            let start = if i == 0 { 0 } else { self.records[i - 1] };
            &self.bytes[start..self.records[i]]
        };
        let i = height - self.start;
        Ok((if height == 0 { &[] } else { record(i - 1) }, record(i)))
    }
}
