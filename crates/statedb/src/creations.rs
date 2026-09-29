use crate::{Amount, journal::Journal, util::invalid};
use std::{io::Result, path::Path};

/// Output facts are the second column of each complete block diff.
pub struct Creations {
    journal: Journal,
}
impl Creations {
    pub fn open(path: &Path) -> Result<Self> {
        Ok(Self {
            journal: Journal::open(&path.join("creations"))?,
        })
    }
    pub(crate) fn open_reader(path: &Path) -> Result<Self> {
        Ok(Self {
            journal: Journal::open_reader(&path.join("creations"))?,
        })
    }
    pub fn len(&self) -> usize {
        self.journal.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == self.start()
    }
    pub fn start(&self) -> usize {
        self.journal.base
    }
    pub fn version(&self) -> u64 {
        self.journal.version
    }
    pub fn validate_version(&mut self, version: u64) -> Result<()> {
        self.journal.validate_version(version)
    }
    pub fn seed(&mut self, height: usize) -> Result<()> {
        self.journal.seed(height)
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
        amount: Amount,
        correction: Option<(u32, Amount)>,
    ) -> Result<()> {
        if amount.count == 0 && amount.sats != 0 {
            return Err(invalid("supply without outputs"));
        }
        if let Some((origin, removed)) = correction
            && (origin as usize >= self.len() || removed.count == 0)
        {
            return Err(invalid("invalid output correction"));
        }
        let mut bytes = Vec::with_capacity(68);
        bytes.extend_from_slice(&hash);
        amount.encode(&mut bytes);
        if let Some((origin, removed)) = correction {
            bytes.extend_from_slice(&origin.to_le_bytes());
            removed.encode(&mut bytes);
        }
        self.journal.push(&bytes)
    }
    pub fn read(&self, height: usize) -> Result<([u8; 32], Amount, Option<(u32, Amount)>)> {
        let mut bytes = Vec::new();
        self.journal.read(height, &mut bytes)?;
        if bytes.len() != 48 && bytes.len() != 68 {
            return Err(invalid("invalid creation record"));
        }
        let correction = if bytes.len() == 68 {
            Some((
                u32::from_le_bytes(bytes[48..52].try_into().unwrap()),
                Amount::decode(&bytes[52..])?,
            ))
        } else {
            None
        };
        let amount = Amount::decode(&bytes[32..48])?;
        if amount.count == 0 && amount.sats != 0 {
            return Err(invalid("supply without outputs"));
        }
        if let Some((origin, removed)) = correction
            && (origin as usize >= height || removed.count == 0)
        {
            return Err(invalid("invalid output correction"));
        }
        Ok((bytes[..32].try_into().unwrap(), amount, correction))
    }
}
