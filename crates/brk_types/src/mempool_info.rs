use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{FeeRate, Sats, Transaction, VSize};

/// Mempool statistics with incrementally maintained fee histogram.
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MempoolInfo {
    /// Number of transactions in the mempool
    #[schemars(example = 5000)]
    pub count: usize,
    /// Total virtual size of all transactions in the mempool (vbytes)
    pub vsize: VSize,
    /// Total fees of all transactions in the mempool (satoshis)
    pub total_fee: Sats,
    /// Fee histogram: `[[fee_rate, vsize], ...]` sorted by descending fee rate
    #[serde(
        serialize_with = "serialize_fee_histogram",
        deserialize_with = "deserialize_fee_histogram"
    )]
    #[schemars(with = "Vec<[f64; 2]>")]
    pub fee_histogram: BTreeMap<FeeRate, VSize>,
}

impl MempoolInfo {
    #[inline]
    pub fn add(&mut self, tx: &Transaction, fee: Sats) {
        self.count += 1;
        self.vsize += tx.vsize();
        self.total_fee += fee;
        let rate = FeeRate::from((fee, tx.vsize()));
        *self.fee_histogram.entry(rate).or_insert(VSize::from(0u64)) += tx.vsize();
    }

    #[inline]
    pub fn remove(&mut self, tx: &Transaction, fee: Sats) {
        self.count -= 1;
        self.vsize -= tx.vsize();
        self.total_fee -= fee;
        let rate = FeeRate::from((fee, tx.vsize()));
        if let Some(v) = self.fee_histogram.get_mut(&rate) {
            *v -= tx.vsize();
            if u64::from(*v) == 0 {
                self.fee_histogram.remove(&rate);
            }
        }
    }
}

fn serialize_fee_histogram<S: Serializer>(
    map: &BTreeMap<FeeRate, VSize>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let vec: Vec<(FeeRate, VSize)> = map.iter().rev().map(|(&r, &v)| (r, v)).collect();
    vec.serialize(serializer)
}

fn deserialize_fee_histogram<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<FeeRate, VSize>, D::Error> {
    let vec: Vec<(FeeRate, VSize)> = Vec::deserialize(deserializer)?;
    Ok(vec.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fee_histogram_schema_matches_its_pair_array_serialization() {
        let mut info = MempoolInfo::default();
        info.fee_histogram
            .insert(FeeRate::from(2.5), VSize::from(100_u64));
        let value = serde_json::to_value(info).unwrap();
        assert_eq!(value["fee_histogram"], serde_json::json!([[2.5, 100]]));

        let schema = serde_json::to_value(schemars::schema_for!(MempoolInfo)).unwrap();
        let histogram = &schema["properties"]["fee_histogram"];
        assert_eq!(histogram["type"], "array");
        assert_eq!(histogram["items"]["type"], "array");
        assert_eq!(histogram["items"]["minItems"], 2);
        assert_eq!(histogram["items"]["maxItems"], 2);
    }
}
