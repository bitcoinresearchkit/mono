use std::collections::BTreeMap;

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{FeeRate, Sats, Transaction, VSize};

/// Mempool statistics with incrementally maintained fee histogram.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct MempoolInfo {
    /// Number of transactions in the mempool
    #[cfg_attr(feature = "schemars", schemars(example = 5000))]
    pub count: usize,
    /// Total virtual size of all transactions in the mempool (vbytes)
    pub vsize: VSize,
    /// Total fees of all transactions in the mempool (satoshis)
    pub total_fee: Sats,
    /// Fee histogram as Esplora bins it: `[[fee_rate, vsize], ...]` by descending fee rate, each
    /// entry the vsize of the transactions paying from its fee rate up to the previous entry's,
    /// a bin closing at the first fee rate change past 50,000 vB
    #[serde(
        serialize_with = "serialize_fee_histogram",
        deserialize_with = "deserialize_fee_histogram"
    )]
    #[cfg_attr(feature = "schemars", schemars(with = "Vec<[f64; 2]>"))]
    fee_histogram: BTreeMap<FeeRate, VSize>,
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

/// Esplora's histogram bin width (vbytes).
const VSIZE_BIN_WIDTH: u64 = 50_000;

/// Esplora's `make_fee_histogram`: from the highest fee rate down, a bin closes at the first fee
/// rate change once it holds more than [`VSIZE_BIN_WIDTH`], labeled by its lowest fee rate.
fn serialize_fee_histogram<S: Serializer>(
    map: &BTreeMap<FeeRate, VSize>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut bins = Vec::new();
    let (mut rate, mut vsize) = (FeeRate::ZERO, 0);
    for (&next_rate, &next_vsize) in map.iter().rev() {
        if vsize > VSIZE_BIN_WIDTH {
            bins.push((rate, VSize::from(vsize)));
            vsize = 0;
        }
        rate = next_rate;
        vsize += u64::from(next_vsize);
    }
    if vsize > 0 {
        bins.push((rate, VSize::from(vsize)));
    }
    serializer.collect_seq(bins)
}

fn deserialize_fee_histogram<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<FeeRate, VSize>, D::Error> {
    let vec: Vec<(FeeRate, VSize)> = Vec::deserialize(deserializer)?;
    Ok(vec.into_iter().collect())
}
