use std::ops::Range;

use bitview_plugin_indexer::has_taproot_envelope;
use bitview_plugin_outputs::published_spend;
use bitview_primitives::TxVersion;
use bitview_types::{TransactionSummary, summary_flags as flags};
use brk_types::{BlockHash, OutputType, Sats, TxIndex, Txid};
use vecdb::{ReadableVec, VecIndex};

use super::ResolvedBlocks;
use crate::{Error, OptionData, Query, ReadPin, Result};

#[cfg(test)]
mod tests;

/// Both forms select once, before conditional HTTP handling; a point selects exactly one row.
/// The block's pin protects canonical membership and every downstream fact through the build.
pub struct ResolvedBlockSummary {
    blocks: ResolvedBlocks,
    block_txs: Range<usize>,
    selected: Range<usize>,
}

impl Query {
    pub fn resolve_block_summary(
        &self,
        hash: &BlockHash,
        txid: Option<&Txid>,
    ) -> Result<ResolvedBlockSummary> {
        // Validates the complete supplied canonical hash, not only its prefix or the tx's block.
        let blocks = self.resolve_block_snapshot(hash)?;
        let pin = blocks.read_pin(self);
        let height = blocks.last_height().data()?;
        let (first, count) = self.block_tx_range(height, &pin)?;
        let end = first.checked_add(count).ok_or(Error::Internal("Block tx range overflow"))?;
        let block_txs = first..end;
        let selected = if let Some(txid) = txid {
            let candidate = match self.resolve_tx_index_bounded(txid, &pin) {
                Ok(index) => Some(index.to_usize()),
                Err(Error::UnknownTxid | Error::StateUpdating) => None,
                Err(error) => return Err(error),
            };
            select_point(block_txs.clone(), candidate, || {
                // A prefix index can name another occurrence (e.g. historical duplicate
                // coinbases). Fall back only to this block's txid column, never its classifier.
                let txids = exact(self.indexer().vecs().transactions.txid
                    .collect_range_at(first, end), count)?;
                Ok(txids.iter().position(|id| id == txid).map(|offset| first + offset))
            })?
        } else {
            block_txs.clone()
        };
        drop(pin);
        Ok(ResolvedBlockSummary { blocks, block_txs, selected })
    }

    fn block_summary_rows(
        &self,
        selected: Range<usize>,
        block_txs: Range<usize>,
        pin: &ReadPin,
    ) -> Result<Vec<TransactionSummary>> {
        let count = selected.len();
        let txs = &self.indexer().vecs().transactions;
        let facts = self.plugins().transactions.summary_facts(selected.clone())?;
        let txids = exact(txs.txid.collect_range_at(selected.start, selected.end), count)?;
        let versions = exact(txs.tx_version.collect_range_at(selected.start, selected.end), count)?;
        let starts = exact(txs.first_txin_index.collect_range_at(selected.start, selected.end), count)?;
        let input_counts = exact(self.plugins().mappings.tx_index.input_count
            .collect_range_at(selected.start, selected.end), count)?;
        let mut masks = vec![0u64; count];
        // Explicit semantic mapping. Native ord-only evidence is a sound positive shortcut;
        // missing broad-envelope evidence is reconstructed below. Native policy/signature/
        // CoinJoin definitions are intentionally retained and documented, not claimed equal.
        macro_rules! map {
            ($($feature:ident => $bit:ident),+ $(,)?) => {$(
                or_column(&mut masks, flags::$bit,
                    txs.features.$feature.flag.collect_range_at(selected.start, selected.end))?;
            )+};
        }
        map! {
            rbf_signaling => RBF,
            p2pk => P2PK, p2ms => P2MS, p2pkh => P2PKH, p2sh => P2SH,
            p2wpkh => P2WPKH, p2wsh => P2WSH, p2tr => P2TR,
            op_return => OP_RETURN, fake_pubkey => FAKE_PUBKEY,
            inscription => INSCRIPTION, fake_scripthash => FAKE_SCRIPTHASH,
            annex => ANNEX, sighash_all => SIGHASH_ALL, sighash_none => SIGHASH_NONE,
            sighash_single => SIGHASH_SINGLE, sighash_default => SIGHASH_DEFAULT,
            sighash_anyone_can_pay => SIGHASH_ACP,
        }

        // Readers/cursors live across the selected range, avoiding repeated page decode per tx.
        let inputs = &self.indexer().vecs().inputs;
        let mut input_type = inputs.output_type.cursor();
        let mut outpoint = inputs.outpoint.cursor();
        let outputs = self.indexer().output_ranges(pin.lengths());
        let mut values = self.indexer().vecs().outputs.value.cursor();
        let spent = self.plugins().outputs.spent.spending_txin_index.reader();
        let mut spending_tx = inputs.tx_index.cursor();
        let mut raw_positions = txs.position.bounded_cursor_at(selected.start, selected.end);
        let mut raw_sizes = txs.total_size.bounded_cursor_at(selected.start, selected.end);
        let mut summaries = Vec::with_capacity(count);
        for (offset, index) in selected.enumerate() {
            self.check_deadline()?;
            let first_input = starts[offset].to_usize();
            let input_count = u64::from(input_counts[offset]) as usize;
            let input_end = checked_end(first_input, input_count, pin.lengths().txin_index.to_usize())?;
            let mut has_parent = false;
            let mut taproot_inputs = Vec::new();
            let needs_envelope = masks[offset] & flags::INSCRIPTION == 0;
            for input in first_input..input_end {
                if !has_parent {
                    let parent = outpoint.get(input).data()?;
                    has_parent = !parent.is_coinbase()
                        && block_txs.contains(&parent.tx_index().to_usize());
                }
                // Coinbase's stored input type is unknown, so cannot enter this branch.
                if needs_envelope && input_type.get(input).data()? == OutputType::P2TR {
                    taproot_inputs.push(input - first_input);
                }
            }
            let mut mask = masks[offset];
            if !taproot_inputs.is_empty() {
                // Decode only this selected tx, only when stored ord evidence was insufficient.
                // No prevout scripts, parent raw txs, signature validation, ASM or block fetch.
                let position = raw_positions.get(index).data()?;
                let size = *raw_sizes.get(index).data()? as usize;
                let (_, tx) = self.indexer().read_tx_at(position, size, txids[offset])?;
                if tx.input.len() != input_count {
                    return Err(Error::Internal("Summary input count mismatch"));
                }
                if taproot_inputs.into_iter().any(|i| has_taproot_envelope(&tx.input[i].witness)) {
                    mask |= flags::INSCRIPTION;
                }
            }
            let range = outputs.get(TxIndex::from(index)).data()?;
            let first_output = range.first.to_usize();
            let output_end = checked_end(first_output, range.len, pin.lengths().txout_index.to_usize())?;
            let mut value = 0u64;
            let mut has_child = false;
            for output in first_output..output_end {
                value = add_output(value, values.get(output).data()?)?;
                if !has_child
                    && let Some(input) = published_spend(spent.try_get_at(output).data()?, pin.lengths()) {
                    has_child |= block_txs.contains(&spending_tx.get(input.to_usize()).data()?.to_usize());
                }
            }
            summaries.push(TransactionSummary {
                txid: txids[offset],
                fee: facts[offset].fee,
                vsize: facts[offset].vsize,
                value: Sats::from(value),
                rate: facts[offset].rate,
                flags: finish_flags(mask, versions[offset], &facts[offset], input_count, range.len, has_parent, has_child),
            });
        }
        Ok(summaries)
    }
}

