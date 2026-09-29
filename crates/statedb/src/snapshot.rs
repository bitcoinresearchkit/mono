use crate::{
    Amount, State,
    util::{atomic_write, checked, checksum, invalid},
};
use lz4_flex::block::{compress, decompress};
use std::{
    fs,
    io::{ErrorKind, Result},
    path::Path,
};

const MAGIC: &[u8; 8] = b"ORGSNAP2";
const HEADER: usize = 72;

pub(crate) fn write(path: &Path, state: &State, versions: (u64, u64)) -> Result<()> {
    let mut sats = Vec::with_capacity(state.len() * 8);
    let mut counts = Vec::with_capacity(state.len() * 8);
    for v in &state.amounts {
        sats.extend_from_slice(&v.sats.to_le_bytes());
        counts.extend_from_slice(&v.count.to_le_bytes());
    }
    let sats = compress(&sats);
    let counts = compress(&counts);
    let mut out = Vec::with_capacity(HEADER + sats.len() + counts.len() + 4);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&(state.len() as u64).to_le_bytes());
    out.extend_from_slice(&state.hash);
    out.extend_from_slice(&versions.0.to_le_bytes());
    out.extend_from_slice(&versions.1.to_le_bytes());
    out.extend_from_slice(&(sats.len() as u64).to_le_bytes());
    out.extend_from_slice(&sats);
    out.extend_from_slice(&counts);
    checksum(&mut out);
    atomic_write(path, &out)
}

fn prefix(bytes: &[u8], versions: (u64, u64)) -> Result<Option<usize>> {
    if bytes.len() < HEADER || &bytes[..8] != MAGIC {
        return Err(invalid("invalid origin snapshot"));
    }
    let word = |i| u64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
    if (word(48), word(56)) != versions {
        return Ok(None);
    }
    let len = usize::try_from(word(8)).map_err(|_| invalid("snapshot length overflow"))?;
    if len > 100_000_000 {
        return Err(invalid("snapshot exceeds height limit"));
    }
    Ok(Some(len))
}

/// A checksum-validated snapshot. Identity checks and restoration share its compressed bytes.
pub(crate) struct Snapshot {
    bytes: Vec<u8>,
    pub len: usize,
}

impl Snapshot {
    pub fn open(path: &Path, versions: (u64, u64)) -> Result<Option<Self>> {
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        };
        let Some(len) = prefix(checked(&bytes)?, versions)? else {
            return Ok(None);
        };
        Ok(Some(Self { bytes, len }))
    }

    pub fn hash(&self) -> [u8; 32] {
        self.bytes[16..48].try_into().unwrap()
    }

    pub fn state(&self) -> Result<State> {
        let bytes = &self.bytes[..self.bytes.len() - 4];
        let len = self.len;
        let sats_len = u64::from_le_bytes(bytes[64..72].try_into().unwrap());
        let end = usize::try_from(sats_len)
            .ok()
            .and_then(|n| HEADER.checked_add(n))
            .filter(|&n| n <= bytes.len())
            .ok_or_else(|| invalid("invalid snapshot section"))?;
        let sats = decompress(&bytes[HEADER..end], len * 8)
            .map_err(|_| invalid("invalid sats compression"))?;
        let counts = decompress(&bytes[end..], len * 8)
            .map_err(|_| invalid("invalid counts compression"))?;
        if sats.len() != len * 8 || counts.len() != len * 8 {
            return Err(invalid("invalid snapshot length"));
        }
        let amounts = sats
            .chunks_exact(8)
            .zip(counts.chunks_exact(8))
            .map(|(s, c)| Amount {
                sats: u64::from_le_bytes(s.try_into().unwrap()),
                count: u64::from_le_bytes(c.try_into().unwrap()),
            })
            .collect();
        State::new(amounts, self.hash())
    }
}
