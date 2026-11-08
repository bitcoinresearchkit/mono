//! BlindBit oracle v1's data, from the silent payments plugin's columns, the indexer's and the
//! Outputs plugin's spends: each answer read under the one publication its block was resolved in.

use std::ops::Range;

use bitcoin::{Block, consensus::deserialize, hashes::Hash, hex::DisplayHex};
use bitview_plugin_silent_payments::{DUST_FLOOR, TAPROOT_ACTIVATION, spent_outputs};
use bitview_primitives::{Lengths, TxInIndex};
use bitview_types::{BlindbitFilter, BlindbitFilterKind, BlindbitSpentIndex, BlindbitUtxo};
use brk_types::{BlockHash, Height, OutputType, Sats, SpentOutputHash, Tweak, TxIndex, Vout};
use vecdb::{AnyVec, ReadableVec, VecIndex};

use crate::{Error, OptionData, Query, ReadPin, Result, check_bound};

/// A published block the BlindBit routes answer for, with the pin it was resolved under: every
/// read of the answer stays in that publication.
pub struct BlindbitBlock {
    height: Height,
    hash: BlockHash,
    pin: ReadPin,
}

/// What a block's cut-through answers depend on: how far its Taproot outputs are spent.
pub struct BlindbitSpends {
    key: String,
    is_final: bool,
}

impl BlindbitSpends {
    /// Names the answers' state exactly: the hash of the block holding the latest spend of one of
    /// this block's Taproot outputs, which commits to every earlier spend too.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Whether the answers can never change again: the block and its last spend are below the
    /// reorg window, and no output it lists is left to spend.
    pub fn is_final(&self) -> bool {
        self.is_final
    }
}

impl BlindbitBlock {
    pub fn hash(&self) -> &BlockHash {
        &self.hash
    }

    /// Whether it lies below the reorg window: what's read of it alone never changes.
    pub fn is_settled(&self) -> bool {
        self.pin.lengths().is_settled(self.height)
    }
}

/// A block's outputs, read once: each transaction's, with their types and values.
struct BlockOutputs {
    txs: Range<usize>,
    /// Each transaction's first output, then the block's end, from the block's first output.
    bounds: Vec<usize>,
    txouts: Range<usize>,
    types: Vec<OutputType>,
    values: Vec<Sats>,
}

impl BlockOutputs {
    /// The transaction's Taproot outputs: their position in it and their index in the block's.
    fn taproot(&self, tx: TxIndex) -> impl Iterator<Item = (Vout, usize)> + '_ {
        let tx = tx.to_usize() - self.txs.start;
        (self.bounds[tx]..self.bounds[tx + 1])
            .filter(|&output| self.types[output] == OutputType::P2TR)
            .map(move |output| (Vout::from(output - self.bounds[tx]), output))
    }
}

/// The block's span of a column whose per-height firsts are `firsts`: up to the next height's
/// first, or `total` (the publication's length) for the publication's last block, whatever may
/// follow it unpublished.
fn first_span<I: VecIndex>(
    firsts: &impl ReadableVec<Height, I>,
    height: usize,
    lengths: Lengths,
    total: usize,
) -> Result<Range<usize>> {
    let start = firsts.collect_one_at(height).data()?.to_usize();
    let end = if height + 1 < lengths.height.to_usize() {
        firsts.collect_one_at(height + 1).data()?.to_usize()
    } else {
        total
    };
    Ok(start..end)
}

/// The block's span of a column whose per-height ends are `ends`: read from its height and the
/// one before alone.
fn end_span<I: VecIndex>(
    ends: &impl ReadableVec<Height, I>,
    height: usize,
) -> Result<Range<usize>> {
    let start = match height.checked_sub(1) {
        Some(before) => ends.collect_one_at(before).data()?.to_usize(),
        None => 0,
    };
    Ok(start..ends.collect_one_at(height).data()?.to_usize())
}

impl Query {
    /// The last block served, for `info` and `block-height`: the published tip, or the plugin's
    /// last block while it catches up after a restart.
    pub fn blindbit_tip(&self) -> Result<(Height, BlockHash)> {
        let pin = self.pin_safe_lengths()?;
        let served = self
            .blindbit_computed()
            .min(pin.lengths().height.to_usize());
        let height = Height::from(
            served
                .checked_sub(1)
                .ok_or_else(|| Error::NotFound("No block indexed".into()))?,
        );
        let hash = self
            .indexer()
            .vecs()
            .blocks
            .blockhash
            .collect_one(height)
            .data()?;
        Ok((height, hash))
    }

