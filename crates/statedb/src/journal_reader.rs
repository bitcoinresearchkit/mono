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
    pub fn read(&mut self, height: usize) -> Result<&[u8]> {
        if height < self.start || height - self.start >= self.records.len() {
            self.start = usize::MAX;
            self.journal.read_batch(
                height,
                self.end.min(height.saturating_add(128)),
                &mut self.index,
                &mut self.records,
                &mut self.bytes,
            )?;
            self.start = height;
        }
        let i = height - self.start;
        let start = if i == 0 { 0 } else { self.records[i - 1] };
        let end = self.records[i];
        Ok(&self.bytes[start..end])
    }
}
