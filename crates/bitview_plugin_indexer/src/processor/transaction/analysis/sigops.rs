use bitcoin::{Script, TxIn, constants::WITNESS_SCALE_FACTOR};
use brk_types::{OutputType, SigOpCount, SigOps32};

use crate::processor::txout::ProcessedOutput;

pub use super::computed_sigops::ComputedSigOps;
use super::input;

#[derive(Default)]
pub struct Accumulator {
    legacy: usize,
    redeem: usize,
    witness: usize,
    executed_legacy: usize,
    track_executed_legacy: bool,
}

impl Accumulator {
    pub fn new(track_executed_legacy: bool) -> Self {
        Self {
            track_executed_legacy,
            ..Self::default()
        }
    }

    pub fn scan_input(
        &mut self,
        prev_kind: OutputType,
        prev_legacy_sigops: SigOps32,
        facts: &input::Facts<'_>,
    ) {
        if !facts.script_sig.push_only {
            self.legacy = self.legacy.saturating_add(facts.script_sig.legacy_sigops);
        }

        if self.track_executed_legacy {
            if !facts.script_sig.push_only {
                self.executed_legacy = self.executed_legacy.saturating_add(
                    facts
                        .script_sig
                        .accurate_sigops
                        .saturating_mul(WITNESS_SCALE_FACTOR),
                );
            }
            if prev_kind != OutputType::P2SH {
                self.executed_legacy = self
                    .executed_legacy
                    .saturating_add(u32::from(prev_legacy_sigops) as usize);
            }
        }

        match prev_kind {
            OutputType::P2SH => {
                let Some(redeem_sigops) = facts.redeem.sigops() else {
                    return;
                };
                self.redeem = self.redeem.saturating_add(redeem_sigops);
                if self.track_executed_legacy {
                    self.executed_legacy = self
                        .executed_legacy
                        .saturating_add(redeem_sigops.saturating_mul(WITNESS_SCALE_FACTOR));
                }
                if !facts.script_sig.push_only {
                    return;
                }
                if facts.redeem.is_p2wpkh() {
                    self.witness = self.witness.saturating_add(1);
                } else if facts.redeem.is_p2wsh()
                    && let Some(last) = facts.witness.last
                {
                    self.witness = self
                        .witness
                        .saturating_add(SigOpCount::of(Script::from_bytes(last)).accurate);
                }
            }
            OutputType::P2WPKH => self.witness = self.witness.saturating_add(1),
            OutputType::P2WSH => {
                if let Some(last) = facts.witness.last {
                    self.witness = self
                        .witness
                        .saturating_add(SigOpCount::of(Script::from_bytes(last)).accurate);
                }
            }
            OutputType::P2TR => {}
            _ => {}
        }
    }

    pub fn scan_coinbase_input(&mut self, input: &TxIn) {
        self.legacy = self
            .legacy
            .saturating_add(SigOpCount::of(&input.script_sig).legacy);
    }

    pub fn scan_output(&mut self, output: &ProcessedOutput) {
        self.legacy = self.legacy.saturating_add(output.script_legacy_sigops());
    }

    pub fn finish(self) -> ComputedSigOps {
        ComputedSigOps {
            total: SigOps32::from(
                self.legacy
                    .saturating_mul(WITNESS_SCALE_FACTOR)
                    .saturating_add(self.redeem.saturating_mul(WITNESS_SCALE_FACTOR))
                    .saturating_add(self.witness),
            ),
            executed_legacy: SigOps32::from(self.executed_legacy),
        }
    }
}