    /// How many blocks the plugin holds every column of.
    fn blindbit_computed(&self) -> usize {
        let silent_payments = self.plugins().silent_payments;
        [
            silent_payments.end_tweak_index.len(),
            silent_payments.end_new_outputs_filter_index.len(),
            silent_payments.end_spent_outputs_filter_index.len(),
        ]
        .into_iter()
        .min()
        .unwrap_or_default()
    }

    /// A published block by height, pinned until the answer is built.
    pub fn blindbit_block(&self, height: Height) -> Result<BlindbitBlock> {
        let pin = self.pin_safe_lengths()?;
        let published = pin.lengths().height;
        check_bound(&pin, height, published)?;
        if height >= published {
            return Err(Error::NotFound("Height not indexed".into()));
        }
        // A publication can run ahead of the plugin's blocks while it catches up after a restart.
        if height.to_usize() >= self.blindbit_computed() {
            return Err(Error::StateUpdating);
        }
        let hash = self
            .indexer()
            .vecs()
            .blocks
            .blockhash
            .collect_one(height)
            .data()?;
        Ok(BlindbitBlock { height, hash, pin })
    }

    /// How far the block's Taproot outputs are spent as of its publication: a later spend always
    /// has a higher input index, so the highest finds the block that names the state.
    pub fn blindbit_spends(&self, block: &BlindbitBlock) -> Result<BlindbitSpends> {
        if self.blindbit_rows(block)?.is_empty() {
            return Ok(BlindbitSpends {
                key: "none".into(),
                is_final: block.is_settled(),
            });
        }
        let vecs = self.indexer().vecs();
        let lengths = block.pin.lengths();
        let txouts = first_span(
            &vecs.outputs.first_txout_index,
            block.height.to_usize(),
            lengths,
            lengths.txout_index.to_usize(),
        )?;
        let types = vecs
            .outputs
            .output_type
            .collect_range_at(txouts.start, txouts.end);
        let spends = self
            .plugins()
            .outputs
            .spent
            .spending_txin_index
            .collect_range_at(txouts.start, txouts.end);
        if types.len() != txouts.len() || spends.len() != txouts.len() {
            return Err(Error::Internal("Output spends missing"));
        }
        let (mut latest, mut any_unspent) = (None, false);
        for (output_type, spend) in types.into_iter().zip(spends) {
            if output_type != OutputType::P2TR {
                continue;
            }
            let spend = TxInIndex::from(spend);
            if spend.is_unspent() || spend >= lengths.txin_index {
                any_unspent = true;
            } else {
                latest = latest.max(Some(spend));
            }
        }
        let Some(latest) = latest else {
            return Ok(BlindbitSpends {
                key: "none".into(),
                is_final: false,
            });
        };
        let spending_tx = vecs.inputs.tx_index.collect_one(latest).data()?;
        let tx_heights = self.plugins().mappings.tx_heights.load();
        let spent_at: Height = tx_heights.cursor().get(spending_tx).data()?;
        Ok(BlindbitSpends {
            key: vecs.blocks.blockhash.collect_one(spent_at).data()?.to_string(),
            is_final: block.is_settled() && !any_unspent && lengths.is_settled(spent_at),
        })
    }

    /// The tweaks of the block's eligible transactions with a Taproot output holding at least
    /// `dust_limit` (the plugin keeps none below the dust floor).
    pub fn blindbit_tweak_index(
        &self,
        block: &BlindbitBlock,
        dust_limit: Sats,
    ) -> Result<Vec<Tweak>> {
        let rows = self.blindbit_rows(block)?;
        if dust_limit <= DUST_FLOOR || rows.is_empty() {
            return self.blindbit_tweaks_where(rows, |_| true);
        }
        let outputs = self.blindbit_outputs(block)?;
        self.blindbit_tweaks_where(rows, |tx| {
            outputs
                .taproot(tx)
                .any(|(_, output)| outputs.values[output] >= dust_limit)
        })
    }

