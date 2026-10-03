use bitcoin::{Amount, Weight as BitcoinWeight};
use brk_error::{Error, Result};
use brk_types::{Sats, Txid, Weight};
use corepc_types::v17::BlockTemplateTransaction;
use rustc_hash::FxHashMap;

use crate::BlockTemplateTx;

use super::ClientInner;

impl ClientInner {
    pub fn build_gbt(transactions: Vec<BlockTemplateTransaction>) -> Result<Vec<BlockTemplateTx>> {
        let invalid = |reason| Error::Parse(format!("invalid block template: {reason}"));
        // Bound the decoded work before trusting a reported weight or reserving
        // an output array. Serialized transaction bytes never exceed weight.
        let max_weight = BitcoinWeight::MAX_BLOCK.to_wu();
        let mut total_weight = 0u64;
        for tx in &transactions {
            if tx.weight == 0 || tx.data.len() as u64 > tx.weight.saturating_mul(2) {
                return Err(invalid("transaction length/weight"));
            }
            total_weight = total_weight
                .checked_add(tx.weight)
                .filter(|weight| *weight <= max_weight)
                .ok_or_else(|| invalid("aggregate weight"))?;
        }

        let mut result: Vec<BlockTemplateTx> = Vec::new();
        let mut positions = FxHashMap::default();
        let mut total_fee = 0u64;
        for raw in transactions {
            let row = raw
                .into_model()
                .map_err(|e| Error::Parse(format!("gbt transaction: {e}")))?;
            let tx = row.data;
            if tx.compute_txid() != row.txid
                || tx.compute_wtxid() != row.wtxid
                || tx.weight() != row.weight
            {
                return Err(invalid("transaction identity/weight mismatch"));
            }
            if tx.is_coinbase() || tx.input.is_empty() || tx.output.is_empty() {
                return Err(invalid("non-coinbase transaction shape"));
            }
            tx.output.iter().try_fold(0u64, |sum, output| {
                sum.checked_add(output.value.to_sat())
                    .filter(|sum| *sum <= Amount::MAX_MONEY.to_sat())
                    .ok_or_else(|| invalid("transaction output value"))
            })?;
            let fee = u64::try_from(row.fee.to_sat()).map_err(|_| invalid("negative fee"))?;
            total_fee = total_fee
                .checked_add(fee)
                .filter(|sum| *sum <= Amount::MAX_MONEY.to_sat())
                .ok_or_else(|| invalid("aggregate fee"))?;
            let txid = Txid::from(row.txid);
            if positions.insert(row.txid, result.len()).is_some() {
                return Err(invalid("duplicate transaction"));
            }
            let depends = row
                .depends
                .into_iter()
                .map(|index| {
                    index
                        .checked_sub(1)
                        .and_then(|index| result.get(index as usize))
                        .map(|parent| parent.txid)
                        .ok_or_else(|| invalid("dependency index"))
                })
                .collect::<Result<Vec<_>>>()?;
            result.push(BlockTemplateTx {
                txid,
                fee: Sats::from(fee),
                weight: Weight::from(row.weight),
                depends,
                tx,
            });
        }

        // Check the dependency graph against transaction inputs, not metadata
        // alone. This also rejects a child preceding its in-template parent.
        for (index, row) in result.iter_mut().enumerate() {
            let mut parents = Vec::new();
            for input in &row.tx.input {
                let txid = input.previous_output.txid;
                if let Some(&position) = positions.get(&txid) {
                    if position >= index {
                        return Err(invalid("parent ordering"));
                    }
                    parents.push(Txid::from(txid));
                }
            }
            parents.sort_unstable_by_key(|txid| **txid);
            parents.dedup();
            row.depends.sort_unstable_by_key(|txid| **txid);
            // Core emits one dependency index per input, so several inputs
            // spending the same parent legitimately repeat its index.
            row.depends.dedup();
            if row.depends != parents {
                return Err(invalid("dependency/body mismatch"));
            }
        }
        Ok(result)
    }
}
