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

use crate::{
    Cents, CentsCompact, CostBasisPercentilePrices, Date, PERCENTILES, PERCENTILES_LEN, Sats,
};

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
    /// Return sat- and acquisition-value-weighted percentiles using shared passes.
    pub fn cost_basis_percentile_prices(&self) -> CostBasisPercentilePrices {
        Self::cost_basis_percentile_prices_from_entries(
            self.map.iter().map(|(&price, &sats)| (price, sats)),
        )
    }

    fn cost_basis_percentile_prices_from_entries(
        entries: impl Iterator<Item = (CentsCompact, Sats)> + Clone,
    ) -> CostBasisPercentilePrices {
        let (total_sats, total_value) = entries.clone().fold(
            (0_u128, 0_u128),
            |(total_sats, total_value), (price, sats)| {
                let sats = u128::from(u64::from(sats));
                (total_sats + sats, total_value + price.as_u128() * sats)
            },
        );
        let per_coin_targets = Self::percentile_targets(total_sats);
        let per_dollar_targets = Self::percentile_targets(total_value);
        let mut prices = CostBasisPercentilePrices::default();
        let mut per_coin_index = if total_sats == 0 { PERCENTILES_LEN } else { 0 };
        let mut per_dollar_index = if total_value == 0 { PERCENTILES_LEN } else { 0 };
        let mut cumulative_sats = 0_u128;
        let mut cumulative_value = 0_u128;

        for (price, sats) in entries {
            let sats = u128::from(u64::from(sats));
            cumulative_sats += sats;
            cumulative_value += price.as_u128() * sats;
            let price = price.into();
            Self::fill_percentile_prices(
                &mut prices.per_coin,
                &per_coin_targets,
                &mut per_coin_index,
                cumulative_sats,
                price,
            );
            Self::fill_percentile_prices(
                &mut prices.per_dollar,
                &per_dollar_targets,
                &mut per_dollar_index,
                cumulative_value,
                price,
            );
            if per_coin_index == PERCENTILES_LEN && per_dollar_index == PERCENTILES_LEN {
                break;
            }
        }

        prices
    }

    fn percentile_targets(total: u128) -> [u128; PERCENTILES_LEN] {
        PERCENTILES.map(|percentile| (total * u128::from(percentile) / 100).saturating_sub(1))
    }

    fn fill_percentile_prices(
        prices: &mut [Cents; PERCENTILES_LEN],
        targets: &[u128; PERCENTILES_LEN],
        target_index: &mut usize,
        cumulative: u128,
        price: Cents,
    ) {
        while *target_index < PERCENTILES_LEN && cumulative > targets[*target_index] {
            prices[*target_index] = price;
            *target_index += 1;
        }
    }

    pub fn dir(states_path: &Path, name: &str) -> PathBuf {
        states_path.join(name).join("urpd")
    }

    pub fn path(states_path: &Path, name: &str, date: Date) -> PathBuf {
        Self::dir(states_path, name).join(date.to_string())
    }

    pub fn read(states_path: &Path, name: &str, date: Date) -> Result<Self> {
        let bytes = Self::read_bytes(states_path, name, date)?;
        Ok(Self {
            map: Self::deserialize_entries(&bytes)?.into_iter().collect(),
        })
    }

    /// Read persisted entries and calculate percentiles without building a map.
    pub fn read_cost_basis_percentile_prices(
        states_path: &Path,
        name: &str,
        date: Date,
    ) -> Result<CostBasisPercentilePrices> {
        let bytes = Self::read_bytes(states_path, name, date)?;
        let entries = Self::deserialize_entries(&bytes)?;
        Ok(Self::cost_basis_percentile_prices_from_entries(
            entries.iter().copied(),
        ))
    }

    fn read_bytes(states_path: &Path, name: &str, date: Date) -> Result<Vec<u8>> {
        let path = Self::path(states_path, name, date);
        fs::read(&path).map_err(|error| {
            io::Error::new(
                error.kind(),
                format!("Cannot read URPD '{}': {error}", path.display()),
            )
            .into()
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
    use crate::PercentileId;

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
        assert_eq!(
            UrpdRaw::read_cost_basis_percentile_prices(&root, "test", date).unwrap(),
            actual.cost_basis_percentile_prices()
        );

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

    #[test]
    fn cost_basis_percentiles_match_distribution_nearest_rank() {
        let raw = UrpdRaw {
            map: BTreeMap::from([
                (CentsCompact::new(100), Sats::from(5_u64)),
                (CentsCompact::new(200), Sats::from(5_u64)),
            ]),
        };

        let prices = raw.cost_basis_percentile_prices();
        assert_eq!(
            prices.per_coin[PercentileId::Pct50 as usize],
            Cents::new(100)
        );
        assert_eq!(
            prices.per_coin[PercentileId::Pct55 as usize],
            Cents::new(100)
        );
        assert_eq!(
            prices.per_coin[PercentileId::Pct60 as usize],
            Cents::new(200)
        );
        assert_eq!(
            prices.per_dollar[PercentileId::Pct50 as usize],
            Cents::new(200)
        );
    }

    #[test]
    fn empty_cost_basis_percentiles_match_distribution_default() {
        assert_eq!(
            UrpdRaw::default().cost_basis_percentile_prices(),
            CostBasisPercentilePrices::default()
        );
    }
}
