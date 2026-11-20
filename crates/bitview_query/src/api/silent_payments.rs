//! BlindBit oracle v1's data, from the silent payments plugin's columns, the indexer's and the
//! Outputs plugin's spends: each answer read under the one publication its block was resolved in.

use std::ops::Range;

use bitcoin::{
    Block, Script, VarInt,
    block::Header,
    consensus::{Decodable, deserialize},
    hex::DisplayHex,
};
use bitview_plugin_outputs::published_spend;
use bitview_plugin_silent_payments::{DUST_FLOOR, TAPROOT_ACTIVATION, is_on_curve, spent_outputs};
use bitview_primitives::TxInIndex;
use bitview_types::{BlindbitFilter, BlindbitFilterKind, BlindbitSpentIndex, BlindbitUtxo};
use brk_types::{BlockHash, Height, OutputType, Sats, SpentOutputHash, Timestamp, Tweak, TxIndex, Vout};
use vecdb::{ReadableVec, VecIndex};

use crate::{Error, OptionData, Query, ReadPin, Result, block_span, check_bound, end_span};

/// A published block the BlindBit routes answer for, with the pin it was resolved under: every
/// read of the answer stays in that publication.
pub struct BlindbitBlock {
    height: Height,
    hash: BlockHash,
    pin: ReadPin,
}

/// The Taproot outputs an answer that says what's spent (`tweaks`, `utxos`) is about, read once
/// under the block's pin: those of its eligible transactions holding at least `least`. The
/// answer's validator and its body both come from [`Self::of`], so neither covers an output the
/// other leaves out.
pub struct BlindbitOutputs {
    block: BlindbitBlock,
    rows: Range<usize>,
    /// The rows' transactions.
    tx_indexes: Vec<TxIndex>,
    outputs: BlockOutputs,
    /// The input spending each of the block's outputs as of the block's publication, if any.
    spends: Vec<Option<TxInIndex>>,
    least: Sats,
    key: String,
    is_final: bool,
}

impl BlindbitOutputs {
    pub fn block(&self) -> &BlindbitBlock {
        &self.block
    }

    /// Names the answer's state exactly: the hash of the block holding the latest spend of one of
    /// those outputs, which commits to every earlier spend too.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Whether the answer can never change again: the block and its last spend are below the
    /// reorg window, and none of those outputs is left to spend.
    pub fn is_final(&self) -> bool {
        self.is_final
    }

    /// The transaction's outputs among them: their position in it and their index in the block's.
    fn of(&self, tx: TxIndex) -> impl Iterator<Item = (Vout, usize)> + '_ {
        self.outputs.taproot(tx, self.least)
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
#[derive(Default)]
struct BlockOutputs {
    txs: Range<usize>,
    /// Each transaction's first output, then the block's end, from the block's first output.
    bounds: Vec<usize>,
    txouts: Range<usize>,
    types: Vec<OutputType>,
    values: Vec<Sats>,
}

impl BlockOutputs {
    /// The transaction's Taproot outputs holding at least the minimum read for this block.
    fn taproot(&self, tx: TxIndex, least: Sats) -> impl Iterator<Item = (Vout, usize)> + '_ {
        let tx = tx.to_usize() - self.txs.start;
        (self.bounds[tx]..self.bounds[tx + 1])
            .filter(move |&output| {
                self.types[output] == OutputType::P2TR
                    && (least == Sats::ZERO || self.values[output] >= least)
            })
            .map(move |output| (Vout::from(output - self.bounds[tx]), output))
    }
}

fn compact_size(bytes: &mut &[u8]) -> Option<usize> {
    usize::try_from(VarInt::consensus_decode_from_finite_reader(bytes).ok()?.0).ok()
}

/// Seek to the outputs of an indexed transaction without building inputs or witnesses.
fn tx_output_bytes(mut bytes: &[u8], expected_outputs: usize) -> Option<&[u8]> {
    bytes = bytes.get(4..)?; // version
    if let [0, 1, rest @ ..] = bytes {
        bytes = rest; // SegWit marker and flag
    }
    let inputs = compact_size(&mut bytes)?;
    if inputs == 0 {
        return None;
    }
    for _ in 0..inputs {
        bytes = bytes.get(36..)?; // previous outpoint
        let script = compact_size(&mut bytes)?;
        bytes = bytes.get(script.checked_add(4)?..)?; // scriptSig and sequence
    }
    (compact_size(&mut bytes)? == expected_outputs).then_some(bytes)
}

fn take_raw_output<'a>(bytes: &mut &'a [u8]) -> Option<(Sats, &'a [u8])> {
    let value = Sats::from(u64::from_le_bytes(*bytes.first_chunk()?));
    *bytes = bytes.get(8..)?;
    let len = compact_size(bytes)?;
    let (script, remaining) = bytes.split_at_checked(len)?;
    *bytes = remaining;
    Some((value, script))
}

