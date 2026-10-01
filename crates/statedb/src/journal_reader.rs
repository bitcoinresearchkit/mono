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

#[cfg(test)]
mod tests {
    use super::JournalReader;
    use crate::journal::Journal;
    use tempfile::tempdir;
    #[test]
    fn windows_cover_nonzero_base_and_stop_at_the_published_end() {
        let root = tempdir().unwrap();
        let mut journal = Journal::open(root.path()).unwrap();
        journal.seed(700000).unwrap();
        for i in 0..260u32 {
            journal.push(&i.to_le_bytes()).unwrap();
        }
        journal.commit().unwrap();
        journal.push(&[99; 4]).unwrap();
        let reader = Journal::open_reader(root.path()).unwrap();
        let mut cursor = JournalReader::new(&reader, 700260);
        for i in 0..260u32 {
            assert_eq!(cursor.read(700000 + i as usize).unwrap(), i.to_le_bytes());
        }
        assert!(cursor.read(700260).is_err());
    }
    #[test]
    fn windows_bound_normal_buffers_and_admit_a_single_large_record() {
        let root = tempdir().unwrap();
        let mut journal = Journal::open(root.path()).unwrap();
        for size in [600000, 600000, 1500000, 4] {
            journal.push(&vec![size as u8; size]).unwrap();
        }
        journal.commit().unwrap();
        let mut cursor = JournalReader::new(&journal, 4);
        for (i, size) in [600000, 600000, 1500000, 4].into_iter().enumerate() {
            let row = cursor.read(i).unwrap();
            assert_eq!(row.len(), size);
            assert!(row.iter().all(|&v| v == size as u8));
            assert_eq!(cursor.bytes.len(), size);
        }
    }
}
