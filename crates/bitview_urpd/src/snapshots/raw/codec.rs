use brk_error::{Error, Result};
use brk_types::{CentsCompact, Sats};
use pco::{ChunkConfig, standalone::simple_compress};

use crate::distribution::{UrpdRaw, checked_supply};

use super::decode;

struct DecodedEntries<'a> {
    entries: Vec<(CentsCompact, Sats)>,
    rest: &'a [u8],
}

impl UrpdRaw {
    /// Decode one validated distribution and leave subsequent sections untouched.
    pub fn deserialize_with_rest(data: &[u8]) -> Result<(Self, &[u8])> {
        Self::decode_entries(data).map(|decoded| {
            (
                Self {
                    map: decoded.entries.into_iter().collect(),
                },
                decoded.rest,
            )
        })
    }

    fn decode_entries(data: &[u8]) -> Result<DecodedEntries<'_>> {
        if data.len() < 24 {
            return Err(Error::Deserialization(format!(
                "UrpdRaw: data too short ({} bytes, need >= 24)",
                data.len()
            )));
        }
        let read_length = |bytes: &[u8]| {
            usize::try_from(u64::from_le_bytes(bytes.try_into().unwrap())).map_err(|_| {
                Error::Deserialization("UrpdRaw: length exceeds platform capacity".into())
            })
        };
        let entry_count = read_length(&data[0..8])?;
        let keys_len = read_length(&data[8..16])?;
        let values_len = read_length(&data[16..24])?;

        let keys_start = 24_usize;
        let values_start = keys_start.checked_add(keys_len).ok_or_else(|| {
            Error::Deserialization("UrpdRaw: key section length overflows".into())
        })?;
        let rest_start = values_start.checked_add(values_len).ok_or_else(|| {
            Error::Deserialization("UrpdRaw: value section length overflows".into())
        })?;
        if rest_start > Self::MAX_ENCODED_BYTES {
            return Err(Error::Deserialization(
                "UrpdRaw: encoded section exceeds snapshot limit".into(),
            ));
        }

        if data.len() < rest_start {
            return Err(Error::Deserialization(format!(
                "UrpdRaw: data too short ({} bytes, need >= {})",
                data.len(),
                rest_start
            )));
        }

        // Reject oversized counts before compressed streams request output memory.
        if entry_count > Self::MAX_ENTRIES {
            return Err(Error::Deserialization(
                "UrpdRaw: entry count exceeds snapshot limit".into(),
            ));
        }
        let keys: Vec<u32> = decode::exact(&data[keys_start..values_start], entry_count)?;
        if keys.last() == Some(&u32::MAX) || !keys.windows(2).all(|pair| pair[0] < pair[1]) {
            return Err(Error::Deserialization(
                "UrpdRaw: prices must be finite and strictly sorted".into(),
            ));
        }
        let values: Vec<u64> = decode::exact(&data[values_start..rest_start], entry_count)?;
        checked_supply(values.iter().copied())?;

        let entries = keys
            .into_iter()
            .zip(values)
            .map(|(k, v)| (CentsCompact::new(k), Sats::from(v)))
            .collect::<Vec<_>>();

        Ok(DecodedEntries {
            entries,
            rest: &data[rest_start..],
        })
    }

    /// Decode finite, strictly sorted prices with bounded entry count and supply.
    /// A standalone section must have no trailing bytes.
    pub fn deserialize_entries(data: &[u8]) -> Result<Vec<(CentsCompact, Sats)>> {
        let decoded = Self::decode_entries(data)?;
        if !decoded.rest.is_empty() {
            return Err(Error::Deserialization(format!(
                "UrpdRaw: {} trailing bytes",
                decoded.rest.len()
            )));
        }
        Ok(decoded.entries)
    }

    pub fn serialize(&self) -> Result<Vec<u8>> {
        Self::serialize_iter(self.map.iter().map(|(&k, &v)| (k, v)))
    }

    /// Encode entries supplied in strictly increasing price order.
    pub fn serialize_iter(iter: impl Iterator<Item = (CentsCompact, Sats)>) -> Result<Vec<u8>> {
        let mut keys = Vec::new();
        let mut values = Vec::new();
        for (key, value) in iter {
            if keys.len() == Self::MAX_ENTRIES {
                return Err(Error::Internal(
                    "UrpdRaw: entry count exceeds snapshot limit",
                ));
            }
            keys.push(
                key.finite_inner()
                    .ok_or(Error::Internal("UrpdRaw: non-finite price"))?,
            );
            values.push(u64::from(value));
        }
        checked_supply(values.iter().copied())?;

        let config = ChunkConfig::default();
        let compressed_keys = simple_compress(&keys, &config)?;
        let compressed_values = simple_compress(&values, &config)?;

        let mut buffer = Vec::new();
        buffer.extend((keys.len() as u64).to_le_bytes());
        buffer.extend((compressed_keys.len() as u64).to_le_bytes());
        buffer.extend((compressed_values.len() as u64).to_le_bytes());
        buffer.extend(compressed_keys);
        buffer.extend(compressed_values);

        Ok(buffer)
    }
}

#[cfg(test)]
#[path = "../../../tests/snapshots/raw.rs"]
mod tests;
