use bitcoin::{Script, TxIn, TxOut, constants::WITNESS_SCALE_FACTOR};
use brk_types::{OutputType, SigOps};

use crate::processor::txout::ProcessedOutput;

use super::input;

#[derive(Default)]
pub(super) struct Accumulator {
    legacy: usize,
    redeem: usize,
    witness: usize,
    executed_legacy: usize,
    track_executed_legacy: bool,
}

impl Accumulator {
    pub(super) fn new(track_executed_legacy: bool) -> Self {
        Self {
            track_executed_legacy,
            ..Self::default()
        }
    }

    pub(super) fn scan_input(
        &mut self,
        prev_kind: OutputType,
        prev_legacy_sigops: SigOps,
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
                let Some(redeem_sigops) = facts.redeem_sigops() else {
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
                if facts.redeem_is_p2wpkh() {
                    self.witness = self.witness.saturating_add(1);
                } else if facts.redeem_is_p2wsh()
                    && let Some(last) = facts.witness.last
                {
                    self.witness = self
                        .witness
                        .saturating_add(Script::from_bytes(last).count_sigops());
                }
            }
            OutputType::P2WPKH => self.witness = self.witness.saturating_add(1),
            OutputType::P2WSH => {
                if let Some(last) = facts.witness.last {
                    self.witness = self
                        .witness
                        .saturating_add(Script::from_bytes(last).count_sigops());
                }
            }
            OutputType::P2TR => {}
            _ => {}
        }
    }

    pub(super) fn scan_coinbase_input(&mut self, input: &TxIn) {
        self.legacy = self
            .legacy
            .saturating_add(input.script_sig.count_sigops_legacy());
    }

    pub(super) fn scan_output(&mut self, txout: &TxOut, output: &ProcessedOutput) {
        self.legacy = self
            .legacy
            .saturating_add(legacy_sigops_for_output(output, &txout.script_pubkey));
    }

    pub(super) fn finish(self) -> ComputedSigOps {
        ComputedSigOps {
            total: SigOps::from(
                self.legacy
                    .saturating_mul(WITNESS_SCALE_FACTOR)
                    .saturating_add(self.redeem.saturating_mul(WITNESS_SCALE_FACTOR))
                    .saturating_add(self.witness),
            ),
            executed_legacy: SigOps::from(self.executed_legacy),
        }
    }
}

#[derive(Clone, Copy, Default)]
pub(super) struct ComputedSigOps {
    pub(super) total: SigOps,
    pub(super) executed_legacy: SigOps,
}

/// Legacy sigop count of a script_pubkey, dispatched on `OutputType`.
/// Every variant except `OpReturn` and `Unknown` has a canonical shape
/// recognised by `OutputType::from`'s exact byte-pattern matchers, so
/// the legacy sigop count is fixed: P2PKH and P2PK both end in a
/// single OP_CHECKSIG (1), P2MS contains one OP_CHECKMULTISIG counted
/// as 20 in legacy mode, and P2SH/P2WPKH/P2WSH/P2TR/P2A/Empty contain
/// no CHECKSIG-class opcodes outside their pushdata. `OpReturn`
/// payloads can include 0xac/0xae bytes outside a push, and `Unknown`
/// can be anything, so both fall back to a real script walk.
#[inline]
fn legacy_sigops_for_output(output: &ProcessedOutput, script_pubkey: &Script) -> usize {
    match output.output_type {
        OutputType::P2PKH | OutputType::P2PK33 | OutputType::P2PK65 => 1,
        OutputType::P2MS => 20,
        OutputType::P2SH
        | OutputType::P2WPKH
        | OutputType::P2WSH
        | OutputType::P2TR
        | OutputType::P2A
        | OutputType::Empty => 0,
        OutputType::OpReturn => output.op_return_legacy_sigops(),
        OutputType::Unknown => script_pubkey.count_sigops_legacy(),
    }
}

#[cfg(test)]
mod tests {
    use bitcoin::{ScriptBuf, TxIn, Witness};
    use brk_types::{OutputType, SigOps};

