use std::borrow::Borrow;

use brk_types::{FeeRate, Timestamp, Transaction, VSize};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A retained transaction with mempool enrichment. The base serializer owns all transaction
/// fields, including sigops; this adds only derived fields and the retained first sighting.
/// Graph, acceleration and upstream bookkeeping fields are omitted when unknown.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CachedTransaction<T = Transaction> {
    #[serde(flatten)]
    pub transaction: T,
    pub order: u32,
    pub vsize: VSize,
    #[serde(rename = "feePerVsize")]
    pub fee_per_vsize: f64,
    #[serde(rename = "adjustedVsize")]
    pub adjusted_vsize: f64,
    #[serde(rename = "adjustedFeePerVsize")]
    pub adjusted_fee_per_vsize: f64,
    /// Native Core chunk rate, quantized to milli-sat/vB; retained at burial for tombstones.
    #[serde(rename = "effectiveFeePerVsize")]
    pub effective_fee_per_vsize: FeeRate,
    #[serde(rename = "firstSeen")]
    pub first_seen: Timestamp,
}

impl<T: Borrow<Transaction>> CachedTransaction<T> {
    pub fn new(transaction: T, first_seen: Timestamp, effective_fee_per_vsize: FeeRate) -> Self {
        let tx = transaction.borrow();
        let fractional_vsize = f64::from(tx.weight) / 4.0;
        let adjusted_vsize = f64::from(tx.total_sigop_cost.adjust_weight(tx.weight)) / 4.0;
        debug_assert!(fractional_vsize > 0.0, "retained transactions have weight");
        // Upstream reads the final four display bytes, little-endian. Txid stores the
        // reversed (internal) bytes, so these are its first four bytes, big-endian.
        let order = u32::from_be_bytes(tx.txid[..4].try_into().unwrap());
        let vsize = VSize::from(tx.weight);
        let fee = u64::from(tx.fee) as f64;
        let fee_per_vsize = fee / fractional_vsize;
        let adjusted_fee_per_vsize = fee / adjusted_vsize;
        Self {
            transaction,
            order,
            vsize,
            fee_per_vsize,
            adjusted_vsize,
            adjusted_fee_per_vsize,
            effective_fee_per_vsize,
            first_seen,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use brk_types::{Sats, SigOps, TxStatus, Weight};

    #[test]
    fn retained_projection_matches_constructor_examples_and_flattens_once() {
        let mut tx = Transaction {
            index: None,
            txid: format!("{}01020304", "00".repeat(28)).parse().unwrap(),
            version: bitcoin::transaction::Version::TWO.into(),
            lock_time: bitcoin::absolute::LockTime::ZERO.into(),
            input: Vec::new(), output: Vec::new(), total_size: 100,
            weight: Weight::from(400_u64), total_sigop_cost: SigOps::ZERO,
            fee: Sats::from(1_000_u64), status: TxStatus::UNCONFIRMED,
        };
        // Published constructor examples: fractional weight is the fee denominator;
        // sigop-adjusted vsize is separate from the unadjusted rounded vsize.
        for (weight, sigops, fee, vsize, adjusted, raw_rate, adjusted_rate) in [
            (400, 0, 1_000, 100, 100.0, 10.0, 10.0),
            (401, 0, 401, 101, 100.25, 4.0, 4.0),
            (402, 0, 402, 101, 100.5, 4.0, 4.0),
            (403, 0, 403, 101, 100.75, 4.0, 4.0),
            (400, 100, 1_000, 100, 500.0, 10.0, 2.0),
            (403, 100, 0, 101, 500.0, 0.0, 0.0),
        ] {
            tx.weight = Weight::from(weight as u64);
            tx.total_sigop_cost = SigOps::new(sigops);
            tx.fee = Sats::from(fee as u64);
            let cached = CachedTransaction::new(&tx, Timestamp::new(17), FeeRate::from_milli(1_234));
            assert_eq!(cached.order, 0x0403_0201);
            assert_eq!(u64::from(cached.vsize), vsize);
            assert_eq!(cached.adjusted_vsize, adjusted);
            assert_eq!(cached.fee_per_vsize, raw_rate);
            assert_eq!(cached.adjusted_fee_per_vsize, adjusted_rate);
            let text = serde_json::to_string(&cached).unwrap();
            assert_eq!(text.matches("\"sigops\":").count(), 1);
            let value = serde_json::to_value(cached).unwrap();
            for key in ["vin", "vout", "fee", "weight", "sigops", "order", "vsize",
                "feePerVsize", "adjustedVsize", "adjustedFeePerVsize", "effectiveFeePerVsize", "firstSeen"] {
                assert!(value.get(key).is_some(), "missing {key}");
            }
            for key in ["transaction", "replacement", "acceleration", "ancestors", "position", "flags"] {
                assert!(value.get(key).is_none(), "invented {key}");
            }
            assert_eq!(value["status"], serde_json::json!({"confirmed": false}));
            assert_eq!(value["firstSeen"], 17);
        }
        for (suffix, expected) in [("00000000", 0), ("ffffffff", u32::MAX)] {
            tx.txid = format!("{}{suffix}", "00".repeat(28)).parse().unwrap();
            assert_eq!(CachedTransaction::new(&tx, Timestamp::new(17), FeeRate::ZERO).order, expected);
        }
    }
}
