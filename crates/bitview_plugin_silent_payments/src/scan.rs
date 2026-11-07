use std::ops::Range;

use bitcoin::{Transaction, consensus::deserialize, secp256k1::PublicKey};
use bitview_plugin_indexer::Indexer;
use bitview_primitives::{Bytes32, TxInIndex, TypeIndex};
use brk_error::{Error, Result};
use brk_types::{BlkPosition, OutputType, Tweak, TxIndex};
use rayon::prelude::*;
use vecdb::{AnyVec, ReadableVec, VecIndex};

use crate::tweak::{Prevout, Tweaker, input_key, smallest_outpoint};

/// Taproot's activation height: wallets scan from it, and no transaction before it is eligible.
pub const TAPROOT_ACTIVATION: usize = 709_632;

/// A block's eligible transactions with their tweaks, in block order.
pub type BlockTweaks = Vec<(TxIndex, Tweak)>;

/// Whether BIP352 may count a key for an input spending this output type.
fn may_have_key(output_type: OutputType) -> bool {
    matches!(
        output_type,
        OutputType::P2PKH | OutputType::P2SH | OutputType::P2WPKH | OutputType::P2TR
    )
}

/// Each block's tweaks over `heights`, in block order: the blocks' indexer columns read in large
/// range reads, then the blocks and their transactions scanned in parallel.
pub fn scan(indexer: &Indexer, tweaker: &Tweaker, heights: Range<usize>) -> Result<Vec<BlockTweaks>> {
    let scanned = heights.start.max(TAPROOT_ACTIVATION)..heights.end;
    let mut blocks = vec![Vec::new(); heights.len() - scanned.len()];
    if scanned.is_empty() {
        return Ok(blocks);
    }
    let columns = Columns::read(indexer, scanned);
    let scanner = Scanner {
        indexer,
        tweaker,
        columns: &columns,
    };
    blocks.extend(
        (0..columns.blocks.len())
            .into_par_iter()
            .map(|block| scanner.block(block))
            .collect::<Result<Vec<_>>>()?,
    );
    Ok(blocks)
}

/// The indexer columns a range of blocks is scanned from, each from its range's start (`*_base`).
struct Columns {
    blocks: Vec<(BlkPosition, Bytes32)>,
    /// Each block's first transaction, then the range's end.
    tx_bounds: Vec<usize>,
    tx_base: usize,
    /// Each transaction's first input, then the range's end.
    txin_bounds: Vec<usize>,
    txin_base: usize,
    /// Each transaction's first output, then the range's end.
    txout_bounds: Vec<usize>,
    txout_base: usize,
    tx_positions: Vec<BlkPosition>,
    tx_sizes: Vec<Bytes32>,
    output_types: Vec<OutputType>,
    /// The output type each input spends.
    input_types: Vec<OutputType>,
    input_type_indexes: Vec<TypeIndex>,
}

impl Columns {
    fn read(indexer: &Indexer, heights: Range<usize>) -> Self {
        let vecs = indexer.vecs();
        let txs = &vecs.transactions;
        let tx_bounds = bounds(
            txs.first_tx_index
                .collect_range_at(heights.start, heights.end + 1)
                .into_iter()
                .map(TxIndex::to_usize)
                .collect(),
            heights.len(),
            txs.txid.len(),
        );
        let (tx_base, tx_end) = (tx_bounds[0], tx_bounds[heights.len()]);
        let txin_bounds = bounds(
            txs.first_txin_index
                .collect_range_at(tx_base, tx_end + 1)
                .into_iter()
                .map(TxInIndex::to_usize)
                .collect(),
            tx_end - tx_base,
            vecs.inputs.output_type.len(),
        );
        let txout_bounds = bounds(
            txs.first_txout_index
                .collect_range_at(tx_base, tx_end + 1)
                .into_iter()
                .map(|index| index.get().to_usize())
                .collect(),
            tx_end - tx_base,
            vecs.outputs.output_type.len(),
        );
        let (txin_base, txin_end) = (txin_bounds[0], txin_bounds[tx_end - tx_base]);
        let (txout_base, txout_end) = (txout_bounds[0], txout_bounds[tx_end - tx_base]);

        Self {
            blocks: vecs
                .blocks
                .position
                .collect_range_at(heights.start, heights.end)
                .into_iter()
                .zip(vecs.blocks.size.collect_range_at(heights.start, heights.end))
                .collect(),
            tx_bounds,
            tx_base,
            txin_bounds,
            txin_base,
            txout_bounds,
            txout_base,
            tx_positions: txs.position.collect_range_at(tx_base, tx_end),
            tx_sizes: txs.total_size.collect_range_at(tx_base, tx_end),
            output_types: vecs.outputs.output_type.collect_range_at(txout_base, txout_end),
            input_types: vecs.inputs.output_type.collect_range_at(txin_base, txin_end),
            input_type_indexes: vecs.inputs.type_index.collect_range_at(txin_base, txin_end),
        }
    }

    /// The transaction's inputs, from the range's first input.
    fn inputs(&self, tx: usize) -> Range<usize> {
        let tx = tx - self.tx_base;
        self.txin_bounds[tx] - self.txin_base..self.txin_bounds[tx + 1] - self.txin_base
    }

    /// The transaction's outputs, from the range's first output.
    fn outputs(&self, tx: usize) -> Range<usize> {
        let tx = tx - self.tx_base;
        self.txout_bounds[tx] - self.txout_base..self.txout_bounds[tx + 1] - self.txout_base
    }

