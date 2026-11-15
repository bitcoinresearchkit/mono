use std::{ops::Range, slice};

use bitcoin::{Block, Transaction, consensus::deserialize, hashes::Hash, secp256k1::PublicKey};
use bitview_plugin_indexer::Indexer;
use bitview_primitives::{Bytes32, TxInIndex, TypeIndex};
use brk_error::{Error, Result};
use brk_types::{BlkPosition, OutputType, Sats, Tweak, TxIndex};
use rayon::prelude::*;
use vecdb::{AnyVec, ReadableVec, VecIndex};

use crate::{
    filters::{filter, new_outputs, spent_outputs},
    tweak::{Prevout, Tweaker, input_key, is_on_curve, smallest_outpoint},
};

/// Taproot's activation height: wallets scan from it, and no transaction before it is eligible.
pub const TAPROOT_ACTIVATION: usize = 709_632;

/// The least a Taproot output holds for its transaction to be kept: one whose Taproot outputs all
/// hold less (mostly inscription and rune postage) is left out everywhere. 600 is the limit Dana
/// and Bull Bitcoin ask for, and sits above the many outputs of exactly 546. A kept transaction
/// keeps all its Taproot outputs, whatever they hold: a wallet matches its outputs in order.
pub const DUST_FLOOR: Sats = Sats::new(600);

/// An eligible transaction.
pub struct Eligible {
    pub tx_index: TxIndex,
    pub tweak: Tweak,
    /// Whether one of its Taproot outputs' keys is off the curve.
    has_off_curve_key: bool,
}

/// A block's scanning data.
pub struct BlockScan {
    /// Its eligible transactions, in block order.
    pub eligible: Vec<Eligible>,
    /// Whether one of its eligible transactions has a Taproot output key off the curve.
    pub has_off_curve_key: bool,
    /// Its new Taproot output keys' filter.
    pub new_outputs_filter: Vec<u8>,
    /// Its spends of Taproot outputs' filter.
    pub spent_outputs_filter: Vec<u8>,
}

impl BlockScan {
    /// A block before Taproot's activation: nothing to scan, and empty filters (an empty set's
    /// filter is its count alone, whatever the key).
    fn before_activation() -> Self {
        Self {
            eligible: Vec::new(),
            has_off_curve_key: false,
            new_outputs_filter: filter::<&[u8]>(&[0; 32], []),
            spent_outputs_filter: filter::<&[u8]>(&[0; 32], []),
        }
    }
}

/// Whether BIP352 may count a key for an input spending this output type.
fn may_have_key(output_type: OutputType) -> bool {
    matches!(
        output_type,
        OutputType::P2PKH | OutputType::P2SH | OutputType::P2WPKH | OutputType::P2TR
    )
}

/// Whether a transaction may be eligible and is kept, from the types its inputs spend: a Taproot
/// output holding at least the [`DUST_FLOOR`], and an input BIP352 may count a key for.
pub(crate) fn is_candidate(transaction: &Transaction, input_types: &[OutputType]) -> bool {
    transaction
        .output
        .iter()
        .any(|output| output.script_pubkey.is_p2tr() && Sats::from(output.value) >= DUST_FLOOR)
        && input_types.iter().any(|&output_type| may_have_key(output_type))
}

/// Each block's scanning data over `heights`, in block order: the blocks' indexer columns read in
/// large range reads, then the blocks and their transactions scanned in parallel.
pub fn scan(indexer: &Indexer, tweaker: &Tweaker, heights: Range<usize>) -> Result<Vec<BlockScan>> {
    let scanned = heights.start.max(TAPROOT_ACTIVATION)..heights.end;
    let mut blocks = Vec::from_fn(heights.len() - scanned.len(), |_| BlockScan::before_activation());
    if scanned.is_empty() {
        return Ok(blocks);
    }
    let scanner = Scanner::read(indexer, tweaker, scanned);
    blocks.extend(
        (0..scanner.blocks.len())
            .into_par_iter()
            .map(|block| scanner.block(block))
            .collect::<Result<Vec<_>>>()?,
    );
    Ok(blocks)
}

/// Scans a range of blocks, from the indexer columns it needs of them, each from its range's start
/// (`*_base`).
struct Scanner<'a> {
    indexer: &'a Indexer,
    tweaker: &'a Tweaker,
    blocks: Vec<(BlkPosition, Bytes32)>,
    /// Each block's first transaction, then the range's end.
    tx_bounds: Vec<usize>,
    tx_base: usize,
    /// Each transaction's first input, then the range's end.
    txin_bounds: Vec<usize>,
    txin_base: usize,
    /// The output type each input spends.
    input_types: Vec<OutputType>,
    input_type_indexes: Vec<TypeIndex>,
}

impl<'a> Scanner<'a> {
    fn read(indexer: &'a Indexer, tweaker: &'a Tweaker, heights: Range<usize>) -> Self {
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
        let (txin_base, txin_end) = (txin_bounds[0], txin_bounds[tx_end - tx_base]);

        Self {
            indexer,
            tweaker,
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
            input_types: vecs.inputs.output_type.collect_range_at(txin_base, txin_end),
            input_type_indexes: vecs.inputs.type_index.collect_range_at(txin_base, txin_end),
        }
    }