    /// Cut-through: the tweaks of the block's eligible transactions with an unspent Taproot output
    /// holding at least `dust_limit` and the dust floor, as of the block's publication.
    pub fn blindbit_tweaks(&self, block: &BlindbitBlock, dust_limit: Sats) -> Result<Vec<Tweak>> {
        let rows = self.blindbit_rows(block)?;
        if rows.is_empty() {
            return Ok(Vec::new());
        }
        let outputs = self.blindbit_outputs(block)?;
        let unspent = self.blindbit_unspent(block, &outputs)?;
        let least = dust_limit.max(DUST_FLOOR);
        self.blindbit_tweaks_where(rows, |tx| {
            outputs
                .taproot(tx)
                .any(|(_, output)| unspent[output] && outputs.values[output] >= least)
        })
    }

    /// One of the block's two filters, stored as served.
    pub fn blindbit_filter(
        &self,
        block: &BlindbitBlock,
        kind: BlindbitFilterKind,
    ) -> Result<BlindbitFilter> {
        let silent_payments = self.plugins().silent_payments;
        let (ends, bytes) = match kind {
            BlindbitFilterKind::NewUtxos => (
                &silent_payments.end_new_outputs_filter_index,
                &silent_payments.new_outputs_filter,
            ),
            BlindbitFilterKind::Spent => (
                &silent_payments.end_spent_outputs_filter_index,
                &silent_payments.spent_outputs_filter,
            ),
        };
        let span = end_span(ends, block.height.to_usize())?;
        let data = bytes.collect_range_at(span.start, span.end);
        if data.len() != span.len() {
            return Err(Error::Internal("Filter bytes missing"));
        }
        Ok(BlindbitFilter {
            block_hash: block.hash,
            block_height: block.height,
            data: data.to_lower_hex_string().into(),
            filter_type: 4,
        })
    }

    /// The block's spends of Taproot outputs, hashed as its spent filter holds them, from the
    /// block's bytes in Core's block files.
    pub fn blindbit_spent_index(&self, block: &BlindbitBlock) -> Result<BlindbitSpentIndex> {
        let height = block.height.to_usize();
        let mut data = Vec::new();
        if height >= TAPROOT_ACTIVATION {
            let vecs = self.indexer().vecs();
            let position = vecs.blocks.position.collect_one(block.height).data()?;
            let size = vecs.blocks.size.collect_one(block.height).data()?;
            let bytes = self
                .reader()
                .read_raw_bytes(position, u32::from(size) as usize)?;
            let parsed = deserialize::<Block>(&bytes)
                .map_err(|_| Error::Internal("Invalid raw block payload"))?;
            let hash = parsed.block_hash().to_byte_array();
            debug_assert_eq!(
                BlockHash::from(parsed.block_hash()),
                block.hash,
                "block {height}'s bytes"
            );
            let lengths = block.pin.lengths();
            let inputs = first_span(
                &vecs.inputs.first_txin_index,
                height,
                lengths,
                lengths.txin_index.to_usize(),
            )?;
            let input_types = vecs
                .inputs
                .output_type
                .collect_range_at(inputs.start, inputs.end);
            data.extend(
                spent_outputs(&hash, &parsed.txdata, &input_types).map(SpentOutputHash::from),
            );
        }
        Ok(BlindbitSpentIndex {
            block_hash: block.hash,
            data,
        })
    }

    /// The Taproot outputs of the block's eligible transactions, spent or not as of the block's
    /// publication.
    pub fn blindbit_utxos(&self, block: &BlindbitBlock) -> Result<Vec<BlindbitUtxo>> {
        let tx_indexes = self.blindbit_tx_indexes(self.blindbit_rows(block)?)?;
        if tx_indexes.is_empty() {
            return Ok(Vec::new());
        }
        let outputs = self.blindbit_outputs(block)?;
        let unspent = self.blindbit_unspent(block, &outputs)?;
        let vecs = self.indexer().vecs();
        let timestamp = vecs.blocks.timestamp.collect_one(block.height).data()?;
        let txids = vecs
            .transactions
            .txid
            .collect_range_at(outputs.txs.start, outputs.txs.end);
        let type_indexes = vecs
            .outputs
            .type_index
            .collect_range_at(outputs.txouts.start, outputs.txouts.end);
        if txids.len() != outputs.txs.len() || type_indexes.len() != outputs.txouts.len() {
            return Err(Error::Internal("Block transactions missing"));
        }
        let mut utxos = Vec::new();
        for tx in tx_indexes {
            let txid = txids[tx.to_usize() - outputs.txs.start];
            for (vout, output) in outputs.taproot(tx) {
                let key = vecs
                    .addrs
                    .p2tr
                    .bytes
                    .collect_one_at(usize::from(type_indexes[output]))
                    .data()?;
                utxos.push(BlindbitUtxo {
                    txid,
                    vout,
                    value: outputs.values[output],
                    scriptpubkey: format!("5120{}", key.to_lower_hex_string()).into(),
                    block_height: block.height,
                    block_hash: block.hash,
                    timestamp,
                    spent: !unspent[output],
                });
            }
        }
        Ok(utxos)
    }