impl ResolvedBlockSummary {
    pub fn block_hash(&self) -> BlockHash {
        self.blocks.anchor().expect("exact canonical block was resolved")
    }

    pub fn is_settled(&self) -> bool {
        self.blocks.is_settled()
    }

    pub fn build(self, query: &Query) -> Result<Vec<TransactionSummary>> {
        let pin = self.blocks.read_pin(query);
        query.block_summary_rows(self.selected, self.block_txs, &pin)
    }
}

/// A valid global match needs no block-wide reads. Misses/other occurrences search only txids.
fn select_point(
    block: Range<usize>,
    candidate: Option<usize>,
    find_in_block: impl FnOnce() -> Result<Option<usize>>,
) -> Result<Range<usize>> {
    let index = match candidate.filter(|index| block.contains(index)) {
        Some(index) => index,
        None => find_in_block()?.filter(|index| block.contains(index))
            .ok_or_else(|| Error::NotFound("Transaction not found in summary".into()))?,
    };
    let end = index.checked_add(1).ok_or(Error::Internal("Summary index overflow"))?;
    Ok(index..end)
}

fn exact<T>(values: Vec<T>, len: usize) -> Result<Vec<T>> {
    if values.len() != len {
        return Err(Error::Internal("Short block summary column read"));
    }
    Ok(values)
}

fn or_column(masks: &mut [u64], bit: u64, values: Vec<bitview_primitives::Boolean>) -> Result<()> {
    let len = masks.len();
    for (mask, value) in masks.iter_mut().zip(exact(values, len)?) {
        if value.is_true() {
            *mask |= bit;
        }
    }
    Ok(())
}

fn checked_end(first: usize, count: usize, bound: usize) -> Result<usize> {
    first.checked_add(count).filter(|&end| count > 0 && end <= bound)
        .ok_or(Error::Internal("Summary range outside publication"))
}

fn add_output(sum: u64, value: Sats) -> Result<u64> {
    sum.checked_add(u64::from(value)).filter(|&sum| sum <= u64::from(Sats::MAX_MONEY))
        .ok_or(Error::Internal("Invalid summary output sum"))
}

fn finish_flags(
    mut mask: u64, version: TxVersion, facts: &bitview_plugin_transactions::SummaryFacts,
    inputs: usize, outputs: usize, has_parent: bool, has_child: bool,
) -> u64 {
    if mask & flags::RBF == 0 { mask |= flags::NO_RBF; }
    mask |= match version {
        TxVersion::ONE => flags::V1,
        TxVersion::TWO => flags::V2,
        TxVersion::THREE => flags::V3,
        _ => 0,
    };
    if facts.nonstandard { mask |= flags::NONSTANDARD; }
    // A broadened token-related envelope can only exclude an existing native candidate.
    if facts.coinjoin && mask & (flags::INSCRIPTION | flags::OP_RETURN) == 0 { mask |= flags::COINJOIN; }
    if facts.consolidation { mask |= flags::CONSOLIDATION; }
    // Upstream's ratio includes coinbase. Wide arithmetic also covers hostile test counts.
    if inputs > 0 && outputs as u128 >= 5 * inputs as u128 { mask |= flags::BATCH_PAYOUT; }
    // Presence of direct relatives is equivalent to nonempty transitive relative sets.
    // Never use native rate-lifting/lowering flags for these structural bits.
    if has_parent { mask |= flags::CPFP_CHILD; }
    if has_child { mask |= flags::CPFP_PARENT; }
    mask
}
