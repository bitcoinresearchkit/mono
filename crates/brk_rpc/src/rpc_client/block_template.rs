use bitcoin::{Amount, Txid as BitcoinTxid, Weight as BitcoinWeight};
use brk_error::{Error, Result};
use brk_types::{Sats, SigOps, Txid, Weight};
use corepc_types::v17::BlockTemplateTransaction;
use rustc_hash::FxHashMap;

use crate::BlockTemplateTx;

use super::ClientInner;

impl ClientInner {
    /// Converts the wire rows without decoding any body: callers decode only
    /// the transactions they don't already hold.
    pub fn build_gbt(transactions: Vec<BlockTemplateTransaction>) -> Result<Vec<BlockTemplateTx>> {
        let invalid = |reason| Error::Parse(format!("invalid block template: {reason}"));
        // Bound the decoded work before trusting a reported weight. Serialized
        // transaction bytes (half the hex length) never exceed weight.
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
        debug_assert_eq!(Self::audit_gbt(&transactions), Ok(()), "block template");

        let mut result: Vec<BlockTemplateTx> = Vec::with_capacity(transactions.len());
        for raw in transactions {
            let txid = raw
                .txid
                .parse::<BitcoinTxid>()
                .map_err(|e| Error::Parse(format!("gbt txid: {e}")))?;
            let fee = u64::try_from(raw.fee).map_err(|_| invalid("negative fee"))?;
            let sigops = u64::try_from(raw.sigops).map_err(|_| invalid("negative sigops"))?;
            result.push(BlockTemplateTx {
                txid: Txid::from(txid),
                fee: Sats::from(fee),
                weight: Weight::from(raw.weight),
                sigops: SigOps::new(sigops),
                data: raw.data,
            });
        }
        Ok(result)
    }

    /// Core's own template invariants, re-derived from the decoded bodies:
    /// identity, weight, shape, monetary bounds and the dependency graph
    /// against transaction inputs. Debug builds only.
    fn audit_gbt(
        transactions: &[BlockTemplateTransaction],
    ) -> std::result::Result<(), &'static str> {
        let rows = transactions
            .iter()
            .map(|raw| {
                raw.clone()
                    .into_model()
                    .map_err(|_| "undecodable transaction")
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let mut positions = FxHashMap::default();
        for (index, row) in rows.iter().enumerate() {
            if positions.insert(row.txid, index).is_some() {
                return Err("duplicate transaction");
            }
        }
        let max_money = Amount::MAX_MONEY.to_sat();
        let mut total_fee = 0u64;
        for (index, row) in rows.iter().enumerate() {
            let tx = &row.data;
            if tx.compute_txid() != row.txid
                || tx.compute_wtxid() != row.wtxid
                || tx.weight() != row.weight
            {
                return Err("transaction identity/weight mismatch");
            }
            if tx.is_coinbase() || tx.input.is_empty() || tx.output.is_empty() {
                return Err("non-coinbase transaction shape");
            }
            tx.output
                .iter()
                .try_fold(0u64, |sum, output| {
                    sum.checked_add(output.value.to_sat())
                        .filter(|sum| *sum <= max_money)
                })
                .ok_or("transaction output value")?;
            total_fee = u64::try_from(row.fee.to_sat())
                .ok()
                .and_then(|fee| total_fee.checked_add(fee))
                .filter(|sum| *sum <= max_money)
                .ok_or("transaction fee")?;

            let mut parents: Vec<usize> = tx
                .input
                .iter()
                .filter_map(|input| positions.get(&input.previous_output.txid).copied())
                .collect();
            if parents.iter().any(|&parent| parent >= index) {
                return Err("parent ordering");
            }
            parents.sort_unstable();
            parents.dedup();
            let mut depends = row
                .depends
                .iter()
                .map(|&index| index.checked_sub(1).map(|index| index as usize))
                .collect::<Option<Vec<_>>>()
                .ok_or("dependency index")?;
            depends.sort_unstable();
            depends.dedup();
            if depends != parents {
                return Err("dependency/body mismatch");
            }
        }
        Ok(())
    }
}