impl Query {
    /// The published tip, for `info` and `block-height`.
    pub fn blindbit_tip(&self) -> Result<(Height, BlockHash)> {
        let pin = self.pin_safe_lengths()?;
        let height = pin
            .lengths()
            .last_height()
            .ok_or_else(|| Error::NotFound("No block indexed".into()))?;
        Ok((height, pin.tip()))
    }

    /// A published block by height, pinned until the answer is built.
    pub fn blindbit_block(&self, height: Height) -> Result<BlindbitBlock> {
        let pin = self.pin_safe_lengths()?;
        let published = pin.lengths().height;
        check_bound(&pin, height, published)?;
        if height >= published {
            return Err(Error::NotFound("Height not indexed".into()));
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

    /// The Taproot outputs of the block's eligible transactions holding at least `least` (zero:
    /// every one), and how far they are spent as of the block's publication: a later spend always
    /// has a higher input index, so the highest finds the block that names the state.
    pub fn blindbit_outputs(&self, block: BlindbitBlock, least: Sats) -> Result<BlindbitOutputs> {
        let rows = self.blindbit_rows(&block)?;
        let (tx_indexes, outputs, spends) = if rows.is_empty() {
            Default::default()
        } else {
            let outputs = self.blindbit_block_outputs(&block, least)?;
            let spends = self.blindbit_output_spends(&block, &outputs)?;
            (self.blindbit_tx_indexes(rows.clone())?, outputs, spends)
        };
        let mut selected = BlindbitOutputs {
            block,
            rows,
            tx_indexes,
            outputs,
            spends,
            least,
            key: "none".into(),
            is_final: false,
        };
        let (mut latest, mut any_unspent) = (None, false);
        for (_, output) in selected.tx_indexes.iter().flat_map(|&tx| selected.of(tx)) {
            match selected.spends[output] {
                Some(spend) => latest = latest.max(Some(spend)),
                None => any_unspent = true,
            }
        }
        selected.is_final = selected.block.is_settled() && !any_unspent;
        if let Some(latest) = latest {
            let vecs = self.indexer().vecs();
            let spending_tx = vecs.inputs.tx_index.collect_one(latest).data()?;
            let tx_heights = self.plugins().mappings.tx_heights.load();
            let spent_at: Height = tx_heights.cursor().get(spending_tx).data()?;
            selected.key = vecs.blocks.blockhash.collect_one(spent_at).data()?.to_string();
            selected.is_final &= selected.block.pin.lengths().is_settled(spent_at);
        }
        Ok(selected)
    }

    /// The tweaks of the block's eligible transactions with a Taproot output holding at least
    /// `dust_limit`.
    pub fn blindbit_tweak_index(
        &self,
        block: &BlindbitBlock,
        dust_limit: Sats,
    ) -> Result<Vec<Tweak>> {
        let rows = self.blindbit_rows(block)?;
        // Every indexed transaction already has a Taproot output worth at least the floor.
        if dust_limit <= DUST_FLOOR || rows.is_empty() {
            return self.blindbit_tweaks_where(rows, |_| true);
        }
        let tx_indexes = self.blindbit_tx_indexes(rows.clone())?;
        let outputs = self.blindbit_block_outputs(block, dust_limit)?;
        self.blindbit_tweaks_where(rows, |row| {
            outputs
                .taproot(tx_indexes[row], dust_limit)
                .next()
                .is_some()
        })
    }

    /// Cut-through: the tweaks of the transactions with one of `outputs` unspent, as of the
    /// block's publication.
    pub fn blindbit_tweaks(&self, outputs: &BlindbitOutputs) -> Result<Vec<Tweak>> {
        self.blindbit_tweaks_where(outputs.rows.clone(), |row| {
            outputs
                .of(outputs.tx_indexes[row])
                .any(|(_, output)| outputs.spends[output].is_none())
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
        let span = end_span(ends, block.height)?;
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
            let bytes = self.block_raw_at_height(block.height, &block.hash, &block.pin)?;
            let parsed = deserialize::<Block>(&bytes)
                .map_err(|_| Error::Internal("Invalid raw block payload"))?;
            let vecs = self.indexer().vecs();
            let lengths = block.pin.lengths();
            let inputs = block_span(
                &vecs.inputs.first_txin_index,
                block.height,
                lengths,
                lengths.txin_index.to_usize(),
            )?;
            let input_types = vecs
                .inputs
                .output_type
                .collect_range_at(inputs.start, inputs.end);
            data.extend(
                spent_outputs(&block.hash, &parsed.txdata, &input_types).map(SpentOutputHash::from),
            );
        }
        Ok(BlindbitSpentIndex {
            block_hash: block.hash,
            data,
        })
    }

    /// `selected`, each spent or not as of the block's publication, but those whose key is off the
    /// curve.
    pub fn blindbit_utxos(&self, selected: &BlindbitOutputs) -> Result<Vec<BlindbitUtxo>> {
        if selected.tx_indexes.is_empty() {
            return Ok(Vec::new());
        }
        let (block, outputs) = (&selected.block, &selected.outputs);
        let check_keys = self
            .plugins()
            .silent_payments
            .has_off_curve_key
            .collect_one(block.height)
            .data()?
            != 0;
        // The block file holds these scripts together. The address column orders
        // keys by first appearance, so reused keys can fault pages across the chain.
        let (bytes, block_position) =
            self.block_raw_with_position(block.height, &block.hash, &block.pin)?;
        let header = Header::consensus_decode_from_finite_reader(&mut bytes.as_slice())
            .map_err(|_| Error::Internal("Invalid raw block header"))?;
        let timestamp = Timestamp::from(header.time);
        let transactions = &self.indexer().vecs().transactions;
        let txids = transactions.txid.reader();
        let positions = transactions
            .position
            .collect_range_at(outputs.txs.start, outputs.txs.end);
        let mut utxos = Vec::new();
        for &tx in &selected.tx_indexes {
            let mut wanted = selected.of(tx).peekable();
            if wanted.peek().is_none() {
                continue;
            }
            let row = tx.to_usize() - outputs.txs.start;
            let txid = txids.try_get(tx).data()?;
            let position = positions.get(row).data()?;
            let mut txouts = position
                .offset()
                .checked_sub(block_position.offset())
                .filter(|_| position.blk_index() == block_position.blk_index())
                .and_then(|offset| bytes.get(offset as usize..))
                .and_then(|tx| tx_output_bytes(tx, outputs.bounds[row + 1] - outputs.bounds[row]))
                .ok_or(Error::Internal("Indexed transaction missing from block"))?;
            let mut raw_outputs = std::iter::from_fn(|| take_raw_output(&mut txouts));
            let mut next = 0;
            for (vout, output) in wanted {
                let index = usize::from(vout);
                let (value, script) = raw_outputs
                    .nth(index - next)
                    .filter(|(_, script)| Script::from_bytes(script).is_p2tr())
                    .ok_or(Error::Internal("Indexed Taproot output missing from block"))?;
                next = index + 1;
                if check_keys && !is_on_curve(&script[2..]) {
                    continue;
                }
                utxos.push(BlindbitUtxo {
                    txid,
                    vout,
                    value,
                    scriptpubkey: script.to_lower_hex_string().into(),
                    block_height: block.height,
                    block_hash: block.hash,
                    timestamp,
                    spent: selected.spends[output].is_some(),
                });
            }
        }
        Ok(utxos)
    }

    /// The block's eligible transactions' rows.
    fn blindbit_rows(&self, block: &BlindbitBlock) -> Result<Range<usize>> {
        end_span(&self.plugins().silent_payments.end_tweak_index, block.height)
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

    /// The tweaks of the rows `keep` keeps, each named by its position among them, in block order.
    fn blindbit_tweaks_where(
        &self,
        rows: Range<usize>,
        mut keep: impl FnMut(usize) -> bool,
    ) -> Result<Vec<Tweak>> {
        let silent_payments = self.plugins().silent_payments;
        let parities = silent_payments
            .tweak_parity
            .collect_range_at(rows.start, rows.end);
        let xs = silent_payments
            .tweak_x
            .collect_range_at(rows.start, rows.end);
        if [parities.len(), xs.len()] != [rows.len(); 2] {
            return Err(Error::Internal("Tweak rows missing"));
        }
        Ok(parities
            .into_iter()
            .zip(xs)
            .enumerate()
            .filter(|(row, _)| keep(*row))
            .map(|(_, (parity, x))| {
                let mut tweak = [parity; 33];
                tweak[1..].copy_from_slice(&*x);
                Tweak::from(tweak)
            })
            .collect())
    }

    /// The block's outputs, by transaction.
    fn blindbit_block_outputs(&self, block: &BlindbitBlock, least: Sats) -> Result<BlockOutputs> {
        let vecs = self.indexer().vecs();
        let lengths = block.pin.lengths();
        let txs = block_span(
            &vecs.transactions.first_tx_index,
            block.height,
            lengths,
            lengths.tx_index.to_usize(),
        )?;
        let txouts = block_span(
            &vecs.outputs.first_txout_index,
            block.height,
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
        let values = if least == Sats::ZERO {
            Vec::new()
        } else {
            vecs.outputs.value.collect_range_at(txouts.start, txouts.end)
        };
        if bounds.len() != txs.len() + 1
            || types.len() != txouts.len()
            || (least != Sats::ZERO && values.len() != txouts.len())
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

    /// The input spending each of the block's outputs as of the block's publication, if any.
    fn blindbit_output_spends(
        &self,
        block: &BlindbitBlock,
        outputs: &BlockOutputs,
    ) -> Result<Vec<Option<TxInIndex>>> {
        let spends = self
            .plugins()
            .outputs
            .spent
            .spending_txin_index
            .collect_range_at(outputs.txouts.start, outputs.txouts.end);
        if spends.len() != outputs.txouts.len() {
            return Err(Error::Internal("Output spends missing"));
        }
        let safe = block.pin.lengths();
        Ok(spends
            .into_iter()
            .map(|spend| published_spend(spend, safe))
            .collect())
    }
}
