mod computed_sigops;
pub mod features;
pub mod input;
pub mod output;
pub mod policy;
pub mod sigops;

use bitcoin::Sequence;
use brk_types::{OutputType, SigOps32};
use rayon::prelude::*;
use vecdb::{likely, unlikely};

use super::ComputedTx;
use crate::TxFeatureFlags;
use crate::processor::{BlockProcessor, txin::InputSource, txout::ProcessedOutput};

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

#[inline(always)]
pub fn record_explicit_rbf_signal(flags: &mut TxFeatureFlags, sequence: Sequence) {
    flags.insert(TxFeatureFlags::EXPLICITLY_RBF * u32::from(sequence.is_rbf()));
}

impl BlockProcessor<'_> {
    /// Analyzes each transaction into `analyses`, cleared first.
    pub fn analyze_transactions(
        &self,
        txs: &[ComputedTx<'_>],
        txins: &[InputSource],
        txouts: &[ProcessedOutput],
        analyses: &mut Vec<TransactionAnalysis>,
    ) {
        let block_first_tx_index = self.lengths.tx_index;
        let track_executed_legacy = policy::tracks_executed_legacy_sigops(self.height);

        analyses.clear();
        analyses.par_extend(txs.par_iter().map(|tx| {
            let tx_inputs = tx.inputs(txins);
            let tx_outputs = tx.outputs(txouts);
            let is_coinbase = tx.is_coinbase(block_first_tx_index);

            let mut sigops = sigops::Accumulator::new(track_executed_legacy);
            let mut flags = TxFeatureFlags::default();
            let mut output_scanner = output::Scanner::default();
            let mut policy = policy::Accumulator::new(self.height);

            if unlikely(is_coinbase) {
                let input = &tx.tx.input[0];
                record_explicit_rbf_signal(&mut flags, input.sequence);
                sigops.scan_coinbase_input(input);
            } else {
                record_epobc_tag(
                    &mut flags,
                    tx.tx,
                    resolved_output_facts(&tx_inputs[0], txouts).0,
                );
                for (input, source) in tx.tx.input.iter().zip(tx_inputs) {
                    record_explicit_rbf_signal(&mut flags, input.sequence);
                    let (output_type, legacy_sigop_cost) = resolved_output_facts(source, txouts);
                    let facts = input::analyze(input, output_type, &mut flags);
                    sigops.scan_input(output_type, legacy_sigop_cost, &facts);
                    policy.scan_input(input, output_type, &facts);
                }
            }

            for (txout, output) in tx.tx.output.iter().zip(tx_outputs) {
                output_scanner.scan(txout, output, &mut flags);
                sigops.scan_output(output);
                if likely(!is_coinbase) {
                    policy.scan_output(txout, output);
                }
            }

            let sigops = sigops.finish();
            if likely(!is_coinbase) {
                policy.finish(tx, sigops, &mut flags);
            }

            TransactionAnalysis {
                total_sigop_cost: sigops.total,
                features: flags,
            }
        }));
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
