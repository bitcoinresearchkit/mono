mod computed_sigops;
pub mod features;
pub mod input;
pub mod output;
pub mod policy;
pub mod sigops;

use bitcoin::Sequence;
use brk_types::{OutputType, SigOps};
use rayon::prelude::*;
use vecdb::{likely, unlikely};

use super::ComputedTx;
use crate::TxFeatureFlags;
use crate::processor::{BlockProcessor, txin::InputSource, txout::ProcessedOutput};

pub struct TransactionAnalysis {
    pub total_sigop_cost: SigOps,
    pub explicitly_rbf: bool,
    pub features: TxFeatureFlags,
}

#[inline(always)]
pub fn record_explicit_rbf_signal(explicitly_rbf: &mut bool, sequence: Sequence) {
    *explicitly_rbf |= sequence.is_rbf();
}

impl BlockProcessor<'_> {
    #[inline]
    pub fn tracks_executed_legacy_sigops(&self) -> bool {
        policy::tracks_executed_legacy_sigops(self.height)
    }

    pub fn analyze_transactions(
        &self,
        txs: &[ComputedTx<'_>],
        txins: &[InputSource],
        txouts: &[ProcessedOutput],
    ) -> Vec<TransactionAnalysis> {
        let block_first_tx_index = self.lengths.tx_index;
        let track_executed_legacy = self.tracks_executed_legacy_sigops();

        txs.par_iter()
            .map(|tx| {
                let tx_inputs = tx.inputs(txins);
                let tx_outputs = tx.outputs(txouts);
                let is_coinbase = tx.is_coinbase(block_first_tx_index);

                let mut sigops = sigops::Accumulator::new(track_executed_legacy);
                let mut flags = TxFeatureFlags::default();
                let mut explicitly_rbf = false;
                let mut output_scanner = output::Scanner::default();
                let mut policy = policy::Accumulator::new(self.height);

                if unlikely(is_coinbase) {
                    let input = &tx.tx.input[0];
                    record_explicit_rbf_signal(&mut explicitly_rbf, input.sequence);
                    sigops.scan_coinbase_input(input);
                } else {
                    for (input, source) in tx.tx.input.iter().zip(tx_inputs) {
                        record_explicit_rbf_signal(&mut explicitly_rbf, input.sequence);
                        let (output_type, legacy_sigops) = resolved_output_facts(source, txouts);
                        let facts = input::analyze(input, output_type, &mut flags);
                        sigops.scan_input(output_type, legacy_sigops, &facts);
                        policy.scan_input(input, output_type, &facts);
                    }
                }

                for (txout, output) in tx.tx.output.iter().zip(tx_outputs) {
                    output_scanner.scan(txout, output, &mut flags);
                    sigops.scan_output(txout, output);
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
                    explicitly_rbf,
                    features: flags,
                }
            })
            .collect()
    }
}

pub fn resolved_output_facts(
    source: &InputSource,
    txouts: &[ProcessedOutput],
) -> (OutputType, SigOps) {
    match source {
        InputSource::PreviousBlock {
            output_type,
            legacy_sigops,
            ..
        } => (*output_type, *legacy_sigops),
        InputSource::SameBlock { txout_offset, .. } => {
            let output = &txouts[*txout_offset];
            (output.output_type, output.legacy_sigops)
        }
        InputSource::Coinbase => unreachable!(),
    }
}
