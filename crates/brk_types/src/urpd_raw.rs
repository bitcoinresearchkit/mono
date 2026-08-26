use std::{
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
};

use brk_error::{Error, Result};
use pco::{
    ChunkConfig,
    standalone::{simple_compress, simple_decompress},
};
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::Bytes;

use crate::{CentsCompact, Date, Sats};

/// Raw on-disk URPD: a map of price (cents) to supply (sats).
/// Processed into [`crate::Urpd`] for API responses.
#[derive(Debug, Clone, Default, Serialize, JsonSchema)]
pub struct UrpdRaw {
    pub map: BTreeMap<CentsCompact, Sats>,
}

struct DecodedEntries<'a> {
    entries: Vec<(CentsCompact, Sats)>,
    rest: &'a [u8],
}

impl UrpdRaw {
    pub fn dir(states_path: &Path, name: &str) -> PathBuf {
        states_path.join(name).join("urpd")
    }

    pub fn path(states_path: &Path, name: &str, date: Date) -> PathBuf {
        Self::dir(states_path, name).join(date.to_string())
    }

    pub fn read(states_path: &Path, name: &str, date: Date) -> Result<Self> {
        let path = Self::path(states_path, name, date);
        let bytes = fs::read(&path).map_err(|error| {
            io::Error::new(
                error.kind(),
                format!("Cannot read URPD '{}': {error}", path.display()),
            )
        })?;
        Ok(Self {
            map: Self::deserialize_entries(&bytes)?.into_iter().collect(),
        })
    }

    pub fn write(
        states_path: &Path,
        name: &str,
        date: Date,
        entries: impl Iterator<Item = (CentsCompact, Sats)>,
    ) -> Result<()> {
        let dir = Self::dir(states_path, name);
        fs::create_dir_all(&dir)?;
        fs::write(dir.join(date.to_string()), Self::serialize_iter(entries)?)?;
        Ok(())
    }

    /// Apply one scalar weight to every price bucket, flooring to whole sats.
    pub fn apply_weight(mut self, weight: f64) -> Self {
        debug_assert!(weight.is_finite() && weight >= 0.0);

        if weight == 1.0 {
            return self;
        }
        if weight == 0.0 {
            self.map.clear();
            return self;
        }

        self.map.retain(|_, sats| {
            *sats = Sats::from((u64::from(*sats) as f64 * weight).floor() as u64);
            *sats != Sats::ZERO
        });
        self
    }

    /// Deserialize from the pco-compressed format, returning remaining bytes.
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
        let entry_count = usize::from_bytes(&data[0..8])?;
        let keys_len = usize::from_bytes(&data[8..16])?;
        let values_len = usize::from_bytes(&data[16..24])?;

        let keys_start = 24;
        let values_start = keys_start + keys_len;
        let rest_start = values_start + values_len;

        if data.len() < rest_start {
            return Err(Error::Deserialization(format!(
                "UrpdRaw: data too short ({} bytes, need >= {})",
                data.len(),
                rest_start
            )));
        }

        let keys: Vec<u32> = simple_decompress(&data[keys_start..values_start])?;
        let values: Vec<u64> = simple_decompress(&data[values_start..rest_start])?;

        let entries = keys
            .into_iter()
            .zip(values)
            .map(|(k, v)| (CentsCompact::new(k), Sats::from(v)))
            .collect::<Vec<_>>();

        debug_assert_eq!(entries.len(), entry_count);
        debug_assert!(entries.windows(2).all(|pair| pair[0].0 < pair[1].0));

        Ok(DecodedEntries {
            entries,
            rest: &data[rest_start..],
        })
    }

    /// Deserialize exactly one sorted sequence of on-disk entries.
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

    /// Deserialize from the pco-compressed format.
    pub fn deserialize(data: &[u8]) -> Result<Self> {
        Self::deserialize_with_rest(data).map(|(s, _)| s)
    }

    /// Serialize to the pco-compressed format.
    pub fn serialize(&self) -> Result<Vec<u8>> {
        Self::serialize_iter(self.map.iter().map(|(&k, &v)| (k, v)))
    }

    /// Serialize from a sorted iterator of (price, sats) pairs.
    pub fn serialize_iter(iter: impl Iterator<Item = (CentsCompact, Sats)>) -> Result<Vec<u8>> {
        let (keys, values): (Vec<u32>, Vec<u64>) = iter
            .map(|(key, value)| (key.inner(), u64::from(value)))
            .unzip();

        let config = ChunkConfig::default();
        let compressed_keys = simple_compress(&keys, &config)?;
        let compressed_values = simple_compress(&values, &config)?;

        let mut buffer = Vec::new();
        buffer.extend(keys.len().to_bytes());
        buffer.extend(compressed_keys.len().to_bytes());
        buffer.extend(compressed_values.len().to_bytes());
        buffer.extend(compressed_keys);
        buffer.extend(compressed_values);

        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_roundtrip() {
        let root = std::env::temp_dir().join(format!("brk-urpd-file-{}", std::process::id()));
        let date = Date::new(2026, 8, 4);
        let expected = BTreeMap::from([
            (CentsCompact::new(100), Sats::from(21_u64)),
            (CentsCompact::new(200), Sats::from(34_u64)),
        ]);

        UrpdRaw::write(
            &root,
            "test",
            date,
            expected.iter().map(|(&price, &sats)| (price, sats)),
        )
        .unwrap();
        let actual = UrpdRaw::read(&root, "test", date).unwrap();

        assert_eq!(actual.map, expected);

        UrpdRaw::write(&root, "empty", date, std::iter::empty()).unwrap();
        assert!(UrpdRaw::read(&root, "empty", date).unwrap().map.is_empty());

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn scalar_weight_floors_each_bucket() {
        let raw = UrpdRaw {
            map: BTreeMap::from([
                (CentsCompact::new(100), Sats::from(3_u64)),
                (CentsCompact::new(200), Sats::from(1_u64)),
            ]),
        };

        assert_eq!(
            raw.apply_weight(0.5).map,
            BTreeMap::from([(CentsCompact::new(100), Sats::from(1_u64))])
        );
    }
}