    /// The block's eligible transactions' rows.
    fn blindbit_rows(&self, block: &BlindbitBlock) -> Result<Range<usize>> {
        end_span(
            &self.plugins().silent_payments.end_tweak_index,
            block.height.to_usize(),
        )
    }

    /// The rows' transactions.
    fn blindbit_tx_indexes(&self, rows: Range<usize>) -> Result<Vec<TxIndex>> {
        let tx_indexes = self
            .plugins()
            .silent_payments
            .tx_index
            .collect_range_at(rows.start, rows.end);
        if tx_indexes.len() != rows.len() {
            return Err(Error::Internal("Tweak rows missing"));
        }
        Ok(tx_indexes)
    }

    /// The tweaks of the rows whose transaction `keep` keeps, in block order.
    fn blindbit_tweaks_where(
        &self,
        rows: Range<usize>,
        mut keep: impl FnMut(TxIndex) -> bool,
    ) -> Result<Vec<Tweak>> {
        let silent_payments = self.plugins().silent_payments;
        let tx_indexes = self.blindbit_tx_indexes(rows.clone())?;
        let parities = silent_payments
            .tweak_parity
            .collect_range_at(rows.start, rows.end);
        let xs = silent_payments
            .tweak_x
            .collect_range_at(rows.start, rows.end);
        if [parities.len(), xs.len()] != [rows.len(); 2] {
            return Err(Error::Internal("Tweak rows missing"));
        }
        Ok(tx_indexes
            .into_iter()
            .zip(parities.into_iter().zip(xs))
            .filter(|(tx, _)| keep(*tx))
            .map(|(_, (parity, x))| {
                let mut tweak = [parity; 33];
                tweak[1..].copy_from_slice(&*x);
                Tweak::from(tweak)
            })
            .collect())
    }

    /// The block's outputs, by transaction.
    fn blindbit_outputs(&self, block: &BlindbitBlock) -> Result<BlockOutputs> {
        let vecs = self.indexer().vecs();
        let height = block.height.to_usize();
        let lengths = block.pin.lengths();
        let txs = first_span(
            &vecs.transactions.first_tx_index,
            height,
            lengths,
            lengths.tx_index.to_usize(),
        )?;
        let txouts = first_span(
            &vecs.outputs.first_txout_index,
            height,
            lengths,
            lengths.txout_index.to_usize(),
        )?;
        let mut bounds = vecs
            .transactions
            .first_txout_index
            .collect_range_at(txs.start, txs.end)
            .into_iter()
            .map(|first| first.get().to_usize() - txouts.start)
            .collect::<Vec<_>>();
        bounds.push(txouts.len());
        let types = vecs
            .outputs
            .output_type
            .collect_range_at(txouts.start, txouts.end);
        let values = vecs.outputs.value.collect_range_at(txouts.start, txouts.end);
        if bounds.len() != txs.len() + 1
            || types.len() != txouts.len()
            || values.len() != txouts.len()
        {
            return Err(Error::Internal("Block outputs missing"));
        }
        Ok(BlockOutputs {
            txs,
            bounds,
            txouts,
            types,
            values,
        })
    }

    /// Whether each of the block's outputs is unspent as of the block's publication.
    fn blindbit_unspent(
        &self,
        block: &BlindbitBlock,
        outputs: &BlockOutputs,
    ) -> Result<Vec<bool>> {
        let spends = self
            .plugins()
            .outputs
            .spent
            .spending_txin_index
            .collect_range_at(outputs.txouts.start, outputs.txouts.end);
        if spends.len() != outputs.txouts.len() {
            return Err(Error::Internal("Output spends missing"));
        }
        let published = block.pin.lengths().txin_index;
        Ok(spends
            .into_iter()
            .map(|spend| {
                let spend = TxInIndex::from(spend);
                spend.is_unspent() || spend >= published
            })
            .collect())
    }
}
