use bitcoin::{
    Script,
    opcodes::{Class, ClassifyContext, all::OP_CHECKMULTISIG},
    script::Instruction,
};
use brk_types::SigOpCount;

use super::super::scan::TxScan;

/// A script a spend reveals, a redeem or a witness script, from its one walk.
#[derive(Clone, Copy)]
pub struct Revealed {
    /// Its signature operations, counted accurately.
    pub sigops: usize,
    /// Whether it is an OP_CHECKMULTISIG template: `OP_m <n pushes> OP_n OP_CHECKMULTISIG`, m
    /// from 1 to n.
    pub multisig: bool,
}

impl Revealed {
    pub const NONE: Self = Self {
        sigops: 0,
        multisig: false,
    };

    /// Walks `script` once, up to the first instruction it can't parse: its sigops and template,
    /// and its opcodes and keys for the counters.
    pub fn of(script: &Script, scan: &mut TxScan<'_>) -> Self {
        let mut sigops = SigOpCount::default();
        let (mut instructions, mut pushes) = (0, 0);
        let (mut first, mut before_last, mut last) = (None, None, None);
        for instruction in script.instructions() {
            let Ok(instruction) = instruction else {
                return Self {
                    sigops: sigops.accurate,
                    multisig: false,
                };
            };
            sigops.step(instruction);
            match instruction {
                Instruction::Op(op) => scan.opcode(op),
                Instruction::PushBytes(bytes) => {
                    pushes += 1;
                    scan.input_pubkey(bytes.as_bytes());
                }
            }
            instructions += 1;
            first.get_or_insert(instruction);
            before_last = last.replace(instruction);
        }
        // Three instructions not pushes, the first and the last two: every other one a push.
        let multisig = instructions == pushes + 3
            && last == Some(Instruction::Op(OP_CHECKMULTISIG))
            && matches!(
                (number(first), number(before_last)),
                (Some(required), Some(keys)) if required <= keys && keys == pushes
            );
        Self {
            sigops: sigops.accurate,
            multisig,
        }
    }
}

/// OP_1 to OP_16's number.
fn number(instruction: Option<Instruction<'_>>) -> Option<usize> {
    let Instruction::Op(op) = instruction? else {
        return None;
    };
    match op.classify(ClassifyContext::Legacy) {
        Class::PushNum(n @ 1..=16) => usize::try_from(n).ok(),
        _ => None,
    }
}
