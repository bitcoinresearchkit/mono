use crate::{
    Amount,
    codec::{decode, encode},
    journal::Journal,
    util::invalid,
};
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
    pub fn start(&self) -> usize {
        self.journal.base
    }
    pub fn len(&self) -> usize {
        self.journal.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == self.start()
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
        rows: impl IntoIterator<Item = (u32, Amount)>,
    ) -> Result<()> {
        self.buffer.clear();
        self.buffer.extend_from_slice(&hash);
        Amount::default().encode(&mut self.buffer);
        let mut total = Amount::default();
        for (origin, amount) in rows {
            if origin as usize > self.len() || amount.count == 0 {
                return Err(invalid("invalid spent origin"));
            }
            total = total.checked_add(amount)?;
            encode(origin.into(), &mut self.buffer);
            encode(amount.sats, &mut self.buffer);
            encode(amount.count, &mut self.buffer);
        }
        self.buffer[32..40].copy_from_slice(&total.sats.to_le_bytes());
        self.buffer[40..48].copy_from_slice(&total.count.to_le_bytes());
        self.journal.push(&self.buffer)
    }
    pub fn read(
        &self,
        height: usize,
        bytes: &mut Vec<u8>,
        rows: &mut Vec<(u32, Amount)>,
    ) -> Result<([u8; 32], Amount)> {
        self.journal.read(height, bytes)?;
        if bytes.len() < 48 {
            return Err(invalid("invalid spend header"));
        }
        let hash = bytes[..32].try_into().unwrap();
        let expected = Amount::decode(&bytes[32..48])?;
        let mut input = &bytes[48..];
        let mut total = Amount::default();
        rows.clear();
        while !input.is_empty() {
            let origin = decode(&mut input)?;
            let amount = Amount {
                sats: decode(&mut input)?,
                count: decode(&mut input)?,
            };
            if origin > height as u64 || amount.count == 0 {
                return Err(invalid("invalid origin row"));
            }
            total = total.checked_add(amount)?;
            rows.push((origin as u32, amount));
        }
        if total != expected {
            return Err(invalid("origin total mismatch"));
        }
        Ok((hash, total))
    }
    pub fn hash(&self, height: usize) -> Result<[u8; 32]> {
        let mut bytes = Vec::new();
        self.journal.read(height, &mut bytes)?;
        bytes
            .get(..32)
            .ok_or_else(|| invalid("missing spend hash"))
            .map(|b| b.try_into().unwrap())
    }
}