    /// The transaction's inputs, from the range's first input.
    fn inputs(&self, tx: usize) -> Range<usize> {
        let tx = tx - self.tx_base;
        self.txin_bounds[tx] - self.txin_base..self.txin_bounds[tx + 1] - self.txin_base
    }

    /// The block's transactions.
    fn txs(&self, block: usize) -> Range<usize> {
        self.tx_bounds[block]..self.tx_bounds[block + 1]
    }

    /// The block's inputs, from the range's first input.
    fn block_inputs(&self, block: usize) -> Range<usize> {
        let txs = self.txs(block);
        let [start, end] =
            [txs.start, txs.end].map(|tx| self.txin_bounds[tx - self.tx_base] - self.txin_base);
        start..end
    }

    /// The block's eligible transactions (its coinbase left out: it spends nothing) and filters,
    /// from its bytes in Core's block files, parsed once.
    fn block(&self, block: usize) -> Result<BlockScan> {
        let (position, size) = self.blocks[block];
        let bytes = self
            .indexer
            .reader()
            .read_raw_bytes(position, u32::from(size) as usize)?;
        let parsed = deserialize::<Block>(&bytes)?;
        let txs = self.txs(block);
        debug_assert_eq!(parsed.txdata.len(), txs.len(), "transactions of block {block}");
        let candidates = (txs.start + 1..txs.end)
            .filter(|&tx| {
                is_candidate(&parsed.txdata[tx - txs.start], &self.input_types[self.inputs(tx)])
            })
            .collect::<Vec<_>>();
        let eligible = candidates
            .into_par_iter()
            .map(|tx| {
                let transaction = &parsed.txdata[tx - txs.start];
                Ok(self.tweak(tx, transaction)?.map(|tweak| Eligible {
                    tx_index: TxIndex::from(tx),
                    tweak,
                    has_off_curve_key: new_outputs(slice::from_ref(transaction))
                        .any(|key| !is_on_curve(key)),
                }))
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        let block_hash = parsed.block_hash().to_byte_array();
        let input_types = &self.input_types[self.block_inputs(block)];
        Ok(BlockScan {
            has_off_curve_key: eligible.iter().any(|tx| tx.has_off_curve_key),
            eligible,
            new_outputs_filter: filter(&block_hash, new_outputs(&parsed.txdata)),
            spent_outputs_filter: filter(
                &block_hash,
                spent_outputs(&block_hash, &parsed.txdata, input_types),
            ),
        })
    }

    /// The transaction's tweak, or `None` when it isn't eligible: it spends a segwit output above
    /// version 1, or BIP352 counts none of its inputs' keys, or they sum to infinity.
    fn tweak(&self, tx: usize, transaction: &Transaction) -> Result<Option<Tweak>> {
        let inputs = self.inputs(tx);
        let types = &self.input_types[inputs.clone()];
        debug_assert_eq!(types.len(), transaction.input.len(), "inputs of transaction {tx}");
        for (input, &output_type) in inputs.clone().zip(types) {
            if output_type == OutputType::UnknownScript
                && self.spends_future_segwit(self.txin_base + input)?
            {
                return Ok(None);
            }
        }

        let mut keys = Vec::<PublicKey>::with_capacity(transaction.input.len());
        for (input, index) in transaction.input.iter().zip(inputs) {
            let prevout = self.prevout(self.input_types[index], self.input_type_indexes[index])?;
            keys.extend(prevout.and_then(|prevout| input_key(&prevout, input)));
        }
        let Some(smallest_outpoint) = smallest_outpoint(&transaction.input) else {
            return Ok(None);
        };
        Ok(self
            .tweaker
            .tweak(&smallest_outpoint, &keys.iter().collect::<Vec<_>>()))
    }

    /// What the input spends, `None` for an output type that contributes no key. A key the indexer
    /// doesn't hold is an error: leaving the input out would store a wrong tweak.
    fn prevout(&self, output_type: OutputType, type_index: TypeIndex) -> Result<Option<Prevout>> {
        const MISSING: Error = Error::Internal("Spent output's key not indexed");
        let addrs = &self.indexer.vecs().addrs;
        let index = usize::from(type_index);
        Ok(match output_type {
            OutputType::P2PKH => {
                let hash = addrs.p2pkh.bytes.collect_one_at(index).ok_or(MISSING)?;
                Some(Prevout::P2PKH(*hash))
            }
            OutputType::P2SH => Some(Prevout::P2SH),
            OutputType::P2WPKH => Some(Prevout::P2WPKH),
            OutputType::P2TR => {
                let key = addrs.p2tr.bytes.collect_one_at(index).ok_or(MISSING)?;
                Some(Prevout::P2TR(*key))
            }
            _ => None,
        })
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

/// `count` ranges' first indexes, read up to one past the last range: the end of the last range
/// is the next one's first index, or `total` when none follows yet.
fn bounds(mut firsts: Vec<usize>, count: usize, total: usize) -> Vec<usize> {
    debug_assert!(firsts.len() == count || firsts.len() == count + 1);
    if firsts.len() == count {
        firsts.push(total);
    }
    firsts
}
