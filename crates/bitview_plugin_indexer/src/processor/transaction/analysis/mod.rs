mod computed_sigops;
pub mod features;
pub mod input;
pub mod output;
pub mod policy;
pub mod scan;
pub mod signature;
pub mod sigops;

use brk_types::{OutputType, SigOps32};
use rayon::prelude::*;
use vecdb::{likely, unlikely};

use super::ComputedTx;
use crate::{BlockCounters, TxFeatureFlags};
use brk_types::{Height, TxIndex};

use crate::processor::{BlockProcessor, txin::InputSource, txout::ProcessedOutput};

#[derive(Clone, Copy, Default)]
pub struct TransactionAnalysis {
    pub total_sigop_cost: SigOps32,
    pub features: TxFeatureFlags,
}

/// EPOBC tags the first input's sequence number: tag bits 0 to 5 (51 transfer, 37 genesis),
/// padding code bits 6 to 11, the rest zero. Version 1 only, before BIP68 gave those bits a
/// meaning; never a Taproot spend, which no EPOBC wallet made, where small sequence numbers
/// carry the tag by chance.
#[inline(always)]
fn record_epobc_tag(flags: &mut TxFeatureFlags, tx: &bitcoin::Transaction, first: OutputType) {
    let sequence = tx.input[0].sequence.0;
    let tagged = tx.version.0 == 1
        && first != OutputType::P2TR
        && sequence < 1 << 12
        && matches!(sequence & 0x3f, 37 | 51);
    flags.insert(TxFeatureFlags::EPOBC * u32::from(tagged));
}

impl BlockProcessor<'_> {
    /// Analyzes each transaction into `analyses`, resized to them, and counts the block.
    /// The block at `height`, its first transaction at `block_first_tx_index`: an associated
    /// function, so it runs beside the finalization that mutably borrows the processor.
    pub fn analyze_transactions(
        height: Height,
        block_first_tx_index: TxIndex,
        txs: &[ComputedTx<'_>],
        txins: &[InputSource],
        txouts: &[ProcessedOutput],
        analyses: &mut Vec<TransactionAnalysis>,
    ) -> BlockCounters {
        let track_executed_legacy = policy::tracks_executed_legacy_sigops(height);

        // Every entry is overwritten: only growth needs initializing.
        analyses.resize(txs.len(), TransactionAnalysis::default());
        analyses
            .par_iter_mut()
            .zip(txs.par_iter())
            .fold(BlockCounters::default, |mut block, (analysis, tx)| {
                *analysis = analyze_transaction(
                    tx,
                    height,
                    block_first_tx_index,
                    track_executed_legacy,
                    txins,
                    txouts,
                    &mut block,
                );
                block
            })
            .reduce(BlockCounters::default, BlockCounters::merged)
    }
}

fn analyze_transaction(
    tx: &ComputedTx<'_>,
    height: Height,
    block_first_tx_index: TxIndex,
    track_executed_legacy: bool,
    txins: &[InputSource],
    txouts: &[ProcessedOutput],
    block: &mut BlockCounters,
) -> TransactionAnalysis {
    let tx_inputs = tx.inputs(txins);
    let tx_outputs = tx.outputs(txouts);
    let is_coinbase = tx.is_coinbase(block_first_tx_index);

    let mut sigops = sigops::Accumulator::new(track_executed_legacy);
    let mut output_scanner = output::Scanner::default();
    let mut policy = policy::Accumulator::new(height);
    let mut scan = scan::TxScan::new(block);

    if unlikely(is_coinbase) {
        let input = &tx.tx.input[0];
        scan.sequence(input.sequence);
        sigops.scan_coinbase_input(input);
    } else {
        record_epobc_tag(
            &mut scan.flags,
            tx.tx,
            resolved_output_facts(&tx_inputs[0], txouts).0,
        );
        // BIP-68 applies from version 2, read as an unsigned number.
        let counts_relative_locks = tx.tx.version.0 as u32 >= 2;
        for (input, source) in tx.tx.input.iter().zip(tx_inputs) {
            scan.sequence(input.sequence);
            let (output_type, legacy_sigop_cost) = resolved_output_facts(source, txouts);
            let facts = input::analyze(input, output_type, &mut scan);
            sigops.scan_input(output_type, legacy_sigop_cost, &facts);
            policy.scan_input(input, output_type, &facts);
            scan.scan_input(input, output_type, &facts, counts_relative_locks);
        }
    }

    for (txout, output) in tx.tx.output.iter().zip(tx_outputs) {
        let keys = output_scanner.scan(txout, output, &mut scan.flags);
        sigops.scan_output(output);
        scan.scan_output(keys);
        if likely(!is_coinbase) {
            policy.scan_output(txout, output);
        }
    }

    let sigops = sigops.finish();
    if likely(!is_coinbase) {
        policy.finish(tx, sigops, &mut scan.flags);
    }

    TransactionAnalysis {
        total_sigop_cost: sigops.total,
        features: scan.finish(tx.tx, is_coinbase),
    }
}

pub fn resolved_output_facts(
    source: &InputSource,
    txouts: &[ProcessedOutput],
) -> (OutputType, SigOps32) {
    match source {
        InputSource::PreviousBlock {
            output_type,
            legacy_sigop_cost,
            ..
        } => (*output_type, *legacy_sigop_cost),
        InputSource::SameBlock { txout_offset, .. } => {
            let output = &txouts[*txout_offset];
            (output.output_type, output.legacy_sigop_cost)
        }
        InputSource::Coinbase => unreachable!(),
    }
}
