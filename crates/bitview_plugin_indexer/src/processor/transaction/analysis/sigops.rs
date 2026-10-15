use bitcoin::{
    Script, TxIn, TxOut,
    constants::WITNESS_SCALE_FACTOR,
    opcodes::all::{OP_CHECKMULTISIG, OP_CHECKMULTISIGVERIFY, OP_CHECKSIG, OP_CHECKSIGVERIFY},
    script::Instruction,
};
use brk_types::{OutputType, SigOps32};

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

    pub fn scan_output(&mut self, txout: &TxOut, output: &ProcessedOutput) {
        self.legacy = self
            .legacy
            .saturating_add(legacy_sigops_for_output(output, &txout.script_pubkey));
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

/// Legacy sigop count of a script_pubkey, dispatched on `OutputType`.
/// P2PKH and P2PK end in a single OP_CHECKSIG (1); P2SH/P2WPKH/P2WSH/P2TR/P2A/Empty contain no
/// CHECKSIG-class opcodes outside their pushdata. The P2MS classifier also accepts extra signature
/// opcodes before its CHECKMULTISIG, `OpReturn` payloads can hold 0xac/0xae outside a push, and
/// `Unknown` can be anything, so those are counted (an OP_RETURN's at classification).
#[inline]
pub fn legacy_sigops_for_output(output: &ProcessedOutput, script_pubkey: &Script) -> usize {
    match output.output_type {
        OutputType::P2PKH | OutputType::P2PK33 | OutputType::P2PK65 => 1,
        OutputType::P2SH
        | OutputType::P2WPKH
        | OutputType::P2WSH
        | OutputType::P2TR
        | OutputType::P2A
        | OutputType::Empty => 0,
        OutputType::OpReturn => output.op_return_legacy_sigops(),
        OutputType::P2MS | OutputType::Unknown => SigOpCount::of(script_pubkey).legacy,
    }
}

/// Bitcoin Core's `CScript::GetSigOpCount`, both modes in one walk: legacy counts every
/// CHECKMULTISIG as 20, accurate as the `OP_n` right before it (20 without one). Every instruction
/// sets that preceding opcode, signature opcodes included (rust-bitcoin's accurate count keeps a
/// stale `OP_n` across them). A malformed push ends the walk, as in Core.
#[derive(Default)]
pub struct SigOpCount {
    pub legacy: usize,
    pub accurate: usize,
    pushnum: Option<usize>,
}

impl SigOpCount {
    const MAX_PUBKEYS_PER_MULTISIG: usize = 20;

    pub fn of(script: &Script) -> Self {
        let mut count = Self::default();
        for instruction in script.instructions() {
            let Ok(instruction) = instruction else {
                break;
            };
            count.step(instruction);
        }
        count
    }

    #[inline]
    pub fn step(&mut self, instruction: Instruction<'_>) {
        let Instruction::Op(opcode) = instruction else {
            self.pushnum = None;
            return;
        };
        match opcode {
            OP_CHECKSIG | OP_CHECKSIGVERIFY => {
                self.legacy += 1;
                self.accurate += 1;
            }
            OP_CHECKMULTISIG | OP_CHECKMULTISIGVERIFY => {
                self.legacy += Self::MAX_PUBKEYS_PER_MULTISIG;
                self.accurate += self.pushnum.unwrap_or(Self::MAX_PUBKEYS_PER_MULTISIG);
            }
            _ => {}
        }
        let opcode = opcode.to_u8();
        self.pushnum = (0x51..=0x60)
            .contains(&opcode)
            .then(|| usize::from(opcode - 0x50));
    }
}