    /// Whether the transaction may be eligible: a taproot output, and an input BIP352 may count a
    /// key for.
    fn is_candidate(&self, tx: usize) -> bool {
        self.output_types[self.outputs(tx)].contains(&OutputType::P2TR)
            && self.input_types[self.inputs(tx)]
                .iter()
                .any(|&output_type| may_have_key(output_type))
    }
}

/// `count` ranges' first indexes, read up to one past the last range: the end of the last range
/// is the next one's first index, or `total` when none follows yet.
fn bounds(mut firsts: Vec<usize>, count: usize, total: usize) -> Vec<usize> {
    debug_assert!(firsts.len() == count || firsts.len() == count + 1);
    if firsts.len() == count {
        firsts.push(total);
    }
    firsts
}

struct Scanner<'a> {
    indexer: &'a Indexer,
    tweaker: &'a Tweaker,
    columns: &'a Columns,
}

impl Scanner<'_> {
    /// The block's eligible transactions, its coinbase left out: it spends nothing.
    fn block(&self, block: usize) -> Result<BlockTweaks> {
        let columns = self.columns;
        let txs = columns.tx_bounds[block] + 1..columns.tx_bounds[block + 1];
        let candidates = txs.filter(|&tx| columns.is_candidate(tx)).collect::<Vec<_>>();
        if candidates.is_empty() {
            return Ok(Vec::new());
        }
        let (position, size) = columns.blocks[block];
        let bytes = self
            .indexer
            .reader()
            .read_raw_bytes(position, u32::from(size) as usize)?;
        let tweaks = candidates
            .into_par_iter()
            .map(|tx| {
                let offset = columns.tx_positions[tx - columns.tx_base].offset() - position.offset();
                let start = offset as usize;
                let end = start + u32::from(columns.tx_sizes[tx - columns.tx_base]) as usize;
                let transaction = bytes
                    .get(start..end)
                    .ok_or(Error::Internal("Transaction past its block's bytes"))?;
                self.tweak(tx, &deserialize(transaction)?)
                    .map(|tweak| tweak.map(|tweak| (TxIndex::from(tx), tweak)))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(tweaks.into_iter().flatten().collect())
    }

    /// The transaction's tweak, or `None` when it isn't eligible: it spends a segwit output above
    /// version 1, or BIP352 counts none of its inputs' keys, or they sum to infinity.
    fn tweak(&self, tx: usize, transaction: &Transaction) -> Result<Option<Tweak>> {
        let columns = self.columns;
        let inputs = columns.inputs(tx);
        let types = &columns.input_types[inputs.clone()];
        debug_assert_eq!(types.len(), transaction.input.len(), "inputs of transaction {tx}");
        for (input, &output_type) in inputs.clone().zip(types) {
            if output_type == OutputType::UnknownScript
                && self.spends_future_segwit(columns.txin_base + input)?
            {
                return Ok(None);
            }
        }

        let keys = transaction
            .input
            .iter()
            .zip(inputs)
            .filter_map(|(input, index)| {
                let prevout = self.prevout(
                    columns.input_types[index],
                    columns.input_type_indexes[index],
                )?;
                input_key(&prevout, input)
            })
            .collect::<Vec<PublicKey>>();
        let Some(smallest_outpoint) = smallest_outpoint(&transaction.input) else {
            return Ok(None);
        };
        Ok(self
            .tweaker
            .tweak(&smallest_outpoint, &keys.iter().collect::<Vec<_>>()))
    }

    fn prevout(&self, output_type: OutputType, type_index: TypeIndex) -> Option<Prevout> {
        let addrs = &self.indexer.vecs().addrs;
        let index = usize::from(type_index);
        match output_type {
            OutputType::P2PKH => addrs.p2pkh.bytes.collect_one_at(index).map(|hash| Prevout::P2PKH(*hash)),
            OutputType::P2SH => Some(Prevout::P2SH),
            OutputType::P2WPKH => Some(Prevout::P2WPKH),
            OutputType::P2TR => addrs.p2tr.bytes.collect_one_at(index).map(|key| Prevout::P2TR(*key)),
            _ => None,
        }
    }

    /// Whether the input spends a segwit output above version 1, which BIP352 leaves for future
    /// versions: such a transaction isn't eligible. The indexer classifies those outputs as unknown
    /// scripts without keeping their scripts, so the funding transaction is read (they're rare).
    fn spends_future_segwit(&self, txin: usize) -> Result<bool> {
        let vecs = self.indexer.vecs();
        let outpoint = vecs
            .inputs
            .outpoint
            .collect_one_at(txin)
            .ok_or(Error::Internal("Input without its outpoint"))?;
        let funding = outpoint.tx_index().to_usize();
        let (Some(position), Some(size)) = (
            vecs.transactions.position.collect_one_at(funding),
            vecs.transactions.total_size.collect_one_at(funding),
        ) else {
            return Err(Error::Internal("Funding transaction not indexed"));
        };
        let bytes = self
            .indexer
            .reader()
            .read_raw_bytes(position, u32::from(size) as usize)?;
        let funding = deserialize::<Transaction>(&bytes)?;
        let output = funding
            .output
            .get(usize::from(outpoint.vout()))
            .ok_or(Error::Internal("Outpoint past its transaction's outputs"))?;
        Ok(output
            .script_pubkey
            .witness_version()
            .is_some_and(|version| version.to_num() > 1))
    }
}
