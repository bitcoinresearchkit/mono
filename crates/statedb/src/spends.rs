use crate::{Amount, journal::Journal, journal_reader::JournalReader, util::invalid};
use std::{io::Result, path::Path};

pub struct Spends {
    journal: Journal,
    buffer: Vec<u8>,
}
impl Spends {
    pub fn open(path: &Path) -> Result<Self> {
        Ok(Self {
            journal: Journal::open(&path.join("spends"))?,
            buffer: Vec::new(),
        })
    }
    pub(crate) fn open_reader(path: &Path) -> Result<Self> {
        Ok(Self {
            journal: Journal::open_reader(&path.join("spends"))?,
            buffer: Vec::new(),
        })
    }
    pub fn end(&self) -> usize {
        self.journal.len()
    }
    pub fn version(&self) -> u64 {
        self.journal.version
    }
    pub fn validate_version(&mut self, version: u64) -> Result<()> {
        self.journal.validate_version(version)
    }
    pub fn truncate(&mut self, height: usize) -> Result<()> {
        self.journal.truncate(height)
    }
    pub fn commit(&mut self) -> Result<()> {
        self.journal.commit()
    }
    pub fn push(
        &mut self,
        hash: [u8; 32],
        rows: impl IntoIterator<Item = (u32, Amount)>,
    ) -> Result<()> {
        self.buffer.clear();
        self.buffer.extend_from_slice(&hash);
        self.buffer.extend_from_slice(&Amount::default().encode());
        let mut total = Amount::default();
        for (origin, amount) in rows {
            if origin as usize > self.end() || amount.count == 0 {
                return Err(invalid("invalid spent origin"));
            }
            total = total.checked_add(amount)?;
            let count =
                u32::try_from(amount.count).map_err(|_| invalid("spent count exceeds u32"))?;
            let mut row = [0; 16];
            row[..4].copy_from_slice(&origin.to_le_bytes());
            row[4..8].copy_from_slice(&count.to_le_bytes());
            row[8..].copy_from_slice(&amount.sats.to_le_bytes());
            self.buffer.extend_from_slice(&row);
        }
        self.buffer[32..40].copy_from_slice(&total.sats.to_le_bytes());
        self.buffer[40..48].copy_from_slice(&total.count.to_le_bytes());
        self.journal.push(&self.buffer)
    }
    pub(crate) fn cursor(&self, end: usize) -> JournalReader<'_> {
        JournalReader::new(&self.journal, end)
    }
    pub(crate) fn header(bytes: &[u8]) -> Result<([u8; 32], Amount)> {
        if bytes.len() < 48 || !(bytes.len() - 48).is_multiple_of(16) {
            return Err(invalid("invalid fixed spend rows"));
        }
        Ok((
            bytes[..32].try_into().unwrap(),
            Amount::decode(&bytes[32..48])?,
        ))
    }
    pub(crate) fn rows(bytes: &[u8]) -> impl ExactSizeIterator<Item = (u32, Amount)> + Clone + '_ {
        bytes.as_chunks::<16>().0.iter().map(|row| {
            (
                u32::from_le_bytes(row[..4].try_into().unwrap()),
                Amount {
                    sats: u64::from_le_bytes(row[8..16].try_into().unwrap()),
                    count: u32::from_le_bytes(row[4..8].try_into().unwrap()) as u64,
                },
            )
        })
    }
    pub fn hash(&self, height: usize) -> Result<[u8; 32]> {
        let mut hash = [0; 32];
        self.journal.read_prefix(height, &mut hash)?;
        Ok(hash)
    }
}