    use super::{Accumulator, input};
    use crate::TxFeatureFlags;

    #[test]
    fn push_only_script_sig_keeps_prevout_executed_sigops() {
        let input = TxIn {
            script_sig: ScriptBuf::from_hex("00").unwrap(),
            ..TxIn::default()
        };
        let facts = input::analyze(&input, OutputType::P2PKH, &mut TxFeatureFlags::default());
        let mut accumulator = Accumulator::new(true);

        accumulator.scan_input(OutputType::P2PKH, SigOps::new(4), &facts);

        assert_eq!(u32::from(accumulator.finish().executed_legacy), 4);
    }

    #[test]
    fn counts_coinbase_legacy_script_sigops() {
        let input = TxIn {
            script_sig: ScriptBuf::from_hex("ac").unwrap(),
            ..TxIn::default()
        };
        let mut accumulator = Accumulator::new(false);

        accumulator.scan_coinbase_input(&input);

        assert_eq!(u32::from(accumulator.finish().total), 4);
    }

    #[test]
    fn counts_legacy_script_sigops_for_known_prevout_types() {
        let input = TxIn {
            script_sig: ScriptBuf::from_hex("ac").unwrap(),
            ..TxIn::default()
        };
        let facts = input::analyze(&input, OutputType::P2TR, &mut TxFeatureFlags::default());
        let mut accumulator = Accumulator::new(false);

        accumulator.scan_input(OutputType::P2TR, SigOps::ZERO, &facts);

        assert_eq!(u32::from(accumulator.finish().total), 4);
    }

    #[test]
    fn scales_accurately_counted_p2sh_redeem_sigops_by_four() {
        let input = TxIn {
            // Push `OP_2 OP_3 OP_CHECKMULTISIG`; accurate counting uses the
            // preceding OP_3 and therefore counts three sigops.
            script_sig: ScriptBuf::from_hex("035253ae").unwrap(),
            ..TxIn::default()
        };
        let facts = input::analyze(&input, OutputType::P2SH, &mut TxFeatureFlags::default());
        let mut accumulator = Accumulator::new(false);

        accumulator.scan_input(OutputType::P2SH, SigOps::ZERO, &facts);

        assert_eq!(u32::from(accumulator.finish().total), 12);
    }

    #[test]
    fn counts_segwit_v0_sigops_without_legacy_scaling() {
        let p2wpkh = TxIn {
            witness: Witness::from_slice(&[[0_u8; 71].as_slice(), [0_u8; 33].as_slice()]),
            ..TxIn::default()
        };
        let facts = input::analyze(&p2wpkh, OutputType::P2WPKH, &mut TxFeatureFlags::default());
        let mut accumulator = Accumulator::new(false);
        accumulator.scan_input(OutputType::P2WPKH, SigOps::ZERO, &facts);
        assert_eq!(u32::from(accumulator.finish().total), 1);

        let witness_script = [0x52, 0x53, 0xae];
        let p2wsh = TxIn {
            witness: Witness::from_slice(&[[0_u8].as_slice(), witness_script.as_slice()]),
            ..TxIn::default()
        };
        let facts = input::analyze(&p2wsh, OutputType::P2WSH, &mut TxFeatureFlags::default());
        let mut accumulator = Accumulator::new(false);
        accumulator.scan_input(OutputType::P2WSH, SigOps::ZERO, &facts);
        assert_eq!(u32::from(accumulator.finish().total), 3);
    }

    #[test]
    fn excludes_tapscript_from_bip141_sigop_cost() {
        let tapscript = [0xac];
        let control_block = [0xc0; 33];
        let input = TxIn {
            witness: Witness::from_slice(&[tapscript.as_slice(), control_block.as_slice()]),
            ..TxIn::default()
        };
        let facts = input::analyze(&input, OutputType::P2TR, &mut TxFeatureFlags::default());
        let mut accumulator = Accumulator::new(false);

        accumulator.scan_input(OutputType::P2TR, SigOps::ZERO, &facts);

        assert_eq!(u32::from(accumulator.finish().total), 0);
    }
}
